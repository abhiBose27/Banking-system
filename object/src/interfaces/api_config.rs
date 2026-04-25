use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub name: String,
    pub environment: String,
    pub host: String,
    pub port: String,
    pub redis_host: String,
    pub server_host: String,
    pub server_port: String,
    pub client_jwt_secret: String,
    pub api_key: String
}