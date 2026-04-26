use std::time::Duration;
use actix_web::{HttpResponse, Responder, get, web};
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob, statement::StatementRequest
};


#[get("/statement/{account_number}")]
async fn get_statement(
    tx: web::Data<Sender<ServiceJob>>,
    path: web::Path<String>,
    payload: web::Json<StatementRequest>
) -> impl Responder {
    
    // Send the request to get statement
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::GetStatement {statement_request: payload.clone(), account_number: path.into_inner() },
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
                EventType::Response { id:_, success, error_message, data, session_customer_id: _ } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::GetStatementResponse { statement } => return HttpResponse::Ok().body(serde_json::to_string(&statement).unwrap()),
                                        _ => {
                                            eprintln!("Error: Invalid response received for getting statement");
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
                }
                _ => {
                    eprintln!("Error: Unknown object received on API: {:?}", response.clone());
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
        Ok(Err(_)) => HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => HttpResponse::RequestTimeout().body("Timed out"),
    }
}
