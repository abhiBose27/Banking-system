use argon2::{
    Argon2, PasswordHasher, 
    password_hash::{SaltString, rand_core::OsRng
}};
use tokio_postgres::Client;
use anyhow::Result;
use uuid::Uuid;

use object::interfaces::{user::User};


fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
    hash
}

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
        customer_id: row.get("customer_id")
    })
}

pub async fn add_user(
    client: &Client,
    username: String,
    password: String,
    customer_id: Uuid,
) -> Result<()> {
    let user = User {
        id: Uuid::new_v4(),
        customer_id,
        username: username,
        password_hash: hash_password(password.clone()),
    };
    let result = client.execute(
        "INSERT INTO db_user (
            id, username, password_hash, customer_id
        ) VALUES ($1, $2, $3, $4)", 
        &[
        &user.id,
        &user.username,
        &user.password_hash,
        &user.customer_id,
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(())
}
