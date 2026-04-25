pub mod handler;
pub mod interfaces;
pub mod service;
pub mod authentication;
pub mod cache;

use dotenvy::dotenv;
use object::interfaces::api_config::ApiConfig;

use crate::interfaces::service::Service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    match envy::prefixed("API_SERVICE_").from_env::<ApiConfig>() {
        Ok(config) => {
            let api_service = Service::new(config).await;
            api_service.worker().await.unwrap();
        },
        Err(e) => eprintln!("Error: {e}"),
    };
    Ok(())
}
