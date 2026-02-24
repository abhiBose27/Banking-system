pub mod handler;
pub mod interfaces;
pub mod dealer;
pub mod authentication;

use dotenvy::dotenv;
use object::interfaces::api_config::ApiConfig;

use crate::interfaces::dealer::DealerService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    match envy::prefixed("API_SERVICE_").from_env::<ApiConfig>() {
        Ok(config) => {
            let dealer_service = DealerService::new(config).await;
            dealer_service.worker().await.unwrap();
        },
        Err(e) => eprintln!("Error: {e}"),
    };
    Ok(())
}
