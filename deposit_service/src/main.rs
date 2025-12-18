use object::interfaces::service_config::ServiceConfig;

use crate::interfaces::dealer::DealerService;

pub mod dealer;
pub mod interfaces;
pub mod database;
pub mod handlers;
pub mod requests;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    match envy::prefixed("DEPOSIT_SERVICE_").from_env::<ServiceConfig>() {
        Ok(config) => {
            let dealer_service = DealerService::new(config).await;
            dealer_service.worker().await.unwrap();
        },
        Err(e) => eprintln!("Error: {e}")
    }
    Ok(())
}
