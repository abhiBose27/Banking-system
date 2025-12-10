use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRequest {
    pub pan_id: String,
    pub first_name: String,
    pub last_name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub account_number: String,
    pub balance: f64,
    pub creation_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub account_number: String,
    pub balance: f64,
    pub creation_timestamp: DateTime<Utc>
}