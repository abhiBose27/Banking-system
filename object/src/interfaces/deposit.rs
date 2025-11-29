use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterestPayout {
    Daily,
    Monthly,
    Quaterly,
    Maturity
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
    pub auto_renewal: bool,
    pub renewed_deposit_tenure: Option<DepositTenure>,
    pub creation_timestamp: DateTime<Utc>,
    pub maturity_date: NaiveDate,
    pub interest_cron: String,
    pub maturity_cron: String
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
    pub maturity_date: NaiveDate
}
