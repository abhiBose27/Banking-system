pub mod handler;
pub mod interfaces;
pub mod dealer;
pub mod authentication;

use std::env;
use anyhow::Ok;
use dotenvy::dotenv;

use crate::interfaces::dealer::DealerService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Optional: fail fast if missing
    env::var("INTERNAL_SERVICE_SECRET")
        .expect("INTERNAL_SERVICE_SECRET must be set");

    let dealer_service = DealerService::new().await;
    dealer_service.worker().await.unwrap();
    Ok(())
}
