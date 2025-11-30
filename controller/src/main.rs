pub mod router;
pub mod interfaces;

use crate::interfaces::router::RouterService;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let router = RouterService::new().await;
    router.worker().await.unwrap();
    Ok(())
}