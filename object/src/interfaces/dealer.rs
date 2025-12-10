use std::time::Duration;
use tokio::time::timeout;
use zeromq::{DealerSocket, Socket, SocketRecv, SocketSend};

use crate::interfaces::{io::EventMessage, ports::Ports};

pub struct Dealer {
    port: Ports,
    endpoint: String,
    socket: DealerSocket,
}

impl Dealer {
    pub async fn new(
        domain: String, 
        port: Ports,
    ) -> Self {
        let socket = DealerSocket::new();
        let endpoint = format!("{domain}:{port}");
        Dealer {
            port,
            endpoint,
            socket,
        }
    }

    pub async fn send_event(&mut self, event_message: EventMessage) -> bool {
        let payload = serde_json::to_vec(&event_message).unwrap();
        if let Err(e) = self.socket.send(payload.into()).await {
            eprintln!("Error: {e}");
            return false;
        }
        true
    }

    pub async fn recv_event(&mut self) -> Option<EventMessage> {
        if let Ok(message) = self.socket.recv().await {
            let message_clone = message.clone();
            let raw_message = message_clone.get(0).unwrap();
            let event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
            return Some(event_message);
        }
        None
    }

    pub async fn connect(&mut self) -> bool {
        let endpoint = self.endpoint.clone();
        let port = self.port.clone();
        match timeout(Duration::from_secs(2), self.socket.connect(&endpoint)).await {
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