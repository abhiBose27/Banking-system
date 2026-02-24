use ulid::Ulid;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::interfaces::authentication::Role;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub customer_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
    pub customer_reference_id: Option<Ulid>,
}