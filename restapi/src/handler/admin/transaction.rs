use std::time::Duration;
use uuid::Uuid;
use actix_web::{HttpResponse, Responder, post, web};
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};

use object::interfaces::{
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob, transaction::TransactionRequest
};


#[post("/transaction")]
async fn create_transaction(
    tx: web::Data<Sender<ServiceJob>>,
    payload: web::Json<TransactionRequest>
) -> impl Responder {

    // Send the request for making a transaction
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateTransaction { transaction_request: payload.clone() },
            session_customer_id: None
        }, 
        from: ServiceType::Api, 
        to: ServiceType::Transaction, 
        timestamp: Utc::now() 
    };

    let service_job = ServiceJob { 
        data: event_message,
        tx_job: Some(tx_job)
    };

    if let Err(e) = tx.send(service_job).await {
        eprintln!("Failed to send job: {e}");
        return HttpResponse::InternalServerError().finish();
    }

    // Wait for the response
    match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response)) => {
            match response.clone() {
                EventType::Response { id: _, success, error_message, data , session_customer_id: _} => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CreateTransactionResponse { transaction } => return HttpResponse::Ok().body(serde_json::to_string(&transaction).unwrap()),
                                        _ => {
                                            eprintln!("Error: Invalid response received for making transaction");
                                            return HttpResponse::InternalServerError().finish();
                                        }
                                    }
                                },
                                None => {
                                    eprintln!("Error: No response data received");
                                    return HttpResponse::InternalServerError().finish();
                                }
                            }
                        },
                        false => {
                            if error_message.is_none() {
                                eprintln!("Error: No error message received");
                                return HttpResponse::InternalServerError().finish();
                            }
                            return HttpResponse::InternalServerError().body(error_message.unwrap());
                        }
                    }
                },
                _ => {
                    eprintln!("Error: Unknown object received on API: {:?}", response.clone());
                    return HttpResponse::InternalServerError().finish();
                }
            }
        },
        Ok(Err(_)) => HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => HttpResponse::RequestTimeout().body("Timed out"),
    }
}