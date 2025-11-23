use actix_web::{web, Responder};
use tokio::sync::mpsc::Sender;

use crate::interfaces::dealer::ServiceJob;

pub async fn handshake_handler(_data: web::Data<Sender<ServiceJob>>) -> impl Responder {
    format!("Running application on 0.0.0.0:3003")
}