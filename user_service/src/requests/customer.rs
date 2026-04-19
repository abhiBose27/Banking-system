use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use ulid::Ulid;
use uuid::Uuid;

use object::interfaces::{
    customer::Customer, 
    io::{DataKind, EventMessage, EventType, Service}, 
    service_job::ServiceJob
};


pub async fn get_customer(
    tx_dealer: &Sender<ServiceJob>, 
    customer_reference_id: Ulid
) ->  Option<Customer> {

    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let request_id = Uuid::new_v4();
    let request = EventMessage {
        data: EventType::Request { 
            id: request_id,
            session_customer_id: None,
            data: DataKind::GetPvtCustomer { customer_reference_id: Some(customer_reference_id) }
        },
        from: Service::User,
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
                EventType::Response { id:_, success, error_message, session_customer_id:_, data } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::GetCustomerPvtResponse { customer } => Some(customer),
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
        Ok(Err(e)) => {
            eprintln!("Error: {e}");
            None
        },
        Err(e) => {
            eprintln!("Error: {e}");
            None
        }
    }
}