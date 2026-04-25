use object::interfaces::service_config::ServiceConfig;

use crate::interfaces::service::Service;

pub mod service;
pub mod interfaces;
pub mod database;
pub mod handlers;
pub mod requests;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    match envy::prefixed("DEPOSIT_SERVICE_").from_env::<ServiceConfig>() {
        Ok(config) => {
            let deposit_service = Service::new(config).await;
            deposit_service.worker().await.unwrap();
        },
        Err(e) => eprintln!("Error: {e}")
    }
    Ok(())
}
