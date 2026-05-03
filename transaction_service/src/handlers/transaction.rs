use uuid::Uuid;
use tokio::sync::{mpsc::Sender};
use tokio_postgres::Client;

use object::interfaces::{
    io::DataKind, 
    service_job::ServiceJob, 
    transaction::{TransactionRequest, TransactionDetail}};

use crate::{
    database::transaction::make_transaction as make_transaction_db, 
    requests::{account::get_account, balance::update_balance}
};

pub async fn make_transaction(
    client: &Client, 
    tx_dealer: &Sender<ServiceJob>,
    session_customer_id: Option<Uuid>,
    transaction_request: TransactionRequest, 
) -> (bool, Option<DataKind>, Option<String>) {
    let from_account_number = transaction_request.from_account_number.clone();
    let to_account_number = transaction_request.to_account_number.clone();

    match (from_account_number, to_account_number) {
        (Some(from_account), Some(to_account)) => {
            if from_account == to_account {
                return (false, None, Some("Error: Invalid parameters".to_string()));
            }
            // Update From account
            let account_result = get_account(tx_dealer, from_account.clone(), session_customer_id).await;
            if let None = account_result {
                return (false, None, Some("Error: Invalid credentials".to_string()));
            }
            let account = account_result.unwrap();
            let new_balance = account.balance - transaction_request.amount;
            if new_balance < 0.0 {
                return (false, None, Some("Error: Insufficient Balance".to_string()));
            }
            if !update_balance(tx_dealer, from_account, new_balance, session_customer_id).await {
                return (false, None, Some("Error: Cannot make transaction".to_string()));
            }

            // Update To Account
            let account_result = get_account(tx_dealer, to_account.clone(), None).await;
            if let None = account_result {
                return (false, None, Some("Error: Invalid credentials".to_string()));
            }
            let account = account_result.unwrap();
            let new_balance = account.balance + transaction_request.amount;
            if !update_balance(tx_dealer, to_account, new_balance, None).await {
                return (false, None, Some("Error: Cannot make transaction".to_string()));
            }
        },
        (None, Some(to_account)) => {
            if session_customer_id.is_some() {
                return (false, None, Some("Error: Invalid parameters".to_string()));
            }
            let account_result = get_account(tx_dealer, to_account.clone(), None).await;
            if let None = account_result {
                return (false, None, Some("Error: Invalid credentials".to_string()));
            }
            let account = account_result.unwrap();
            let new_balance = account.balance + transaction_request.amount;
            if !update_balance(tx_dealer, to_account, new_balance, None).await {
                return (false, None, Some("Error: Cannot make transaction".to_string()));
            }
        },
        (Some(from_account), None) => {
            if session_customer_id.is_some() {
                return (false, None, Some("Error: Invalid parameters".to_string()));
            }
            let account_result = get_account(tx_dealer, from_account.clone(), None).await;
            if let None = account_result {
                return (false, None, Some("Error: Invalid credentials".to_string()));
            }
            let account = account_result.unwrap();
            let new_balance = account.balance - transaction_request.amount;
            if new_balance < 0.0 {
                return (false, None, Some("Error: Insufficient Balance".to_string()));
            }
            if !update_balance(tx_dealer, from_account, new_balance, None).await {
                return (false, None, Some("Error: Cannot make transaction".to_string()));
            }
        },
        (None, None) => return (false, None, Some("Error: Invalid paramaters".to_string())),
    };
    match make_transaction_db(client, transaction_request).await {
        Ok(transaction) => {
            let transaction_detail = TransactionDetail {
                reference_id: transaction.reference_id,
                from_account_number: transaction.from_account_number,
                to_account_number: transaction.to_account_number,
                amount: transaction.amount,
                transaction_timestamp: transaction.transaction_timestamp,
            };
            (true, Some(DataKind::CreateTransactionResponse { transaction_detail }), None)
        }
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to record the transaction".to_string()))
        }
    }
}