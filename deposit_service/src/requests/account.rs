use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{account::Account, io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob};


pub async fn get_account(
    tx_dealer: &Sender<ServiceJob>, 
    account_number: String,
    customer_id: Option<Uuid>
) ->  Option<Account> {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id, 
            data: DataKind::GetAccount { account_number },
            customer_id, 
        },
        from: Service::Deposit,
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
                EventType::Response { id:_, success:_, error_message:_, data, customer_id: _ } => {
                    match data {
                        Some(d) => {
                            match d {
                                DataKind::GetAccountResponse { account } => Some(account),
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