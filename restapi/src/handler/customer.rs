use uuid::Uuid;
use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use actix_web::{post, web, HttpResponse, Responder};

use object::interfaces::{customer::CustomerRequest, io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob};


#[post("/customer")]
async fn create(
    tx: web::Data<Sender<ServiceJob>>, 
    api_obj: web::Json<CustomerRequest>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request {
            id: Uuid::new_v4(),
            data: DataKind::CreateCustomer { customer_request: api_obj.clone() }, 
            customer_id: None 
        },
        from: Service::Api,
        to: Service::Account,
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
                EventType::Response { id: _, success, error_message, data, customer_id: _ } => {
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