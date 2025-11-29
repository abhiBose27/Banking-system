use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub pan_id: String,
    pub email_id: String,
    pub age: i64,
    pub date_of_birth: NaiveDate,
    pub contact_number: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub customer_reference_id: Ulid,
    pub first_name: String,
    pub last_name: String,
    pub pan_id: String, // Always use lowercase to compare
    pub email_id: String,
    pub age: i64,
    pub date_of_birth: NaiveDate,
    pub contact_number: String,
    pub creation_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub customer_reference_id: Ulid,
    pub first_name: String,
    pub last_name: String,
    pub pan_id: String, // Always use lowercase to compare
    pub email_id: String,
    pub age: i64,
    pub date_of_birth: NaiveDate,
    pub contact_number: String,
}