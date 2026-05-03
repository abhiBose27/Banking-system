use uuid::Uuid;
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::{select, sync::mpsc::{self, Sender}};
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_postgres::{Client, NoTls, connect};

use object::interfaces::{
    dealer::Dealer, 
    io::{DataKind, EventMessage, EventType, ServiceType}, 
    service_config::ServiceConfig, service_job::ServiceJob
};

use crate::{
    handlers::{deposit::{close_deposit, create_deposit, get_deposits}, 
    interest::process_interests, maturity::process_maturity}, 
    interfaces::service::Service
};


impl Service {
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

    async fn connect(service_config: ServiceConfig) -> Dealer {
        let mut dealer = Dealer::new(
            service_config.host, 
            service_config.port
        ).await;
        if !dealer.connect().await {
            panic!("Error: Connection error with Controller");
        }
        let event_message = EventMessage {
            data: EventType::Ping,
            from: ServiceType::Deposit,
            to: ServiceType::Controller,
            timestamp: Utc::now(),
        };
        if !dealer.send_event(event_message).await {
            panic!("Error: Cannot register to the Controller");
        }
        dealer
    }

    async fn resolve_request(
        client: &Client, 
        tx_dealer: &Sender<ServiceJob>, 
        data: DataKind, 
        request_id: Uuid,
        session_customer_id: Option<Uuid>
    ) -> EventType {
        let (success, data, error_message) = match data {
            DataKind::CreateDeposit { deposit_request } => {
                create_deposit(client, tx_dealer, deposit_request, session_customer_id).await
            },
            DataKind::CloseDeposit { deposit_number } => {
                close_deposit(client, tx_dealer, deposit_number, session_customer_id).await
            },
            DataKind::GetDepositsDetail { customer_reference_id } => {
                get_deposits(client, tx_dealer, customer_reference_id, session_customer_id).await
            }
            _  => panic!("Error: Invalid request received on Deposit Service")
        };
        EventType::Response { 
            id: request_id, 
            success, 
            error_message, 
            data,
            session_customer_id 
        }
    }

    pub async fn new(service_config: ServiceConfig) -> Self {
        let client = Self::connect_to_db(service_config.clone()).await;
        let dealer = Self::connect(service_config).await;
        let id_to_tx_job = HashMap::new();
        let (tx_incoming, rx_incoming) = mpsc::channel::<ServiceJob>(128);
        let (tx_outgoing, rx_outgoing) = mpsc::channel::<ServiceJob>(128);
        Self {
            dealer,
            client: Arc::new(client),
            tx_incoming,
            rx_incoming,
            tx_outgoing,
            rx_outgoing,
            id_to_tx_job,
        }
    }

    pub async fn worker(self) -> anyhow::Result<()> {
        println!("Starting Deposit Service");
        let mut dealer = self.dealer;
        let mut id_to_tx_job = self.id_to_tx_job;
        let mut rx_outgoing = self.rx_outgoing;
        let mut rx_incoming = self.rx_incoming;
        let tx_outgoing1 = self.tx_outgoing.clone();
        let tx_outgoing2 = self.tx_outgoing.clone();
        let tx_outgoing3 = self.tx_outgoing.clone();
        let tx_incoming = self.tx_incoming;
        let client1 = self.client.clone();
        let client2 = self.client.clone();
        let client3 = self.client.clone();
        
        tokio::spawn(async move {
            let job_scheduler = JobScheduler::new().await.unwrap();
            let interest_job = Job::new_async("0 25 0 ? * *", {
                move |uuid, mut lock| {
                    let client = client2.clone();
                    let tx_outgoing = tx_outgoing2.clone();
                    Box::pin(async move {
                        process_interests(&client, &tx_outgoing).await;
                        let next_tick = lock.next_tick_for_job(uuid).await;
                        match next_tick {
                            Ok(Some(ts)) => println!("Next tick for interests at {:?}", ts),
                            _ => println!("Could not get next tick"),
                        }
                    })
                }
            }).unwrap();

            let maturity_job = Job::new_async("0 30 0 ? * *", { 
                move |uuid, mut lock| {
                    let client = client3.clone();
                    let tx_outgoing = tx_outgoing3.clone();
                    Box::pin(async move {
                        process_maturity(&client, &tx_outgoing).await;
                        let next_tick = lock.next_tick_for_job(uuid).await;
                        match next_tick {
                            Ok(Some(ts)) => println!("Next time for maturity tick at {:?}", ts),
                            _ => println!("Could not get next tick"),
                        }
                    })
                }
            }).unwrap();

            job_scheduler.add(interest_job).await.unwrap();
            job_scheduler.add(maturity_job).await.unwrap();
            job_scheduler.start().await.unwrap();
        });
        
        tokio::spawn(async move {
            loop {
                if let Some(message) = rx_incoming.recv().await {
                    let event_message = message.data;
                    println!("Received request {:?}", event_message);
                    if let EventType::Request { id, data, session_customer_id } = event_message.data {
                        let response_message = EventMessage {
                            data: Self::resolve_request(&client1, &tx_outgoing1, data, id, session_customer_id).await,
                            from: event_message.to,
                            to: event_message.from,
                            timestamp: Utc::now()
                        };
                        let service_job = ServiceJob { tx_job: None, data: response_message };
                        tx_outgoing1.send(service_job).await.unwrap();
                    }
                }
            }
        });
        loop {
            select! {
                Some(message) = rx_outgoing.recv() => {
                    let event_message = message.data;
                    if let EventType::Request { id, data:_, session_customer_id: _ } = event_message.data {
                        let tx_job = message.tx_job;
                        id_to_tx_job.insert(id, tx_job.unwrap());
                    }
                    if !dealer.send_event(event_message.clone()).await {
                        eprintln!("Error: Cant send message {:?}", event_message)
                    }
                }

                Some(message) = dealer.recv_event() => {
                    match message.data {
                        EventType::Request { id:_,data:_, session_customer_id: _ } => {
                            let service_job = ServiceJob { 
                                tx_job: None, 
                                data: message.clone() 
                            };
                            tx_incoming.send(service_job).await.unwrap();
                        },
                        EventType::Response { id, success:_,error_message:_,data:_, session_customer_id: _ } => {
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