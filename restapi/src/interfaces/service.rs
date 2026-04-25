use std::{collections::HashMap};
use tokio::sync::{mpsc::{Receiver, Sender}, oneshot};
use uuid::Uuid;

use object::interfaces::{dealer::Dealer, io::EventType, service_job::ServiceJob};

pub struct Service {
    pub dealer: Dealer,
    pub client_secret: String,
    pub api_key: String,
    pub redis_host: String,
    pub server_host: String,
    pub server_port: String,
    pub tx_service: Sender<ServiceJob>,
    pub rx_service: Receiver<ServiceJob>,
    pub id_to_tx_job: HashMap<Uuid, oneshot::Sender<EventType>>
}