use std::time::Duration;
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use actix_web::{post, web, HttpResponse, Responder};
use object::interfaces::{customer::CustomerRequest, io::{DataKind, EventMessage, EventType, Service}};
use uuid::Uuid;

use crate::interfaces::dealer::ServiceJob;


#[post("/customer")]
async fn create(
    tx: web::Data<Sender<ServiceJob>>, 
    api_obj: web::Json<CustomerRequest>
) -> impl Responder {
    let (response_tx, response_rx) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateCustomer { customer_request: api_obj.clone() }
        },
        from: Service::Api,
        to: Service::Account,
        timestamp: Utc::now()
    };
    let service_job = ServiceJob { 
        data: event_message,
        response_tx
    };
    if let Err(e) = tx.send(service_job).await {
        eprintln!("Failed to send job: {e}");
        return HttpResponse::InternalServerError().finish();
    }
    match tokio::time::timeout(Duration::from_secs(5), response_rx).await {
        Ok(Ok(response)) => {
            match response.clone() {
                EventType::Response { id: _, executed, error_message, data:_ } => {
                    if !executed {
                        HttpResponse::BadRequest().body(error_message.unwrap())
                    }
                    else {
                        HttpResponse::Ok().body("Successfully created the customer")
                    }
                },
                _ => panic!("Error: Unknown object received on API: {:?}", response.clone())
            }
        },
        Ok(Err(_)) => HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => HttpResponse::RequestTimeout().body("Timed out"),
    }
}