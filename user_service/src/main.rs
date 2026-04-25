use object::interfaces::service_config::ServiceConfig;

use crate::interfaces::service::Service;

pub mod interfaces;
pub mod handlers;
pub mod service;
pub mod database;
pub mod requests;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok(); 
    match envy::prefixed("USER_SERVICE_").from_env::<ServiceConfig>() {
        Ok(config)=> {
            let user_service = Service::new(config).await;
            user_service.worker().await.unwrap();
        }
        Err(e) => eprintln!("Error: {e}"),
    };
    Ok(())
}
