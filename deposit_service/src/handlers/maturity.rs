use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{deposit::{DepositRequest, DepositDetail}, service_job::ServiceJob, transaction::{TransactionRequest}};

use crate::{
    database::{
        deposit::{add_deposit, close_deposit}, 
        maturity::get_deposit_for_maturity}, 
        requests::transaction::make_transaction
    };

pub async fn process_maturity(client: &Client, tx_dealer: &Sender<ServiceJob>) {
    let deposit_for_maturity = get_deposit_for_maturity(client).await.unwrap();
    for deposit in deposit_for_maturity {        
        // Calculate the new amount
        let amount= deposit.principal_amount + (deposit.total_interest_amount - deposit.total_interest_paid);
        let transaction_request = TransactionRequest {
            amount,
            from_account_number: None,
            to_account_number: Some(deposit.linked_account_number.clone()),
        };
        let transaction_response = make_transaction(tx_dealer, transaction_request, None).await;
        if let None = transaction_response {
            eprintln!("Error: Unable to credit principal amount {} to {}", amount, deposit.linked_account_number);
            continue;
        } 
        // Close the current deposit
        let close_deposit_response = close_deposit(client, deposit.id).await;
        if let Err(e) = close_deposit_response {
            eprintln!("Error {e}: Unable to close deposit id: {:?}", deposit.id);
            continue;
        }

        if !deposit.auto_renewal {
            continue;
        }
        // Renewed deposit request
        let new_deposit_request = DepositRequest {
            linked_account_number: deposit.linked_account_number,
            principal_amount: amount,
            deposit_tenure: deposit.renewed_deposit_tenure.clone().unwrap(),
            interest_payout: deposit.interest_payout,
            auto_renewal: deposit.auto_renewal,
            renewed_deposit_tenure: deposit.renewed_deposit_tenure,
        };
        let transaction_request_debit = TransactionRequest {
            amount,
            from_account_number: Some(new_deposit_request.linked_account_number.clone()),
            to_account_number: None
        };
        let transaction_response_debit = make_transaction(tx_dealer, transaction_request_debit, None).await;
        if let None = transaction_response_debit {
            eprintln!("Error: Unable to debit principal amount {} to {}", amount, new_deposit_request.linked_account_number);
            continue;
        }

        // Add deposit to DB
        match add_deposit(client, deposit.customer_id, new_deposit_request).await {
            Ok(new_deposit) => {
                let deposit_response = DepositDetail {
                    deposit_number: new_deposit.deposit_number,
                    linked_account_number: new_deposit.linked_account_number,
                    principal_amount: new_deposit.principal_amount,
                    interest_rate: new_deposit.interest_rate,
                    interest_payout: new_deposit.interest_payout,
                    auto_renewal: new_deposit.auto_renewal,
                    maturity_date: new_deposit.maturity_date,
                    deposit_tenure: new_deposit.deposit_tenure,
                    renewed_deposit_tenure: new_deposit.renewed_deposit_tenure,
                    creation_timestamp: new_deposit.creation_timestamp,
                    total_interest_amount: new_deposit.total_interest_amount,
                };
                println!("Renewed deposit {:?}", deposit_response);
            },
            Err(e) => eprintln!("Error {e}: Failed to renew deposit id {:?}", deposit.id),
        };
    }
}