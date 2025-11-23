use anyhow::Result;
use bytes::Bytes;
use std::{collections::HashMap, sync::Arc};
use tokio::{select, sync::{Mutex, mpsc}};
use zeromq::{RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};
use object::interfaces::{io::{EventMessage, EventType, Service}, ports::Ports::{self, ControllerRoute}};

use crate::interfaces::router::Router;


impl Router {
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

    async fn register_services(self) -> Result<()> {
        let mut services_to_register = vec![
            Service::Api, 
            Service::Account, 
            Service::Transaction
        ];
        println!("Services to register: {:?}", services_to_register);
        while !services_to_register.is_empty() {
            let mut listener_socket_clone = self.router_socket.lock().await;
            let mut service_to_identity_clone = self.service_to_identity.lock().await;
            if let Ok(message) = listener_socket_clone.recv().await {
                let (sender_id, event_message) = Self::fetch_socket_message(message);
                match event_message.data {
                    EventType::Ping => {
                        let service_idx = services_to_register.iter().position(|x| *x == event_message.from);
                        if let Some(idx) = service_idx {
                            let service = services_to_register.remove(idx);
                            println!("Registering the service {:?}", service);
                            service_to_identity_clone.insert(service, sender_id);
                        }
                    },
                    _ => eprintln!("Error: Invalid data before registration")
                }
            }
            println!("Registered services {:?}", service_to_identity_clone);
        }
        Ok(())
    }

    pub async fn new() -> Self {
        let service_to_identity = HashMap::new();
        let router_socket = Self::bind(ControllerRoute).await;
        let (tx_queue, rx_queue) = mpsc::channel::<EventMessage>(128);
        Self {
            service_to_identity: Arc::new(Mutex::new(service_to_identity)),
            router_socket: Arc::new(Mutex::new(router_socket)),
            tx_queue: Arc::new(Mutex::new(tx_queue)),
            rx_queue: Arc::new(Mutex::new(rx_queue))
        }
    }

    pub async fn worker(self) -> Result<()> {
        println!("Starting the service");
        let tx_queue = self.tx_queue.clone();
        let rx_queue = self.rx_queue.clone();
        let router_socket2 = self.router_socket.clone();
        let service_to_identity2 = self.service_to_identity.clone();
        let _ = self.register_services().await;

        loop {
            let mut rx_queue = rx_queue.lock().await;
            let tx_queue = tx_queue.lock().await;
            let mut router_socket_clone = router_socket2.lock().await;
            let service_to_identity = service_to_identity2.lock().await;
            select! {
                // Receive messages
                Ok(message) = router_socket_clone.recv() => {
                    let (sender_id, event_message) = Self::fetch_socket_message(message);
                    println!("Received {:?}", event_message);
                    
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

                    match event_message.data {
                        EventType::Ping => print!(""),
                        _ => tx_queue.send(event_message).await.unwrap()
                    }
                }

                // Route the message
                Some(message) = rx_queue.recv() => {
                    let to_service = message.clone().to;
                    let service_id = service_to_identity.get(&to_service).unwrap();
                    let zmq_message = Self::prepare_payload(message.clone(), service_id.clone());
                    println!("Routing message to id: {:?}", service_id.clone());
                    
                    if let Err(e) = router_socket_clone.send(zmq_message).await {
                        eprintln!("Error: Cant send response {:?}: {e}", message)
                    }
                }
            }
        }
    }
}