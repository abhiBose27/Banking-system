use std::collections::HashMap;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterestPayout {
    Daily,
    Monthly,
    Quaterly,
    Maturity,
    Renew
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepositStatus {
    Active,
    Matured
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositTenure {
    pub years: u32,
    pub months: u32,
    pub days: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    pub linked_account_number: String,
    pub principal_amount: f64,
    pub deposit_tenure: DepositTenure,
    pub interest_payout: InterestPayout,
    pub auto_renewal: bool,
    pub renewed_deposit_tenure: Option<DepositTenure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositClose {
    pub deposit_number: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub id: Uuid,
    pub status: DepositStatus,
    pub customer_id: Uuid,
    pub deposit_number: String,
    pub linked_account_number: String,
    pub principal_amount: f64,
    pub interest_rate: f64,
    pub deposit_tenure: DepositTenure,
    pub interest_payout: InterestPayout,
    pub interest_amount_to_frequency: HashMap<String, usize>,
    pub total_interest_paid: f64,
    pub creation_timestamp: DateTime<Utc>,
    pub next_interest_date: Option<NaiveDate>,
    pub maturity_date: NaiveDate,
    pub auto_renewal: bool,
    pub renewed_deposit_tenure: Option<DepositTenure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResponse {
    pub deposit_number: String,
    pub linked_account_number: String,
    pub principal_amount: f64,
    pub interest_rate: f64,
    pub deposit_tenure: DepositTenure,
    pub interest_payout: InterestPayout,
    pub auto_renewal: bool,
    pub renewed_deposit_tenure: Option<DepositTenure>,
    pub maturity_date: NaiveDate,
    pub creation_timestamp: DateTime<Utc>,
}
