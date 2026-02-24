use uuid::Uuid;
use tokio::sync::{mpsc::Sender};
use tokio_postgres::Client;

use object::interfaces::{
    io::DataKind, 
    service_job::ServiceJob, 
    transaction::{TransactionRequest, TransactionResponse}};

use crate::{
    database::transaction::make_transaction as make_transaction_db, 
    requests::{account::get_account, balance::update_balance}
};

pub async fn make_transaction(
    client: &Client, 
    tx_dealer: &Sender<ServiceJob>,
    customer_id: Option<Uuid>,
    transaction_request: TransactionRequest, 
) -> (bool, Option<DataKind>, Option<String>) {
    if let Some(account_number) = transaction_request.from_account_number.clone() {
        let account_result = get_account(tx_dealer, account_number.clone(), customer_id).await;
        if let None = account_result {
            return (false, None, Some("Error: Cannot fetch account details".to_string()));
        }
        
        let account = account_result.unwrap();
        if customer_id.is_some() && customer_id.unwrap() != account.customer_id {
            return (false, None, Some("Error: Invalid customer id".to_string()));
        }

        let new_balance = account.balance - transaction_request.amount;
        if new_balance < 0.0 {
            return (false, None, Some("Error: Insufficient Balance".to_string()));
        }
        if !update_balance(tx_dealer, account_number.clone(), new_balance, customer_id).await {
            return (false, None, Some("Error: Cannot make transaction".to_string()));
        }
    }
    if let Some(account_number) = transaction_request.to_account_number.clone() {
        let account_result = get_account(tx_dealer, account_number.clone(), customer_id).await;
        if let None = account_result {
            return (false, None, Some("Error: Invalid credentials".to_string()));
        }
        let account = account_result.unwrap();
        let new_balance = account.balance + transaction_request.amount;
        if !update_balance(tx_dealer, account_number.clone(), new_balance, customer_id).await {
            return (false, None, Some("Error: Cannot make transaction".to_string()));
        }
    }
    match make_transaction_db(client, transaction_request.clone()).await {
        Ok(transaction) => {
            let transaction_api = TransactionResponse {
                reference_id: transaction.reference_id,
                from_account_number: transaction.from_account_number,
                to_account_number: transaction.to_account_number,
                amount: transaction.amount,
                transaction_timestamp: transaction.transaction_timestamp,
            };
            (true, Some(DataKind::CreateTransactionResponse { transaction: transaction_api.clone() }), None)
        }
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to record the transaction".to_string()))
        }
    }
}