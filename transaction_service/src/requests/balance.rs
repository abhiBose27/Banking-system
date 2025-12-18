use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob};


pub async fn update_balance(
    tx_dealer: &Sender<ServiceJob>, 
    account_number: String,
    balance: f64
) ->  bool {

    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::UpdateBalance { account_number, balance } 
        },
        from: Service::Transaction,
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
                EventType::Response { id:_, success, error_message:_, data:_ } => success,
                _ => panic!("Error: Invalid event received")
            }
        },
        Ok(Err(e)) => panic!("Error: {e}"),
        Err(e) => {
            eprintln!("Error: {e}");
            false
        }
    }
}