use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;
use tokio::sync::{mpsc::Sender, oneshot};
use actix_web::{HttpResponse, Responder, post, web};

use object::interfaces::{
    io::{DataKind, EventMessage, EventType, Service}, 
    service_job::ServiceJob, signin::SignInRequest, user::UserRequest
};


#[post("/signin")]
async fn client_signin(
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<SignInRequest>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            customer_id: None,
            data: DataKind::CreateUser { user_request: UserRequest { 
                username: api_obj.username.clone(), 
                password: api_obj.password.clone(), 
                customer_reference_id: api_obj.customer_reference_id.clone() }}
        }, 
        from: Service::Api, 
        to: Service::User, 
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
                EventType::Response { id:_, success, customer_id:_, error_message, data } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::CreateUserResponse => return HttpResponse::Ok().body("Added user"),
                                        _ => return HttpResponse::InternalServerError().body("Internal Server Error"),
                                    }
                                },
                                None => return HttpResponse::InternalServerError().body(error_message.unwrap())
                            }
                        },
                        false => return HttpResponse::InternalServerError().body(error_message.unwrap()),
                    }
                }
                _ => panic!("Error: Unknown object received on API: {:?}", response.clone())
            }
        }
        Ok(Err(_)) => return HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => return HttpResponse::RequestTimeout().body("Timed out")
    }
}