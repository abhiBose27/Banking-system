use argon2::{
    Argon2, PasswordHasher, 
    password_hash::{SaltString, rand_core::OsRng
}};
use tokio_postgres::Client;
use anyhow::Result;
use uuid::Uuid;

use object::interfaces::{authentication::Role, user::User};

pub async fn get_user(client: &Client, username: String) -> Result<User> {
    let row_result = client.query_one(
        "SELECT * FROM db_user WHERE username = $1", &[&username]).await;
    if let Err(e) = row_result {
        return Err(e.into());
    }
    let row = row_result.unwrap();
    Ok(User { 
        id: row.get("id"), 
        username: row.get("username"),
        password_hash: row.get("password_hash"), 
        role: serde_json::from_str(row.get("role")).unwrap(),
        customer_id: row.get("customer_id")
    })
}

fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
    hash
}

pub async fn add_user(
    client: &Client,
    role: Role,
    username: String,
    password: String,
    customer_id: Option<Uuid>,
) -> Result<()> {
    let user = User {
        id: Uuid::new_v4(),
        role,
        customer_id,
        username: username,
        password_hash: hash_password(password.clone()),
    };
    let result = client.execute(
        "INSERT INTO db_user (
            id, username, password_hash, role, customer_id
        ) VALUES ($1, $2, $3, $4, $5)", 
        &[
        &user.id,
        &user.username,
        &user.password_hash,
        &serde_json::to_string(&user.role).unwrap(),
        &user.customer_id,
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(())
}
