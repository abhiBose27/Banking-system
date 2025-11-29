use chrono::Utc;
use tokio_postgres::Client;
use uuid::Uuid;

use object::interfaces::{
    dealer::Dealer, io::{DataKind, EventMessage, EventType, Service}, transaction::{TransactionResponse, TransactionRequest, TransactionStatus}};

use crate::database::transaction::make_transaction_db;

// Ask account services 
// if the transaction is valid
async fn is_valid_transaction(
    dealer: &mut Dealer,
    transaction_request: TransactionRequest, 
) -> bool {
    let from_acc = transaction_request.clone().from_account_number;
    let to_acc = transaction_request.clone().to_account_number;
    if let (None, None) = (from_acc, to_acc) {
        return false
    }
    
    let request_id = Uuid::new_v4();
    let transaction_request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::UpdateBalance {
                transaction_request: TransactionRequest { 
                    amount: transaction_request.amount, 
                    from_account_number: transaction_request.from_account_number, 
                    to_account_number: transaction_request.to_account_number 
                }
            },
        },
        from: Service::Transaction,
        to: Service::Account,
        timestamp: Utc::now(),
    };
    if !dealer.send_event(transaction_request.clone()).await {
        eprintln!("Error: Cannot send message: {:?}", transaction_request);
        return false;
    }
    if let Some(response_message) = dealer.recv_event(Some(request_id)).await {
        match response_message.data {
            EventType::Response { id, success, error_message:_, data:_ } => {
                assert_eq!(id, request_id);
                return success
            },
            _ => panic!("Error: Invalid object")
        }
    }
    return false;
}

pub async fn make_transaction(
    client: &Client, 
    dealer: &mut Dealer,
    transaction_request: TransactionRequest, 
) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut error_message = None;
    let mut data = None;
    let is_valid_transaction = is_valid_transaction(dealer, transaction_request.clone()).await;
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