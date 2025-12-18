use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob, transaction::{TransactionRequest, TransactionResponse}};


pub async fn make_transaction(
    tx_dealer: &Sender<ServiceJob>, 
    transaction_request: TransactionRequest
) -> Option<TransactionResponse> {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::CreateTransaction { 
                transaction_request: transaction_request.clone() 
            } 
        },
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
                    match data {
                        Some(d) => {
                            match d {
                                DataKind::CreateTransactionResponse { transaction } => Some(transaction),
                                _ => panic!("Error: Invalid datakind received")
                            }
                        }
                        None => None
                    }
                },
                _ => panic!("Error: Invalid event received")
            }
        },
        Ok(Err(e)) => panic!("Error: {e}"),
        Err(e) => {
            eprintln!("Error: {e}");
            None
        }
    }
}