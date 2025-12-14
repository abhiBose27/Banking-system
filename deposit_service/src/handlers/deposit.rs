use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use tokio_postgres::Client;
use uuid::Uuid;

use object::interfaces::{
    account::Account, deposit::{DepositClose, DepositRequest, DepositResponse, DepositTenure, InterestPayout}, 
    io::{DataKind, EventMessage, EventType, Service}, 
    service_job::ServiceJob, 
    transaction::{TransactionRequest, TransactionResponse, TransactionStatus}
};

use crate::database::deposit::{add_deposit, close_deposit as close_deposit_db, get_deposit};

fn is_valid_interest_payout(interest_payout: InterestPayout, deposit_tenure: Option<DepositTenure>) -> bool {
    match deposit_tenure {
        Some(d) => {
            let total_days = d.days + d.months * 30 + d.years * 365;
            match interest_payout {
                InterestPayout::Monthly => total_days >= 30,
                InterestPayout::Quaterly => total_days >= 91,
                _ => true,
            }
        },
        None => true
    }
}

fn is_valid_deposit_tenure(deposit_tenure: Option<DepositTenure>) -> bool {
    match deposit_tenure {
        Some(d) => !(d.days == 0 && d.months == 0 && d.years == 0),
        None => true
    }
}

async fn get_account(
    tx_dealer: &Sender<ServiceJob>, 
    account_number: String
) ->  Option<Account> {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let mut result = None;
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::GetAccount { account_number } 
        },
        from: Service::Deposit,
        to: Service::Account,
        timestamp: Utc::now()
    };

    let service_job = ServiceJob {
        tx_job: Some(tx_job),
        data: request
    };

    tx_dealer.send(service_job).await.unwrap();

    match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response)) => {
            match response {
                EventType::Response { id:_, success:_, error_message:_, data } => {
                    if let Some(datakind) = data {
                        match datakind {
                            DataKind::GetAccountResponse { account } => result = Some(account),
                            _ => panic!("Error: Invalid object received")
                        }
                    }
                },
                _ => panic!("Error: Invalid response received")
            }
        },
        Ok(Err(e)) => panic!("Error: {e}"),
        Err(e) => eprintln!("Error: {e}"),
    };
    return result;
}

pub async fn make_transaction(
    tx_dealer: &Sender<ServiceJob>, 
    transaction_request: TransactionRequest
) -> Option<TransactionResponse> {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let mut result = None;
    let request_id = Uuid::new_v4();
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::CreateTransaction { transaction_request: transaction_request.clone() } },
        from: Service::Deposit,
        to: Service::Transaction,
        timestamp: Utc::now(),
    };

    let service_job = ServiceJob {
        tx_job: Some(tx_job),
        data: request
    };

    tx_dealer.send(service_job).await.unwrap();

    match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response)) => {
            match response {
                EventType::Response { id:_, success:_, error_message:_, data } => {
                    if let Some(datakind) = data {
                        match datakind {
                            DataKind::CreateTransactionResponse { transaction } => result = Some(transaction),
                            _ => panic!("Error: Invalid object received")
                        }
                    }
                },
                _ => panic!("Error: Invalid response received")
            }
        },
        Ok(Err(e)) => panic!("Error: {e}"),
        Err(e) => eprintln!("Error: {e}"),
    };
    result
}

