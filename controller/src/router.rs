use anyhow::Result;
use bytes::Bytes;
use tokio::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use zeromq::{RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

use object::interfaces::{io::{EventMessage, EventType, Service}, ports::Ports::{self, ControllerRoute}};

use crate::interfaces::router::RouterService;


impl RouterService {
    async fn bind(port: Ports) -> RouterSocket {
        let mut api_listener_socket = RouterSocket::new();
        let endpoint = format!("tcp://localhost:{port}");
        match api_listener_socket.bind(&endpoint).await {
            Ok(_) => {
                println!("Listening to requests {port}");
                api_listener_socket
            },
            Err(e) => panic!("Error: Cannot listen to {port}: {e}"),
        }
    }

    fn fetch_socket_message(message: ZmqMessage) -> (Bytes, EventMessage) {
        let raw_message = message.get(1).unwrap();
        let sender_id = message.get(0).unwrap();
        let event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
        return (sender_id.clone(), event_message);
    }

    fn prepare_payload(event_message: EventMessage, service_id: Bytes) -> ZmqMessage {
        let mut zmq_message = ZmqMessage::from(service_id.clone());
        let raw_message = serde_json::to_vec(&event_message).unwrap();
        zmq_message.push_back(raw_message.into());
        return zmq_message;
    }

    async fn register_services(&self) -> Result<()> {
        let mut services_to_register = vec![
            Service::Api, 
            Service::Account, 
            Service::Transaction,
            Service::Deposit
        ];
        println!("Services to register: {:?}", services_to_register);
        while !services_to_register.is_empty() {
            let mut router = self.router.lock().await;
            if let Ok(message) = router.recv().await {
                let (sender_id, event_message) = Self::fetch_socket_message(message);
                match event_message.data {
                    EventType::Ping => {
                        let service_idx = services_to_register.iter().position(|x| *x == event_message.from);
                        if let Some(idx) = service_idx {
                            let service = services_to_register.remove(idx);
                            let mut services_to_identity = self.service_to_identity.lock().await;
                            println!("Registering the service {:?}", service);
                            services_to_identity.insert(service, sender_id);
                        }
                    },
                    _ => eprintln!("Error: Invalid data before registration")
                }
            }
        }
        let services_to_identity = self.service_to_identity.lock().await;
        println!("Registered services {:?}", services_to_identity);
        Ok(())
    }

    pub async fn new() -> Self {
        let service_to_identity = HashMap::new();
        let router = Self::bind(ControllerRoute).await;
        Self {
            service_to_identity: Arc::new(Mutex::new(service_to_identity)),
            router: Arc::new(Mutex::new(router)),
        }
    }

    pub async fn worker(self) -> Result<()> {
        println!("Starting the Controller");
        let _ = self.register_services().await;
        loop {
            let mut router = self.router.lock().await;
            loop {
                if let Ok(message) = router.recv().await {
                    let (sender_id, event_message) = Self::fetch_socket_message(message);
                    println!("Received {:?}", event_message);
                    
                    let service_to_identity = self.service_to_identity.lock().await;
                    let is_service_from_registered = service_to_identity.get(&event_message.from);
                    let is_service_to_registered = service_to_identity.get(&event_message.to);
                    if let None = is_service_from_registered {
                        eprintln!("Error: Invalid 'from' microservice. Not registered");
                        continue;
                    }
                    if let None = is_service_to_registered {
                        eprintln!("Error: Invalid 'to' microservice. Not registered");
                        continue;
                    }
                    if sender_id != is_service_from_registered.unwrap() {
                        eprintln!("Error: Invalid microservice. Wrong sender id");
                        continue;
                    }

                    let to_service = event_message.clone().to;
                    let service_id = service_to_identity.get(&to_service).unwrap();
                    let zmq_message = Self::prepare_payload(event_message.clone(), service_id.clone());
                    println!("Routing message to id: {:?}", service_id.clone());
                    
                    if let Err(e) = router.send(zmq_message).await {
                        eprintln!("Error: Cant send response {:?}: {e}", event_message)
                    }
                }
            }
        }
    }
}