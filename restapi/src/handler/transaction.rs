use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, post, web};
use tokio::sync::{mpsc::Sender, oneshot};

use object::interfaces::{authentication::AuthContext, io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob, transaction::TransactionRequest};

#[post("/transaction")]
async fn create(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<TransactionRequest>
) -> impl Responder {
    let auth = request.extensions().get::<AuthContext>().cloned().unwrap();
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateTransaction { transaction_request: api_obj.clone() },
            customer_id: auth.customer_id
        }, 
        from: Service::Api, 
        to: Service::Transaction, 
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

    match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response)) => {
            match response.clone() {
                EventType::Response { id: _, success, error_message, data , customer_id: _} => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    let body = serde_json::to_string(&d).unwrap();
                                    HttpResponse::Ok().body(body)
                                },
                                None => HttpResponse::Ok().body("Success".to_string()),
                            }
                        },
                        false => HttpResponse::BadRequest().body(error_message.unwrap_or("Error: Failed request".to_string())),
                    }
                },
                _ => panic!("Error: Unknown object received on API: {:?}", response.clone())
            }
        },
        Ok(Err(_)) => HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => HttpResponse::RequestTimeout().body("Timed out"),
    }
}