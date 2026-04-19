use ulid::Ulid;
use uuid::Uuid;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub customer_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
    pub customer_reference_id: Ulid,
}