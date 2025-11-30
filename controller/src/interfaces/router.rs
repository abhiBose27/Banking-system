use std::{collections::HashMap, sync::Arc};
use bytes::Bytes;
use tokio::sync::Mutex;
use zeromq::RouterSocket;

use object::interfaces::io::Service;

pub struct RouterService {
    pub service_to_identity: Arc<Mutex<HashMap<Service, Bytes>>>,
    pub router: Arc<Mutex<RouterSocket>>,
}