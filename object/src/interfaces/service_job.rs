
use tokio::sync::oneshot::Sender;

use crate::interfaces::io::{EventMessage, EventType};


#[derive(Debug)]
pub struct ServiceJob {
    pub tx_job: Option<Sender<EventType>>,
    pub data: EventMessage
}