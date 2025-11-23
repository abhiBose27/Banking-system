pub mod router;
pub mod interfaces;

use crate::interfaces::router::Router;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let router = Router::new().await;
    router.worker().await.unwrap();
    Ok(())
}