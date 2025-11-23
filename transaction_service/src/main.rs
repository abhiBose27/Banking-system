use object::interfaces::service_config::ServiceConfig;

use crate::interfaces::dealer::DealerService;

pub mod database;
pub mod interfaces;
pub mod dealer;
pub mod handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    match envy::prefixed("TRANSACTION_SERVICE_").from_env::<ServiceConfig>() {
        Ok(config) => {
            let dealer_service = DealerService::new(config).await;
            dealer_service.worker().await.unwrap();
        },
        Err(e) => eprintln!("Error: {e}"),
    }
    Ok(())
}