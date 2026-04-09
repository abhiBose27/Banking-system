use std::{time::Duration};
use actix_web::{HttpRequest, HttpMessage, HttpResponse, Responder, delete, post, web};
use chrono::Utc;
use deadpool_redis::{Pool, redis::AsyncTypedCommands};
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    authentication::AuthContext, 
    deposit::{DepositClose, DepositRequest}, 
    io::{DataKind, EventMessage, EventType, Service}, 
    service_job::ServiceJob
};


#[post("/deposit")]
async fn create(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<DepositRequest>,
    pool: web::Data<Pool>
) -> impl Responder {
    let auth = request.extensions().get::<AuthContext>().cloned().unwrap();
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis Unavailable"),
    };
    
    let exists= match conn.exists::<_>(&auth.token).await {
        Ok(e) =>  e,
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis Unavailable")
    };

    if !exists {
        return HttpResponse::BadRequest().body("Not logged in");
    }

    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateDeposit{ deposit_request: api_obj.clone() },
            customer_id: auth.customer_id
        },
        from: Service::Api,
        to: Service::Deposit,
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
                EventType::Response { id:_, success, data, error_message, customer_id: _ } => {
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


#[delete("/deposit")]
async fn delete(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<DepositClose>
) -> impl Responder {
    let auth = request.extensions().get::<AuthContext>().cloned().unwrap();
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CloseDeposit { deposit_number: api_obj.deposit_number.clone() },
            customer_id: auth.customer_id
        },
        from: Service::Api,
        to: Service::Deposit,
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
                EventType::Response { id:_, success, data, error_message, customer_id: _ } => {
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