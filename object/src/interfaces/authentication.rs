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
    pub customer_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub client_secret: Vec<u8>,
    pub admin_secret: Vec<u8>,
    pub issuer: String,
    pub client_aud: String,
    pub admin_aud: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub customer_id: Option<Uuid>,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}