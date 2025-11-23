use chrono::Utc;
use uuid::Uuid;
use tokio_postgres::{Client, NoTls, connect};

use object::interfaces::{dealer::Dealer, io::{DataKind, EventMessage, EventType, Service}, ports::Ports::{self, ControllerRoute}, service_config::ServiceConfig};

use crate::{handlers::{statement::get_statement, transaction::make_transaction}, interfaces::dealer::DealerService};

impl DealerService {

    async fn connect_to_db(service_config: ServiceConfig) -> Client {
        let config_str = format!(
            "host={0} user={1} password={2} dbname={3}",                    
            service_config.db_host,
            service_config.db_user,
            service_config.db_password,
            service_config.db_database
        );
        let (client, connection) = match connect(&config_str, NoTls).await {
            Ok((client, connection)) => (client, connection),
            Err(e) => panic!("Error: Cannot connect to DB: {e}"),
        };
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("Error: Postgresql connection error: {e}");
            }
        });
        client
    }

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
            from: Service::Transaction,
            to: Service::Controller,
            timestamp: Utc::now(),
        };
        if !dealer.send_event(event_message).await {
            panic!("Error: Cannot register to the Controller");
        }
        dealer
    }

    async fn resolve_request(client: &Client, dealer: &mut Dealer, data: DataKind, request_id: Uuid) -> EventType {
        let (executed, data, error_message) = match data {
            DataKind::CreateTransaction { transaction_request } => {
                make_transaction(&client, dealer, transaction_request).await
            },
            DataKind::GetStatement { statement_request } => {
                get_statement(&client, statement_request).await
            }
            _  => panic!("Error: Invalid request received {ControllerRoute}")
        };
        EventType::Response { 
            id: request_id, 
            executed, 
            error_message, 
            data 
        }
    }

    pub async fn new(service_config: ServiceConfig) -> Self {
        let client = Self::connect_to_db(service_config).await;
        let dealer = Self::connect(ControllerRoute).await;
        Self {
            dealer,
            client
        }
    }

    pub async fn worker(self) -> anyhow::Result<()> {
        println!("Starting Transaction Service");
        let mut dealer = self.dealer;
        let client = self.client;
        loop {
            if let Some(event_message) = dealer.recv_event(None).await {
                println!("Received request: {:?}", event_message);
                if let EventType::Request { id, data } = event_message.data {
                   let response_message = EventMessage {
                        data: Self::resolve_request(&client, &mut dealer, data, id).await,
                        from: event_message.to,
                        to: event_message.from,
                        timestamp: Utc::now()
                    };
                    let is_sent = dealer.send_event(response_message.clone()).await;
                    if !is_sent {
                        eprintln!("Error: Cant send response {:?}", response_message)
                    }
                }
            }
            /* if let Ok(message) = dealer_socket_clone.recv().await {
                let message_clone = message.clone();
                let raw_message = message_clone.get(0).unwrap();
                let event_message = serde_json::from_slice::<EventMessage>(raw_message).unwrap();
                println!("Received request: {:?}", event_message);
                if let EventType::Request { id, data } = event_message.data {
                    let response_message = EventMessage {
                        data: Self::resolve_request(&client, &mut dealer_socket_clone, data, id).await,
                        from: event_message.to,
                        to: event_message.from,
                        timestamp: Utc::now()
                    };
                    let payload = serde_json::to_vec(&response_message).unwrap();
                    if let Err(e) = dealer_socket_clone.send(payload.into()).await {
                        eprintln!("Error: Cant send response {:?}: {e}", response_message)
                    }
                }
            } */
        }
    }
}