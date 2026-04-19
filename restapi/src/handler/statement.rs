use std::time::Duration;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web};
use chrono::Utc;
use deadpool_redis::Pool;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{authentication::AuthContext, io::{DataKind, EventMessage, EventType, Service}, service_job::ServiceJob, statement::StatementRequest};

use crate::authentication::redis::is_logged_in_with_token;

#[get("/statement")]
async fn get(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<StatementRequest>,
    pool: web::Data<Pool>
) -> impl Responder {
    let session_customer_id = match request.extensions().get::<AuthContext>().cloned() {
        Some(auth_context) => {
            let mut conn = match pool.get().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: Redis Error {e}");
                    return HttpResponse::InternalServerError().body("Error: Redis Unavailable")
                }
            };

            // Check if the user is logged in
            let is_logged_in = is_logged_in_with_token(&auth_context.token, &mut conn).await;
            if let Err(e) = is_logged_in {
                eprintln!("Error: Redis error {e}");
                return HttpResponse::InternalServerError().finish();
            }
            if let false = is_logged_in.unwrap() {
                return HttpResponse::BadRequest().body("Not logged in");
            }
            Some(auth_context.customer_id)
        },
        None => None,
    };

    // Send the request to get statement
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::GetStatement { statement_request: api_obj.clone() },
            session_customer_id
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
                                            eprintln!("Error: Invalid response received for getting User");
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