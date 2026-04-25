use std::time::Duration;
use actix_web::{HttpResponse, Responder, post, web};
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob, signin::SignInRequest, user::UserRequest
};


#[post("/signup")]
async fn client_signup(
    tx: web::Data<Sender<ServiceJob>>,
    payload: web::Json<SignInRequest>
) -> impl Responder {
    // Send a message to create a User
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            session_customer_id: None,
            data: DataKind::CreateUser { user_request: UserRequest { 
                username: payload.username.clone(),
                password: payload.password.clone(), 
                customer_reference_id: payload.customer_reference_id.clone() }}
        }, 
        from: ServiceType::Api, 
        to: ServiceType::User, 
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
                EventType::Response { id:_, success, session_customer_id:_, error_message, data } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CreateUserResponse => return HttpResponse::Ok().body("Added user"),
                                        _ => {
                                            eprintln!("Error: Invalid response received on create user endpoint");
                                            return HttpResponse::InternalServerError().finish();
                                        },
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
                        },
                    }
                }
                _ => {
                    eprintln!("Error: Unknown object received on API: {:?}", response.clone());
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
        Ok(Err(_)) => return HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => return HttpResponse::RequestTimeout().body("Timed out")
    }
}