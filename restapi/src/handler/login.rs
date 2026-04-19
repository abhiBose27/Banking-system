use std::{time::Duration};
use actix_web::{HttpResponse, Responder, post, web};
use chrono::Utc;
use deadpool_redis::Pool;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    authentication::{JwtConfig}, 
    io::{DataKind, EventMessage, EventType, Service}, 
    login::{LoginRequest, LoginResponse}, service_job::ServiceJob
};

use crate::authentication::{authentication::issue_jwt, redis::{is_logged_in_with_username, login_user}};


#[post("/login")]
async fn client_login(
    pool: web::Data<Pool>,
    jwt: web::Data<JwtConfig>,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<LoginRequest>
) -> impl Responder {
    // Get the connection from connection pool
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Redis error {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Check if the user is already logged in
    let is_logged_in = is_logged_in_with_username(&api_obj.username, &mut conn).await;
    if let Err(e) = is_logged_in {
        eprintln!("Error: Redis error {e}");
        return HttpResponse::InternalServerError().finish();
    }
    if let true = is_logged_in.unwrap() {
        return HttpResponse::BadRequest().body("Already logged in");
    }

    // Send the request to get the User
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage { 
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            session_customer_id: None,
            data: DataKind::GetUser { 
                username: api_obj.username.clone(), 
                password: api_obj.password.clone() 
            }
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

    let user = match tokio::time::timeout(Duration::from_secs(5), rx_job).await {
        Ok(Ok(response)) => {
            match response {
                EventType::Response { id:_, success, session_customer_id:_, error_message, data } => {
                    match success {
                        true => {
                            match data {
                                Some(d) => {
                                    match d {
                                        DataKind::GetUserResponse { user } => user,
                                        _ => {
                                            eprintln!("Error: Invalid response received for getting User");
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
                        }
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
    };

    let ttl_seconds = 5 * 60;
    let token = match issue_jwt(
        &jwt,
        ttl_seconds,
        user.id,
        user.customer_id
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if let Err(e) = login_user(&api_obj.username, &token, ttl_seconds, &mut conn).await {
        eprintln!("Error: Redis error {e}");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string()
    })
}