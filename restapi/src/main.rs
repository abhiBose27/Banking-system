pub mod handler;
pub mod interfaces;
pub mod dealer;

use anyhow::Ok;

use crate::interfaces::dealer::DealerService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dealer_service = DealerService::new().await;
    dealer_service.worker().await.unwrap();
    Ok(())
}
