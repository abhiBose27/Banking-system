use std::collections::HashMap;
use tokio::sync::{mpsc::{Receiver, Sender}, oneshot};
use uuid::Uuid;

use object::interfaces::{dealer::Dealer, io::{EventMessage, EventType}};

#[derive(Debug)]
pub struct ServiceJob {
    pub data: EventMessage,
    pub response_tx: oneshot::Sender<EventType>,
}

pub struct DealerService {
    pub dealer: Dealer,
    pub tx_controller: Sender<ServiceJob>,
    pub rx_controller: Receiver<ServiceJob>,
    pub id_to_response_tx: HashMap<Uuid, oneshot::Sender<EventType>>
}