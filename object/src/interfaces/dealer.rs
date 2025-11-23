use uuid::Uuid;
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::timeout};
use zeromq::{DealerSocket, Socket, SocketRecv, SocketSend};

use crate::interfaces::{io::{EventMessage, EventType}, ports::Ports};

#[derive(Clone)]
pub struct Dealer {
    port: Ports,
    endpoint: String,
    socket: Arc<Mutex<DealerSocket>>,
    buffer_recv_messages: VecDeque<EventMessage>
}

impl Dealer {
    pub async fn new(
        domain: String, 
        port: Ports,
    ) -> Self {
        let socket = DealerSocket::new();
        let endpoint = format!("{domain}:{port}");
        Dealer {
            socket: Arc::new(Mutex::new(socket)),
            endpoint,
            port,
            buffer_recv_messages: VecDeque::new()
        }
    }

    pub async fn send_event(&self, event_message: EventMessage) -> bool {
        let mut socket = self.socket.lock().await;
        let payload = serde_json::to_vec(&event_message).unwrap();
        if let Err(e) = socket.send(payload.into()).await {
            eprintln!("Error: {e}");
            return false;
        }
        true
    }

    pub async fn recv_event(&mut self, request_id: Option<Uuid>) -> Option<EventMessage> {
        let mut socket = self.socket.lock().await;

        // If no request_id is provided
        // Get the first message in the pipeline
        if request_id.is_none() {
            if !self.buffer_recv_messages.is_empty() {
                return self.buffer_recv_messages.pop_front();
            }
            if let Ok(message) = socket.recv().await {
                let message_clone = message.clone();
                let raw_message = message_clone.get(0).unwrap();
                let event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
                return Some(event_message);
            }
            return None;
        }

        // Find the message with that request_id
        let is_found= self.buffer_recv_messages.iter().position(|x| {
            if let EventType::Response { id, executed:_, error_message:_, data:_ } = x.data {
                if id == request_id.unwrap() {
                    return true;
                }
            }
            return false;
        });
        if let Some(position) = is_found {
            return self.buffer_recv_messages.remove(position);
        }
        
        // Wait for the message on the pipeline
        loop {
            match timeout(Duration::from_secs(5), socket.recv()).await {
                Ok(Ok(message)) => {
                    let message_clone = message.clone();
                    let raw_message = message_clone.get(0).unwrap();
                    let event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
                    if let EventType::Response { id, executed:_, error_message:_, data:_ } = event_message.data {
                        if id == request_id.unwrap() {
                            return Some(event_message);
                        }
                    }
                    self.buffer_recv_messages.push_back(event_message);
                },
                Ok(Err(e)) => panic!("Error: {e}"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    break;
                }
            }
        }
        None
    }

    pub async fn connect(&self) -> bool {
        let mut socket = self.socket.lock().await;
        let endpoint = self.endpoint.clone();
        let port = self.port.clone();
        match timeout(Duration::from_secs(2), socket.connect(&endpoint)).await {
            Ok(Ok(())) => {
                println!("Connected to port {port}");
                true
            }
            Ok(Err(e)) => {
                eprintln!("Error: {e}");
                false
            }
            Err(e) => {
                eprintln!("Error: {e}");
                false
            }
        }
    }
}