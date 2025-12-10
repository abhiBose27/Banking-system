use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use tokio_postgres::Client;
use uuid::Uuid;

use object::interfaces::{
    io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob, transaction::{TransactionRequest, TransactionResponse, TransactionStatus}};

use crate::database::transaction::make_transaction_db;

// Ask account services 
// if the transaction is valid
async fn is_valid_transaction(
    tx_dealer: &Sender<ServiceJob>,
    transaction_request: TransactionRequest, 
) -> bool {
    let from_acc = transaction_request.clone().from_account_number;
    let to_acc = transaction_request.clone().to_account_number;
    if let (None, None) = (from_acc, to_acc) {
        return false
    }

    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let update_balance_request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::UpdateBalance {
                transaction_request: transaction_request.clone()
            },
        },
        from: Service::Transaction,
        to: Service::Account,
        timestamp: Utc::now(),
    };

    let service_job = ServiceJob {
        tx_job: Some(tx_job),
        data: update_balance_request
    };

    tx_dealer.send(service_job).await.unwrap();

    match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response_message)) => {
            match response_message {
                EventType::Response { id:_, success, error_message:_, data:_ } => success,
                _ => panic!("Error: Invalid message")
            }
        },
        Ok(Err(e)) => panic!("Error: {e}"),
        Err(e) => {
            eprintln!("Error: {e}");
            false
        },
    }
}

pub async fn make_transaction(
    client: &Client, 
    tx_dealer: &Sender<ServiceJob>,
    transaction_request: TransactionRequest, 
) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut error_message = None;
    let mut data = None;
    let is_valid_transaction = is_valid_transaction(tx_dealer, transaction_request.clone()).await;
    let transaction_status = if is_valid_transaction {TransactionStatus::Complete} else {TransactionStatus::Reject};
    match make_transaction_db(client, transaction_request.clone(), transaction_status).await {
        Ok(transaction) => {
            success = true;
            let transaction_api = TransactionResponse {
                reference_id: transaction.reference_id,
                from_account_number: transaction.from_account_number,
                to_account_number: transaction.to_account_number,
                transaction_status: transaction.transaction_status,
                amount: transaction.amount,
                transaction_timestamp: transaction.transaction_timestamp,
            };
            data = Some(DataKind::CreateTransactionResponse { transaction: transaction_api.clone() });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to make transaction".to_string());
        },
    }
    (success, data, error_message)
}