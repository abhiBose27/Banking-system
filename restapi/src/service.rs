use chrono::Utc;
use actix_cors::Cors;
use std::{collections::HashMap};
use actix_web::{http::header::HeaderName, middleware, web, App, HttpServer};
use tokio::{select, sync::mpsc, task};
use deadpool_redis::{Config as RedisConfig, Runtime};


use object::interfaces::{
    api_config::ApiConfig, authentication::{ApiKeyConfig, JwtConfig}, 
    dealer::Dealer, io::{EventMessage, EventType, ServiceType}, 
    service_job::ServiceJob
};

use crate::{
    authentication::{client::authentication::client_auth, admin::authentication::admin_auth}, 
    handler, interfaces::service::Service
};


impl Service {
    
    async fn connect(api_config: ApiConfig) -> Dealer {
        let mut dealer = Dealer::new(
            api_config.host, 
            api_config.port,
        ).await;
        if !dealer.connect().await {
            panic!("Error: Connection error with Controller");
        }
        let event_message = EventMessage {
            data: EventType::Ping,
            from: ServiceType::Api,
            to: ServiceType::Controller,
            timestamp: Utc::now()
        };
        if !dealer.send_event(event_message).await {
            panic!("Error: Cannot register to the Controller");
        }
        dealer
    }

    pub async fn new(api_config: ApiConfig) -> Self {
        let dealer = Self::connect(api_config.clone()).await;
        let (tx_service, rx_service) = mpsc::channel::<ServiceJob>(128);
        let id_to_tx_job = HashMap::new();
        Self {
            dealer,
            tx_service,
            rx_service,
            id_to_tx_job,
            client_secret: api_config.client_jwt_secret,
            api_key: api_config.api_key,
            redis_host: api_config.redis_host,
            server_host: api_config.server_host,
            server_port: api_config.server_port,
            
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
                        if let EventType::Request { id, data:_, session_customer_id: _ } = event_message.data {
                            id_to_response_tx.insert(id, tx_job.unwrap());
                            println!("Sending request: {:?}", event_message);
                            if !dealer.send_event(event_message.clone()).await {
                                eprintln!("Error: Cant send message {:?}", event_message);
                            }
                        }
                    }

                    Some(event_message) = dealer.recv_event() => {
                        println!("Received response: {:?}", event_message.clone());
                        if let EventType::Response { id, data: _, success:_, error_message: _ , session_customer_id: _} = event_message.data {
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
            //admin_aud:"bank-admin".to_string(), 
            //admin_secret: "".as_bytes().to_vec()
        };

        let api_key_config = ApiKeyConfig { 
            private_key: self.api_key 
        };

        let redis_endpoint = format!("redis://{}/", self.redis_host);
        let server_endpoint = format!("{}:{}", self.server_host, self.server_port);
        let cfg = RedisConfig::from_url(redis_endpoint);
        let pool = cfg.create_pool(Some(Runtime::Tokio1)).expect("redis pool");

        HttpServer::new(move || {
            let x_total = HeaderName::from_lowercase(b"x-total").unwrap();
            let cors = Cors::permissive().expose_headers([x_total]);
            App::new()
                .app_data(web::Data::new(self.tx_service.clone()))
                .app_data(web::Data::new(jwt_cfg.clone()))
                .app_data(web::Data::new(api_key_config.clone()))
                .app_data(web::Data::new(pool.clone()))
                .wrap(cors)
                .wrap(middleware::DefaultHeaders::new().add(("X-Version", "0.1")))
                .wrap(middleware::Compress::default())
                .route("/", web::get().to(handler::handshake::handshake_handler))
                .service(
                    web::scope("/client/auth")
                        .service(handler::client::login::client_login)
                        .service(handler::client::signup::client_signup)
                )
                .service(
                    web::scope("/client/api")
                        .wrap(middleware::from_fn(client_auth))
                        .service(handler::client::logout::client_logout)
                        .service(handler::client::account::get_accounts)
                        .service(handler::client::account::get_account)
                        .service(handler::client::customer::get_customer)
                        .service(handler::client::deposit::get_deposit)
                        .service(handler::client::deposit::get_deposits)
                        .service(handler::client::deposit::create_deposit)
                        .service(handler::client::deposit::close_deposit)
                        .service(handler::client::transaction::create_transaction)
                        .service(handler::client::statement::get_statement)
                )
                .service(
                    web::scope("/admin/api")
                        .wrap(middleware::from_fn(admin_auth))
                        .service(handler::admin::account::create_account)
                        .service(handler::admin::customer::create_customer)
                        .service(handler::admin::account::get_accounts)
                        .service(handler::admin::account::get_account)
                        .service(handler::admin::customer::get_customer)
                        .service(handler::admin::deposit::get_deposit)
                        .service(handler::admin::deposit::get_deposits)
                        .service(handler::admin::deposit::create_deposit)
                        .service(handler::admin::deposit::close_deposit)
                        .service(handler::admin::transaction::create_transaction)
                        .service(handler::admin::statement::get_statement)
                )
        })
        .bind(server_endpoint)?
        .run()
        .await
    }
}