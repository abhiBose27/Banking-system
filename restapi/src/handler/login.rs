use std::{time::Duration};
use actix_web::{HttpResponse, Responder, post, web};
use chrono::Utc;
use deadpool_redis::{Pool, redis::{AsyncTypedCommands}};
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    authentication::{JwtConfig, Role}, 
    io::{DataKind, EventMessage, EventType, Service}, 
    login::{LoginRequest, LoginResponse}, service_job::ServiceJob
};

use crate::{authentication::{authentication::issue_jwt}};


#[post("/login")]
async fn client_login(
    jwt: web::Data<JwtConfig>,
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<LoginRequest>,
    pool: web::Data<Pool>
) -> impl Responder {

    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis Unavailable"),
    };
    let exists= match conn.exists::<_>(&api_obj.username).await {
        Ok(e) =>  e,
        Err(_) => return HttpResponse::InternalServerError().body("Error: Redis Unavailable")
    };

    if exists {
        return HttpResponse::BadRequest().body("Already logged in");
    }

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

    let ttl_seconds = match login_user.role {
        Role::Client => 5 * 60,  // 5 min
        Role::Admin => 15 * 60, // 15 min
    };
    let token = match issue_jwt(
        &jwt,
        login_user.role,
        login_user.id,
        ttl_seconds,
        login_user.customer_id
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if let Err(_) = conn.set_ex::<_, _>(&api_obj.username, &token, ttl_seconds as u64).await {
        return HttpResponse::InternalServerError().body("Error: Redis set failed");
    }

    if let Err(_) = conn.set_ex::<_, _>(&token, &api_obj.username, ttl_seconds as u64).await {
        return HttpResponse::InternalServerError().body("Error: Redis set failed");
    }

    HttpResponse::Ok().json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string()
    })
}