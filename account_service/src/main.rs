pub mod service;
pub mod database;
pub mod handlers;
pub mod interface;

use object::interfaces::{service_config::ServiceConfig};

use crate::interface::service::Service;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok(); 
    match envy::prefixed("ACCOUNT_SERVICE_").from_env::<ServiceConfig>() {
        Ok(config)=> {
            let account_service = Service::new(config).await;
            account_service.worker().await.unwrap();
        }
        Err(e) => eprintln!("Error: {e}"),
    };
    Ok(())
}