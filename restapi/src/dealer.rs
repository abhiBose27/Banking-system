use chrono::Utc;
use actix_cors::Cors;
use std::{collections::HashMap};
use actix_web::{http::header::HeaderName, middleware, web, App, HttpServer};
use tokio::{select, sync::mpsc, task};
use deadpool_redis::{Config as RedisConfig, Runtime};


use object::interfaces::{
    api_config::ApiConfig, authentication::JwtConfig, 
    dealer::Dealer, io::{EventMessage, EventType, Service}, 
    ports::Ports::{self, ControllerRoute}, service_job::ServiceJob
};

use crate::{authentication::authentication::{internal_auth}, handler, interfaces::dealer::DealerService};


impl DealerService {
    
    async fn connect(port: Ports) -> Dealer {
        let mut dealer = Dealer::new(
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

    pub async fn new(api_config: ApiConfig) -> Self {
        let dealer = Self::connect(ControllerRoute).await;
        let (tx_service, rx_service) = mpsc::channel::<ServiceJob>(128);
        let id_to_tx_job = HashMap::new();
        Self {
            dealer,
            tx_service,
            rx_service,
            id_to_tx_job,
            client_secret: api_config.client_jwt_secret,
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
            let mut dealer = self.dealer;
            let mut rx_controller = self.rx_service;
            let mut id_to_response_tx = self.id_to_tx_job;
            loop {
                select! {
                    Some(request_job) = rx_controller.recv() => {
                        let event_message = request_job.data.clone();
                        let tx_job = request_job.tx_job;
                        if let EventType::Request { id, data:_, customer_id: _ } = event_message.data {
                            id_to_response_tx.insert(id, tx_job.unwrap());
                            println!("Sending request: {:?}", event_message);
                            if !dealer.send_event(event_message.clone()).await {
                                eprintln!("Error: Cant send message {:?}", event_message);
                            }
                        }
                    }

                    Some(event_message) = dealer.recv_event() => {
                        println!("Received response: {:?}", event_message.clone());
                        if let EventType::Response { id, data: _, success:_, error_message: _ , customer_id: _} = event_message.data {
                            let response_tx = id_to_response_tx.remove(&id).unwrap();
                            response_tx.send(event_message.data.clone()).unwrap();
                        }
                    }
                }
            }
        });

        let jwt_cfg = JwtConfig {
            client_secret: self.client_secret.into_bytes(),
            issuer: "bank-auth".to_string(),
            client_aud: "bank-clients".to_string(),
            admin_aud:"bank-admin".to_string(), 
            admin_secret: "".as_bytes().to_vec()
        };

        let cfg = RedisConfig::from_url("redis://127.0.0.1/");
        let pool = cfg.create_pool(Some(Runtime::Tokio1)).expect("redis pool");

        HttpServer::new(move || {
            let x_total = HeaderName::from_lowercase(b"x-total").unwrap();
            let cors = Cors::permissive().expose_headers([x_total]);
            App::new()
                .app_data(web::Data::new(self.tx_service.clone()))
                .app_data(web::Data::new(jwt_cfg.clone()))
                .app_data(web::Data::new(pool.clone()))
                .wrap(cors)
                .wrap(middleware::DefaultHeaders::new().add(("X-Version", "0.1")))
                .wrap(middleware::Compress::default())
                .route("/", web::get().to(handler::handshake::handshake_handler))
                .service(
                    web::scope("/auth")
                        .service(handler::login::client_login)
                        .service(handler::signin::client_signin)
                )
                .service(
                    web::scope("/api")
                        .wrap(middleware::from_fn(internal_auth))
                        .service(handler::logout::client_logout)
                        .service(handler::account::create)
                        .service(handler::customer::create)
                        .service(handler::transaction::create)
                        .service(handler::deposit::create)
                        .service(handler::deposit::delete)
                        .service(handler::statement::get)

                )
        })
        .bind(("0.0.0.0", 3003))?
        .run()
        .await
    }
}