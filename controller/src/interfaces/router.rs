use std::{collections::HashMap, sync::Arc};
use bytes::Bytes;
use object::interfaces::io::{EventMessage, Service};
use tokio::sync::{Mutex, mpsc::{Receiver, Sender}};
use zeromq::RouterSocket;


pub struct Router {
    pub service_to_identity: Arc<Mutex<HashMap<Service, Bytes>>>,
    pub router_socket: Arc<Mutex<RouterSocket>>,
    pub tx_queue: Arc<Mutex<Sender<EventMessage>>>,
    pub rx_queue: Arc<Mutex<Receiver<EventMessage>>>
}