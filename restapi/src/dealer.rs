use chrono::Utc;
use actix_cors::Cors;
use std::collections::HashMap;
use actix_web::{http::header::HeaderName, middleware, web, App, HttpServer};
use tokio::{select, sync::mpsc, task};

use object::interfaces::{
    dealer::Dealer, io::{EventMessage, EventType, Service}, ports::Ports::{self, ControllerRoute}
};

use crate::{handler, interfaces::dealer::{DealerService, ServiceJob}};


impl DealerService {
    async fn connect(port: Ports) -> Dealer {
        let dealer = Dealer::new(
            "tcp://localhost".to_string(), 
            port,
        ).await;
        if !dealer.connect().await {
            panic!("Error: Connection error with Controller");
        }
        let event_message = EventMessage {
            data: EventType::Ping,
            from: Service::Api,
            to: Service::Controller,
            timestamp: Utc::now()
        };
        if !dealer.send_event(event_message).await {
            panic!("Error: Cannot register to the Controller");
        }
        dealer
    }

    pub async fn new() -> Self {
        let dealer = Self::connect(ControllerRoute).await;
        let (tx_controller, rx_controller) = mpsc::channel::<ServiceJob>(128);
        let id_to_response_tx = HashMap::new();
        Self {
            dealer,
            tx_controller,
            rx_controller,
            id_to_response_tx
        }
    }

    pub async fn worker(self) -> anyhow::Result<()> {
        println!("Starting API Service");
        let _service_worker = task::spawn_blocking(move || self.http_server()).await.unwrap();
        Ok(())
    }

    #[actix_web::main]
    async fn http_server(self) -> std::io::Result<()> {
        task::spawn(async move {
            let mut rx_controller = self.rx_controller;
            let mut dealer = self.dealer;
            let mut id_to_response_tx = self.id_to_response_tx;
            loop {
                select! {
                    Some(request_job) = rx_controller.recv() => {
                        let event_message = request_job.data.clone();
                        let response_tx = request_job.response_tx;
                        if let EventType::Request { id, data:_ } = event_message.data {
                            id_to_response_tx.insert(id, response_tx);
                            println!("Sending request: {:?}", event_message);
                            if !dealer.send_event(event_message.clone()).await {
                                eprintln!("Error: Cant send message {:?}", event_message);
                            }
                        }
                    }

                    Some(event_message) = dealer.recv_event(None) => {
                        println!("Received client response: {:?}", event_message.clone());
                        if let EventType::Response { id, data: _, success:_, error_message: _ } = event_message.data {
                            let response_tx = id_to_response_tx.remove(&id).unwrap();
                            response_tx.send(event_message.data.clone()).unwrap();
                        }
                    }
                }
            }
            /* loop {
                select! {
                    // Messages from api endpoints.
                    // Distribute to controller
                    Some(message) = rx_controller.recv() => {
                        let event_message = message.data.clone();
                        println!("Sending client request: {:?}", event_message);
                        match event_message.data {
                            EventType::ClientRequest { id, data: _} => {
                                id_to_response_tx.insert(id, message.response_tx);
                                let payload = serde_json::to_vec(&event_message).unwrap();
                                if let Err(e) = controller_socket_clone.send(payload.into()).await {
                                    eprintln!("Error: Cannot send request {:?}: {}", event_message, e);
                                }
                            },
                            _ => eprintln!("Error: Invalid data to send {:?}", event_message)
                        }
                    }

                    // Received response from controller
                    Ok(message) = controller_socket_clone.recv() => {
                        let message_clone = message.clone();
                        let raw_message = message_clone.get(0).unwrap();
                        let event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
                        println!("Received client response: {:?}", event_message);
                        match event_message.data {
                            EventType::ClientResponse { id, data:_, response_msg } => {
                                let response_tx = id_to_response_tx.remove(&id).unwrap();
                                response_tx.send(response_msg.clone()).unwrap();
                            },
                            _ => eprintln!("Error: Invalid response message {:?}", event_message)
                        }
                    }

                    // Ping
                    _ = sleep(Duration::from_secs(5)) => {
                        let event_message = EventMessage { 
                            data: EventType::Ping, 
                            timestamp: Utc::now()
                        };
                        let payload = serde_json::to_vec(&event_message).unwrap();
                        if let Err(e) = controller_socket_clone.send(payload.into()).await {
                            eprintln!("Error: Cannot send PING messages: {}", e);
                            controller_socket_clone = Self::connect(APIRoute).await;
                        }
                    }
                }
            } */
        });

        HttpServer::new(move || {
            let x_total = HeaderName::from_lowercase(b"x-total").unwrap();
            let cors = Cors::permissive().expose_headers([x_total]);
            App::new()
                .app_data(web::Data::new(self.tx_controller.clone()))
                .wrap(cors)
                .wrap(middleware::DefaultHeaders::new().add(("X-Version", "0.1")))
                .wrap(middleware::Compress::default())
                .route("/", web::get().to(handler::handshake::handshake_handler))
                .service(handler::account::create)
                .service(handler::customer::create)
                .service(handler::transaction::create)
                .service(handler::deposit::create)
                .service(handler::statement::get)
        })
        .bind(("0.0.0.0", 3003))?
        .run()
        .await
    }
}