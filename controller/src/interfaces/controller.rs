use std::{collections::HashMap, sync::Arc};
use bytes::Bytes;
use tokio::sync::Mutex;
use zeromq::RouterSocket;

use object::interfaces::io::ServiceType;

pub struct Controller {
    pub service_to_identity: Arc<Mutex<HashMap<ServiceType, Bytes>>>,
    pub router: Arc<Mutex<RouterSocket>>,
}