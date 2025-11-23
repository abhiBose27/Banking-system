use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterestPayout {
    Monthly,
    Quaterly,
    Maturity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositTenure {
    pub year: u64,
    pub month: u64,
    pub days: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAccount<'a> {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub linked_account_id: Uuid,
    pub deposit_account_number: &'a str,
    pub principal_amount: f64,
    pub interest_rate: f64,
    pub deposit_tenure: DepositTenure,
    pub interest_payout: InterestPayout,
    pub auto_renewal: bool,
    pub renewed_deposit_tenure: Option<DepositTenure>,
    pub creation_timestamp: DateTime<Utc>,
    pub maturity_date: Option<DateTime<Utc>>
}