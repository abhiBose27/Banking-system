use std::{collections::HashMap, time::Duration};
use actix_web::{HttpResponse, Responder, get, post, web};
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use ulid::Ulid;
use uuid::Uuid;

use object::interfaces::{
    customer::CustomerRequest, 
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob
};


#[get("/customer")]
async fn get_customer(
    tx: web::Data<Sender<ServiceJob>>,
    query: web::Query<HashMap<String, String>>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(),
            session_customer_id: None,
            data: DataKind::GetCustomer { 
                customer_reference_id: match query.get("customer_reference_id") {
                    Some(query) => match Ulid::from_string(query) {
                        Ok(id) => Some(id),
                        Err(e) => {
                            eprintln!("Error: {e}");
                            return HttpResponse::BadRequest().body("Error: Invalid customer reference Id")
                        }
                    },
                    None => return HttpResponse::BadRequest().body("Error: Invalid Parameter")
                }, 
            }
        },
        from: ServiceType::Api,
        to: ServiceType::Account,
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
                                        DataKind::GetCustomerResponse { customer } => return HttpResponse::Ok().body(serde_json::to_string(&customer).unwrap()),
                                        _ => {
                                            eprintln!("Error: Invalid response received for getting customer");
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

#[post("/customer")]
async fn create_customer(
    tx: web::Data<Sender<ServiceJob>>,
    payload: web::Json<CustomerRequest>
) -> impl Responder {
    // Send the request to create an account
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request {
            id: Uuid::new_v4(),
            data: DataKind::CreateCustomer { customer_request: payload.clone() }, 
            session_customer_id: None 
        },
        from: ServiceType::Api,
        to: ServiceType::Account,
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
                EventType::Response { id: _, success, error_message, data, session_customer_id: _ } => {
                   match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CreateCustomerResponse { customer } => return HttpResponse::Ok().body(serde_json::to_string(&customer).unwrap()),
                                        _ => {
                                            eprintln!("Error: Invalid response received for creating customer");
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