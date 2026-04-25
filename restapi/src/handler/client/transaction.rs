use std::time::Duration;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, post, web};
use chrono::Utc;
use deadpool_redis::Pool;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    authentication::AuthContext, 
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob, transaction::TransactionRequest
};

use crate::cache::redis::is_logged_in_with_token;


#[post("/transaction")]
async fn create_transaction(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    payload: web::Json<TransactionRequest>,
    redis_pool: web::Data<Pool>
) -> impl Responder {
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    if let None = auth_context {
        return HttpResponse::BadRequest().body("Error: Unable to retrieve auth context")
    }
    let session_customer_id = Some(auth_context.clone().unwrap().customer_id);

    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Redis Error {e}");
            return HttpResponse::InternalServerError().body("Error: Redis Unavailable")
        }
    };

    // Check if the user is logged in
    let is_logged_in = is_logged_in_with_token(&auth_context.unwrap().token, &mut conn).await;
    if let Err(e) = is_logged_in {
        eprintln!("Error: Redis error {e}");
        return HttpResponse::InternalServerError().finish();
    }
    if let false = is_logged_in.unwrap() {
        return HttpResponse::BadRequest().body("Not logged in");
    }

    // Send the request for making a transaction
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateTransaction { transaction_request: payload.clone() },
            session_customer_id
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