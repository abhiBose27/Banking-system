use std::{time::Duration};
use actix_web::{HttpResponse, Responder, delete, get, post, web};
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use ulid::Ulid;
use uuid::Uuid;

use object::interfaces::{
    deposit::{DepositRequest}, 
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob
};


#[get("/deposits/{customer_reference_id}")]
async fn get_deposits(
    tx: web::Data<Sender<ServiceJob>>,
    path: web::Path<String>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            session_customer_id: None, 
            data: DataKind::GetDepositsDetail { 
                customer_reference_id: match Ulid::from_string(&path.into_inner()) {
                    Ok(id) => Some(id),
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return HttpResponse::BadRequest().body("Error: Invalid customer reference Id")
                    },
                }
            },
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
    tx: web::Data<Sender<ServiceJob>>,
    payload: web::Json<DepositRequest>,
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateDeposit{ deposit_request: payload.clone() },
            session_customer_id: None
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
    tx: web::Data<Sender<ServiceJob>>,
    path: web::Path<String>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(),
            session_customer_id: None,
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