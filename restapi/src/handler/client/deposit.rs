use std::{time::Duration};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, web};
use chrono::Utc;
use deadpool_redis::Pool;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    authentication::AuthContext, 
    deposit::{DepositRequest}, 
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob
};

use crate::cache::redis::is_logged_in_with_token;


#[get("/deposits")]
async fn get_deposits(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
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

    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::GetDepositsDetail { customer_reference_id: None }, 
            session_customer_id
        },
        from: ServiceType::Api,
        to: ServiceType::Deposit,
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
            match response {
                EventType::Response { id: _, success, session_customer_id: _, error_message, data } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::GetDepositsDetailResponse { deposits_detail } => return HttpResponse::Ok().body(serde_json::to_string(&deposits_detail).unwrap()),
                                        _ => {
                                            eprintln!("Error: Invalid response received for getting deposits");
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

#[post("/deposit")]
async fn create_deposit(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    payload: web::Json<DepositRequest>,
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

    // Send the request to create a deposit
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateDeposit{ deposit_request: payload.clone() },
            session_customer_id
        },
        from: ServiceType::Api,
        to: ServiceType::Deposit,
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
            match response {
                EventType::Response { id:_, success, data, error_message, session_customer_id: _ } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CreateDepositResponse { deposit_detail } => return HttpResponse::Ok().body(serde_json::to_string(&deposit_detail).unwrap()),
                                        _ => {
                                            eprintln!("Error: Invalid response received for creating deposit");
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

#[delete("/deposit/{deposit_number}")]
async fn close_deposit(
    request: HttpRequest,
    tx: web::Data<Sender<ServiceJob>>,
    path: web::Path<String>,
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

    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(),
            session_customer_id,
            data: DataKind::CloseDeposit { deposit_number: path.into_inner() },
        },
        from: ServiceType::Api,
        to: ServiceType::Deposit,
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
            match response {
                EventType::Response { id:_, success, data, error_message, session_customer_id: _ } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CloseDepositResponse => return HttpResponse::Ok().body("Closed deposit"),
                                        _ => {
                                            eprintln!("Error: Invalid response received for closing deposit");
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