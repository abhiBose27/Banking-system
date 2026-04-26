use serde::{Serialize, Deserialize};
use chrono::NaiveDate;
use ulid::Ulid;

use crate::interfaces::transaction::TransactionType;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementRequest {
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementResponse {
    pub date: NaiveDate,
    pub amount: f64,
    pub reference_id: Ulid,
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
    pub transaction_type: TransactionType,
}