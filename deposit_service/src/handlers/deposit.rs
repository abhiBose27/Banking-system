use ulid::Ulid;
use uuid::Uuid;
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{
    deposit::{DepositDetail, DepositRequest, InterestPayout}, 
    io::DataKind, 
    service_job::ServiceJob, 
    transaction::TransactionRequest
};

use crate::{
    database::deposit::{
        add_deposit, 
        close_deposit as close_deposit_db, 
        get_deposit_from_deposit_number, 
        get_deposits_from_customer_id
    }, 
    requests::{
        account::get_account, 
        customer::get_customer, 
        transaction::make_transaction
    }, utils::{deposit::{get_premature_withdrawal_amount, is_valid_deposit_tenure}, interest::is_valid_interest_payout}
};

pub async fn get_deposit(
    client: &Client,
    deposit_number: String,
    session_customer_id: Option<Uuid>
) -> (bool, Option<DataKind>, Option<String>) {
    let deposit_result = get_deposit_from_deposit_number(client, deposit_number).await;
    if let Err(e) = deposit_result {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Invalid deposit_number".to_string()));
    }

    let deposit = deposit_result.unwrap();
    if session_customer_id.is_some() && deposit.customer_id != session_customer_id.unwrap() {
        return (false, None, Some("Error: Invalid customer id".to_string()));
    }
    let deposit_detail = DepositDetail {
        deposit_number: deposit.deposit_number,
        linked_account_number: deposit.linked_account_number,
        principal_amount: deposit.principal_amount,
        interest_rate: deposit.interest_rate,
        deposit_tenure: deposit.deposit_tenure,
        interest_payout: deposit.interest_payout,
        total_interest_amount: deposit.total_interest_amount,
        auto_renewal: deposit.auto_renewal,
        renewed_deposit_tenure: deposit.renewed_deposit_tenure,
        maturity_date: deposit.maturity_date,
        creation_timestamp: deposit.creation_timestamp,
    };
    (true, Some(DataKind::GetDepositDetailResponse { deposit_detail }), None)
}

pub async fn get_deposits(
    client: &Client,
    tx_dealer: &Sender<ServiceJob>,
    customer_reference_id: Option<Ulid>,
    session_customer_id: Option<Uuid>
) -> (bool, Option<DataKind>, Option<String>) {
    let customer_result = get_customer(tx_dealer, customer_reference_id, session_customer_id).await;
    if let None = customer_result {
        return (false, None, Some("Error: Invalid parameters".to_string()));
    }
    
    let deposits_result = get_deposits_from_customer_id(client, customer_result.unwrap().id).await;
    if let Err(e) = deposits_result {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Invalid to fetch deposits".to_string()));
    }
    let deposits_detail = deposits_result.unwrap().into_iter().map(|deposit| DepositDetail {
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
        total_interest_amount: deposit.total_interest_amount,
    }).collect::<Vec<DepositDetail>>();
    (true, Some(DataKind::GetDepositsDetailResponse { deposits_detail }), None)

}

pub async fn close_deposit(
    client: &Client,
    tx_dealer: &Sender<ServiceJob>,
    deposit_number: String,
    session_customer_id: Option<Uuid>
) -> (bool, Option<DataKind>, Option<String>) {
    let deposit_result = get_deposit_from_deposit_number(client, deposit_number).await;
    if let Err(e) = deposit_result {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Invalid deposit_number".to_string()));
    }

    let deposit = deposit_result.unwrap();
    if session_customer_id.is_some() && deposit.customer_id != session_customer_id.unwrap() {
        return (false, None, Some("Error: Invalid customer id".to_string()));
    }

    let total_to_pay = get_premature_withdrawal_amount(
        deposit.principal_amount, 
        deposit.interest_rate, 
        deposit.total_interest_paid,
        deposit.creation_timestamp, 
    );
    
    let transaction_request = TransactionRequest {
        amount: total_to_pay,
        from_account_number: None,
        to_account_number: Some(deposit.linked_account_number.clone())
    };
    let transaction_response = make_transaction(tx_dealer, transaction_request, None).await;
    if let None = transaction_response {
        return (false, None, Some("Error: Unable to make transaction".to_string()));
    }

    match close_deposit_db(client, deposit.id).await {
        Ok(_) => (true, Some(DataKind::CloseDepositResponse), None),
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to close deposit".to_string()))
        },
    }
}

pub async fn create_deposit(
    client: &Client, 
    tx_dealer: &Sender<ServiceJob>, 
    deposit_request: DepositRequest,
    session_customer_id: Option<Uuid>
) -> (bool, Option<DataKind>, Option<String>) {
    // Get the customer id from linked account number
    let account_result = get_account(tx_dealer, deposit_request.linked_account_number.clone(), session_customer_id).await;
    if let None = account_result {
        return (false, None, Some("Error: Cannot fetch account details".to_string()));
    }

    let deposit_request_clone = deposit_request.clone();
    let deposit_tenure = deposit_request_clone.deposit_tenure;
    let interest_payout = deposit_request_clone.interest_payout;
    let renewed_deposit_tenure = deposit_request_clone.renewed_deposit_tenure;

    // Validate the deposit request
    if deposit_request_clone.auto_renewal && renewed_deposit_tenure.is_none() {
        return (false, None, Some("Error: Invalid auto renewal enabled without tenure".to_string()));
    }
    if interest_payout == InterestPayout::Renew && !deposit_request_clone.auto_renewal {
        return (false, None, Some("Error: Interest payout to renew without auto renewal".to_string()));
    }
    if !is_valid_deposit_tenure(Some(deposit_tenure.clone())) {
        return (false, None, Some("Error: Invalid deposit tenure".to_string()));
    }
    if !is_valid_deposit_tenure(renewed_deposit_tenure.clone()) {
        return (false, None, Some("Error: Invalid renewed deposit tenure".to_string()));
    }
    if !is_valid_interest_payout(&interest_payout, Some(deposit_tenure.clone())) {
        return (false, None, Some("Error: Invalid interest payout".to_string()));
    }
    if !is_valid_interest_payout(&interest_payout, renewed_deposit_tenure.clone()) {
        return (false, None, Some("Error: Invalid interest payout".to_string()));
    }

    // Make the required transaction
    let transaction_request = TransactionRequest {
        amount: deposit_request.principal_amount.clone(),
        from_account_number: Some(deposit_request.linked_account_number.clone()),
        to_account_number: None,
    };
    let transaction_result = make_transaction(tx_dealer, transaction_request, None).await;
    if let None = transaction_result {
        return (false, None, Some("Error: Cannot make transaction".to_string()));
    }

    match add_deposit(client, account_result.unwrap().customer_id, deposit_request).await {
        Ok(deposit) => {
            let deposit_detail = DepositDetail {
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
                total_interest_amount: deposit.total_interest_amount,
            };
            (true, Some(DataKind::CreateDepositResponse { deposit_detail }), None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to open deposit account".to_string()))
        },
    }

}