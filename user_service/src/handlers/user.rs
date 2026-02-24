use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

use object::interfaces::{authentication::Role, io::DataKind, service_job::ServiceJob, user::UserRequest};

use crate::{database::user::{add_user, get_user as get_user_db}, requests::customer::get_customer};

pub async fn create_user(
    client: &Client, 
    tx_dealer: &Sender<ServiceJob>, 
    user_request: UserRequest
) -> (bool, Option<DataKind>, Option<String>) {
    let mut role = Role::Admin;
    let mut customer_id = None;
    if let Some(id) = user_request.customer_reference_id {
        let customer_result = get_customer(tx_dealer, id).await;
        if let None = customer_result {
            return (false, None, Some("Error: Invalid customer reference id".to_string()));
        }
        role = Role::Client;
        customer_id = Some(customer_result.unwrap().id);
    }
    match add_user(client, role, user_request.username, user_request.password, customer_id).await {
        Ok(_) => (true, Some(DataKind::CreateUserResponse), None),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            (false, None, Some("Error: Cannot create user".to_string()))
        },
    }
}

fn verify_password(password: String, hash: String) -> bool {
    let parsed = PasswordHash::new(&hash).unwrap();
    let is_valid = Argon2::default().verify_password(password.as_bytes(), &parsed);
    is_valid.is_ok()
}

pub async fn get_user(
    client: &Client,
    username: String,
    password: String
) -> (bool, Option<DataKind>, Option<String>) {
    let user_result = get_user_db(client, username).await;
    if let Err(e) = user_result {
        eprintln!("Error: {:?}", e);
        return (false, None, Some("Error: Cannot get user".to_string()));
    }
    let user = user_result.unwrap();
    if !verify_password(password, user.password_hash.clone()) {
        return (false, None, Some("Error: Invalid password".to_string()));
    }
   /*  if !user.is_active {
        return (false, None, Some("Error: Invalid credentials".to_string()));
    } */
    (true, Some(DataKind::GetUserResponse { user }), None)
}