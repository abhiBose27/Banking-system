use std::time::Duration;
use actix_web::{HttpResponse, Responder, delete, post, web};
use chrono::Utc;
use tokio::sync::{mpsc::Sender, oneshot};
use uuid::Uuid;

use object::interfaces::{
    deposit::{DepositClose, DepositRequest}, 
    io::{DataKind, EventMessage, EventType, Service}, 
    service_job::ServiceJob
};

#[post("/deposit")]
async fn create(
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<DepositRequest>
) -> impl Responder {
    let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CreateDeposit{ deposit_request: api_obj.clone() }
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
                EventType::Response { id:_, success, data, error_message } => {
                    if !success {
                        HttpResponse::BadRequest().body(error_message.unwrap())
                    }
                    else {
                        let data= data.unwrap();
                        let body = serde_json::to_vec(&data).unwrap();
                        HttpResponse::Ok().body(body)
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
    tx: web::Data<Sender<ServiceJob>>,
    api_obj: web::Json<DepositClose>
) -> impl Responder {
     let (tx_job, rx_job) = oneshot::channel::<EventType>();
    let event_message = EventMessage {
        data: EventType::Request { 
            id: Uuid::new_v4(), 
            data: DataKind::CloseDeposit{ deposit_close: api_obj.clone() }
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
                EventType::Response { id:_, success, data, error_message } => {
                    if !success {
                        HttpResponse::BadRequest().body(error_message.unwrap())
                    }
                    else {
                        let data= data.unwrap();
                        let body = serde_json::to_vec(&data).unwrap();
                        HttpResponse::Ok().body(body)
                    }
                },
                _ => panic!("Error: Unknown object received on API: {:?}", response.clone())
            }
        },
        Ok(Err(_)) => HttpResponse::InternalServerError().body("Worker failed"),
        Err(_) => HttpResponse::RequestTimeout().body("Timed out"),
    }
    
}