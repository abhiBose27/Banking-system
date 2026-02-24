use serde::{Deserialize, Serialize};
use ulid::Ulid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInRequest {
    pub customer_reference_id: Option<Ulid>,
    pub username: String,
    pub password: String
}