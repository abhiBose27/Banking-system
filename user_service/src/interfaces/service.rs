use std::collections::HashMap;
use tokio::sync::{mpsc::{Receiver, Sender}, oneshot};
use tokio_postgres::Client;
use uuid::Uuid;

use object::interfaces::{dealer::Dealer, io::EventType, service_job::ServiceJob};

pub struct Service {
    pub dealer: Dealer,
    pub client: Client,
    pub tx_incoming: Sender<ServiceJob>,
    pub rx_incoming: Receiver<ServiceJob>,
    pub tx_outgoing: Sender<ServiceJob>,
    pub rx_outgoing: Receiver<ServiceJob>,
    pub id_to_tx_job: HashMap<Uuid, oneshot::Sender<EventType>>
}