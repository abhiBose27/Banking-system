use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub pan_id: String,
    pub email_id: String,
    pub age: u64,
    pub date_of_birth: NaiveDate,
    pub contact_number: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer<'a> {
    pub id: Uuid,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub pan_id: &'a str, // Always use lowercase to compare
    pub email_id: &'a str,
    pub age: u64,
    pub date_of_birth: NaiveDate,
    pub contact_number: &'a str,
    pub creation_timestamp: DateTime<Utc>,
}