pub async fn close_deposit(
    client: &Client,
    tx_dealer: &Sender<ServiceJob>,
    deposit_close: DepositClose
) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut data = None;
    let mut error_message = None;
    let deposit_result = get_deposit(client, deposit_close.deposit_number.clone()).await;
    if let Err(e) = deposit_result {
        eprintln!("Error: {e}");
        return (success, data, Some("Error: Invalid deposit_number".to_string()));
    }
    //let mut paid_interest = 0.0;
    let deposit = deposit_result.unwrap();

    let current_date = Utc::now().date_naive();
    let creation_date = deposit.creation_timestamp.date_naive();
    let days_spanned = (current_date - creation_date).num_days();

    let premature_interest = deposit.principal_amount * (days_spanned as f64) * (deposit.interest_rate / 100.0) / 365.0;
    let paid_interest = deposit.total_interest_paid;

    let difference = premature_interest - paid_interest;
    let total_to_pay = deposit.principal_amount + difference;

    let transaction_request = TransactionRequest {
        amount: total_to_pay,
        from_account_number: None,
        to_account_number: Some(deposit.linked_account_number.clone())
    };
    let transaction_response = make_transaction(tx_dealer, transaction_request).await;
    if let None = transaction_response {
        return (success, data, Some("Error: Unable to make transaction".to_string()));
    }
    let transaction = transaction_response.unwrap();
    if transaction.transaction_status == TransactionStatus::Reject {
        return (success, data, Some("Error: Unable to make transaction".to_string()));
    }

    match close_deposit_db(client, deposit.id).await {
        Ok(_) => {
            success = true;
            data = Some(DataKind::CloseDepositResponse);
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to close deposit".to_string());
        },
    };
    (success, data, error_message)
}

pub async fn create_deposit(
    client: &Client, 
    tx_dealer: &Sender<ServiceJob>, 
    deposit_request: DepositRequest
) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut data = None;
    let mut error_message = None;

    let deposit_request_clone = deposit_request.clone();
    let deposit_tenure = deposit_request_clone.deposit_tenure;
    let renewed_deposit_tenure = deposit_request_clone.renewed_deposit_tenure;
    let interest_payout = deposit_request_clone.interest_payout;
    
    if deposit_request_clone.auto_renewal && renewed_deposit_tenure.is_none() {
        return (success, data, Some("Error: Invalid auto renewal enabled without tenure".to_string()));
    }
    if interest_payout == InterestPayout::Renew && !deposit_request_clone.auto_renewal {
        return (success, data, Some("Error: Interest payout to renew without auto renewal".to_string()));
    }
    if !is_valid_deposit_tenure(Some(deposit_tenure.clone())) {
        return (success, data, Some("Error: Invalid deposit tenure".to_string()));
    }
    if !is_valid_deposit_tenure(renewed_deposit_tenure.clone()) {
        return (success, data, Some("Error: Invalid renewed deposit tenure".to_string()));
    }
    if !is_valid_interest_payout(interest_payout.clone(), Some(deposit_tenure.clone())) {
        return (success, data, Some("Error: Invalid interest payout".to_string()));
    }
    if !is_valid_interest_payout(interest_payout.clone(), renewed_deposit_tenure.clone()) {
        return (success, data, Some("Error: Invalid interest payout".to_string()));
    }

    let account_result = get_account(tx_dealer, deposit_request.linked_account_number.clone()).await;
    if let None = account_result {
        return (success, data, Some("Error: Cannot fetch account details".to_string()));
    }
    let account = account_result.unwrap();

    let transaction_request = TransactionRequest {
        amount: deposit_request.principal_amount.clone(),
        from_account_number: Some(deposit_request.linked_account_number.clone()),
        to_account_number: None,
    };
    let transaction_result = make_transaction(tx_dealer, transaction_request).await;
    if let None = transaction_result {
        return (success, data, Some("Error: Cannot make transaction".to_string()));
    }
    let transaction = transaction_result.unwrap();
    if transaction.transaction_status == TransactionStatus::Reject {
        return (success, data, Some("Error: Cannot make transaction".to_string()));
    }

    match add_deposit(client, deposit_request, account.customer_id).await {
        Ok(deposit) => {
            success = true;
            let deposit_api = DepositResponse {
                deposit_number: deposit.deposit_number,
                linked_account_number: deposit.linked_account_number,
                principal_amount: deposit.principal_amount,
                interest_rate: deposit.interest_rate,
                interest_payout: deposit.interest_payout,
                auto_renewal: deposit.auto_renewal,
                maturity_date: deposit.maturity_date,
                deposit_tenure: deposit.deposit_tenure,
                renewed_deposit_tenure: deposit.renewed_deposit_tenure,
                creation_timestamp: deposit.creation_timestamp,
            };
            data = Some(DataKind::CreateDepositResponse { deposit: deposit_api.clone() });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to open deposit account".to_string());
        },
    };

    (success, data, error_message)
}