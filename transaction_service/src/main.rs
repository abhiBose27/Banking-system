use object::interfaces::service_config::ServiceConfig;

use crate::interfaces::service::Service;

pub mod database;
pub mod interfaces;
pub mod service;
pub mod handlers;
pub mod requests;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    match envy::prefixed("TRANSACTION_SERVICE_").from_env::<ServiceConfig>() {
        Ok(config) => {
            let transaction_service = Service::new(config).await;
            transaction_service.worker().await.unwrap();
        },
        Err(e) => eprintln!("Error: {e}"),
    }
    Ok(())
}