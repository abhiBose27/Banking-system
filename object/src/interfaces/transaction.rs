use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {  
    Credit,
    Debit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub amount: f64,
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub amount: f64,
    pub reference_id: Ulid,
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
    pub transaction_timestamp: DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub reference_id: Ulid,
    pub amount: f64,
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
    pub transaction_timestamp: DateTime<Utc>
}