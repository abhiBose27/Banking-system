use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub environment: String,
    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
    pub db_database: String
}