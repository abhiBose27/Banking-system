use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob, transaction::{TransactionRequest, TransactionResponse}};


pub async fn make_transaction(
    tx_dealer: &Sender<ServiceJob>,
    transaction_request: TransactionRequest,
    session_customer_id: Option<Uuid>
) -> Option<TransactionResponse> {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::CreateTransaction { transaction_request },
            session_customer_id, 
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
                EventType::Response { id:_, success, error_message, data, session_customer_id: _ } => {
                    match success {
                        true =>  {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CreateTransactionResponse { transaction } => Some(transaction),
                                        _ => {
                                            eprintln!("Error Invalid response received");
                                            None
                                        }
                                    }
                                }
                                None => {
                                    eprintln!("Error: No data received");
                                    None
                                }
                            }
                        },
                        false => {
                            if error_message.is_none() {
                                eprintln!("Error: No error message received");
                            }
                            eprintln!("Error {:?}", error_message.unwrap());
                            None
                        },
                    }
                },
                _ => {
                    eprintln!("Error: Invalid event received");
                    None
                }
            }
        },
        Ok(Err(e)) => panic!("Error: {e}"),
        Err(e) => {
            eprintln!("Error: {e}");
            None
        }
    }
}