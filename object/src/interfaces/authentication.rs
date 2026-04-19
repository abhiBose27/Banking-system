use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Client,
    Admin,
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: String,
    pub role: Role,
    pub token: String,
    pub customer_id: Uuid,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub client_secret: Vec<u8>,
    pub issuer: String,
    pub client_aud: String,
}

#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    pub private_key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub customer_id: Uuid,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}