use std::collections::HashMap;
use chrono::Utc;
use tokio::{select, sync::mpsc::{self, Sender}};
use tokio_postgres::{Client, NoTls, connect};
use uuid::Uuid;

use object::interfaces::{
    dealer::Dealer, 
    io::{DataKind, EventMessage, EventType, Service}, 
    ports::Ports::{self, ControllerRoute}, 
    service_config::ServiceConfig, 
    service_job::ServiceJob
};

use crate::{handlers::user::{create_user, get_user}, interfaces::dealer::DealerService};

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
        let mut dealer = Dealer::new(
            "tcp://localhost".to_string(), 
            port
        ).await;
        if !dealer.connect().await {
            panic!("Error: Connection error with Controller");
        }
        let event_message = EventMessage {
            data: EventType::Ping,
            from: Service::User,
            to: Service::Controller,
            timestamp: Utc::now(),
        };
        if !dealer.send_event(event_message).await {
            panic!("Error: Cannot register to the Controller");
        }
        dealer
    }

    async fn resolve_request(client: &Client, tx_dealer: &Sender<ServiceJob>, data: DataKind, request_id: Uuid, customer_id: Option<Uuid>) -> EventType {
        let (success, data, error_message) = match data {
            DataKind::GetUser { username, password } => {
                get_user(client, username, password).await
            }
            DataKind::CreateUser { user_request } => {
                create_user(client, tx_dealer, user_request).await
            }
            _ => panic!("Error: Invalid request received {ControllerRoute}")
        };
        EventType::Response { 
            id: request_id, 
            success, 
            session_customer_id: customer_id, 
            error_message, 
            data 
        }
    }

    pub async fn new(service_config: ServiceConfig) -> Self {
        let client = Self::connect_to_db(service_config).await;
        let dealer = Self::connect(ControllerRoute).await;
        let id_to_tx_job: HashMap<Uuid, tokio::sync::oneshot::Sender<EventType>> = HashMap::new();
        let (tx_incoming, rx_incoming) = mpsc::channel::<ServiceJob>(128);
        let (tx_outgoing, rx_outgoing) = mpsc::channel::<ServiceJob>(128);
        Self {
            dealer,
            client,
            tx_incoming,
            rx_incoming,
            tx_outgoing,
            rx_outgoing,
            id_to_tx_job,
        }
    }

    pub async fn worker(self) -> anyhow::Result<()> {
        println!("Starting User Service");
        let client = self.client;
        let mut dealer = self.dealer;
        let mut id_to_tx_job = self.id_to_tx_job;
        let mut rx_outgoing = self.rx_outgoing;
        let mut rx_incoming = self.rx_incoming;
        let tx_outgoing = self.tx_outgoing;
        let tx_incoming = self.tx_incoming;

        tokio::spawn(async move {
            loop {
                if let Some(message) = rx_incoming.recv().await {
                    let event_message = message.data;
                    println!("Received request {:?}", event_message);
                    if let EventType::Request { id, data, session_customer_id } = event_message.data {
                        let response_message = EventMessage {
                            data: Self::resolve_request(&client, &tx_outgoing, data, id, session_customer_id).await,
                            from: event_message.to,
                            to: event_message.from,
                            timestamp: Utc::now(),
                        };
                        let service_job = ServiceJob { tx_job: None, data: response_message };
                        tx_outgoing.send(service_job).await.unwrap();
                    }
                }
            }
        });
        loop {
            select! {
                Some(message) = rx_outgoing.recv() => {
                    let event_message = message.data;
                    if let EventType::Request { id, session_customer_id: _, data: _ } = event_message.data {
                        let tx_job = message.tx_job;
                        id_to_tx_job.insert(id, tx_job.unwrap());
                    }
                    if !dealer.send_event(event_message.clone()).await {
                        eprintln!("Error: Cant send message {:?}", event_message)
                    }
                }
                Some(message) = dealer.recv_event() => {
                    match message.data {
                        EventType::Request { id:_, session_customer_id:_, data:_ } => {
                            let service_job = ServiceJob { 
                                tx_job: None, 
                                data: message
                            };
                            tx_incoming.send(service_job).await.unwrap();
                        },
                        EventType::Response { id, success:_, error_message:_, data:_, session_customer_id:_ } => {
                            let tx_job = id_to_tx_job.remove(&id).unwrap();
                            tx_job.send(message.data.clone()).unwrap();
                        },
                        _ => eprintln!("Error: Invalid message received")
                    }
                }
            }
        }
    }
}