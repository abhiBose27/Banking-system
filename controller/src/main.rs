pub mod controller;
pub mod interfaces;

use object::interfaces::controller_config::ControllerConfig;

use crate::interfaces::controller::Controller;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok(); 
    match envy::prefixed("CONTROLLER_").from_env::<ControllerConfig>() {
        Ok(config)=> {
            let controller = Controller::new(config).await;
            controller.worker().await.unwrap();
        }
        Err(e) => eprintln!("Error: {e}"),
    };
    Ok(())
}