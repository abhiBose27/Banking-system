use chrono::Utc;
use tokio_postgres::Client;
use uuid::Uuid;

use object::interfaces::{
    dealer::Dealer, io::{DataKind, EventMessage, EventType, Service}, transaction::{TransactionRequest, TransactionStatus}};

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
    /* let payload = serde_json::to_vec(&transaction_request).unwrap();
    if let Err(e) = dealer_socket.send(payload.into()).await {
        eprintln!("Error: {e}");
        return false;
    } */
    if let Some(response_message) = dealer.recv_event(Some(request_id)).await {
        match response_message.data {
            EventType::Response { id, executed, error_message:_, data:_ } => {
                assert_eq!(id, request_id);
                return executed
            },
            _ => panic!("Error: Invalid object")
        }
    }
    return false;
    /* let is_valid = match timeout(Duration::from_secs(5), dealer_socket.recv()).await {
        Ok(Ok(resp)) => {
            let resp_clone = resp.clone();
            let raw_message = resp_clone.get(0).unwrap();
            let resp_event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
            assert_eq!(resp_event_message.from, Service::Account);
            assert_eq!(resp_event_message.to, Service::Transaction);
            match resp_event_message.data {
                EventType::Response { id, executed, error_message:_, data:_ } => {
                    assert_eq!(id, request_id);
                    executed
                },
                _ => panic!("Error: Invalid object")
            }
        }
        Ok(Err(e)) => {
            eprintln!("Error: {e}");
            false
        }
        Err(e) => {
            eprintln!("Error: {e}");
            false
        } 
    };
    is_valid */
}

pub async fn make_transaction(
    client: &Client, 
    dealer: &mut Dealer,
    transaction_request: TransactionRequest, 
) -> (bool, Option<DataKind>, Option<String>) {
    let mut executed = false;
    let mut error_message = None;
    let mut data = None;
    let is_valid_transaction = is_valid_transaction(dealer, transaction_request.clone()).await;
    let transaction_status = if is_valid_transaction {TransactionStatus::Complete} else {TransactionStatus::Reject};
    match make_transaction_db(client, transaction_request.clone(), transaction_status).await {
        Ok(reference_id) => {
            executed = true;
            data = Some(DataKind::Transaction { reference_id });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to make transaction".to_string());
        },
    }
    (executed, data, error_message)
}