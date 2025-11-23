use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

use crate::interfaces::transaction::TransactionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementRequest {
    pub account_number: String,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub date: NaiveDate,
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
    pub transaction_type: TransactionType,
}