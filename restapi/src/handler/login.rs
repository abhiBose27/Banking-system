use std::{time::Duration};
use actix_web::{HttpResponse, Responder, post, web};
use chrono::Utc;
use object::interfaces::{authentication::{JwtConfig, Role}, io::{DataKind, EventMessage, EventType, Service}, login::{LoginRequest, LoginResponse}, service_job::ServiceJob};
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use crate::{authentication::{authentication::issue_jwt}};


#[post("/login")]
async fn client_login(
    jwt: web::Data<JwtConfig>,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<LoginRequest>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            customer_id: None,
            data: DataKind::GetUser { username: api_obj.username.clone(), password: api_obj.password.clone() }
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

    let login_user = match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response)) => {
            match response {
                EventType::Response { id:_, success, customer_id:_, error_message, data } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::GetUserResponse { user } => user,
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
    };

    let token = match issue_jwt(
        &jwt,
        &login_user.id.to_string(),
        if login_user.role == Role::Client {"client"} else {"admin"},
        login_user.customer_id
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string()
    })
}