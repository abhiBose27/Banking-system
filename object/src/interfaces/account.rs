use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRequest {
    pub customer_reference_id: Ulid
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
pub struct AccountDetail {
    pub account_number: String,
    pub balance: f64,
    pub creation_timestamp: DateTime<Utc>
}