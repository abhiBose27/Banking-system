use tokio_postgres::Client;
use uuid::Uuid;

use object::interfaces::{
    account::{AccountResponse, AccountRequest}, 
    io::DataKind
};

use crate::database::{
    account::{
        add_account, update_balance as update_balance_db, 
        get_account as get_account_db
    }, 
    customer::get_customer
};


pub async fn create_account(client: &Client, account_request: AccountRequest) -> (bool, Option<DataKind>, Option<String>) {
    let result_customer = get_customer(&client, account_request.customer_reference_id).await;

    if let Err(e) = result_customer {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Invalid credentials".to_string()));
    }

    let customer = result_customer.unwrap();
    match add_account(client, customer.id).await {
        Ok(account) => {
            let data = Some(DataKind::CreateAccountResponse { 
                account: AccountResponse {
                    account_number: account.account_number,
                    creation_timestamp: account.creation_timestamp,
                }
            });
            (true, data, None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to create account".to_string()))
        }
    }
}

pub async fn get_account(client: &Client, account_number: String, customer_id: Option<Uuid>) -> (bool, Option<DataKind>, Option<String>) {
    match get_account_db(client, account_number).await {
        Ok(account) => {
            if customer_id.is_some() && account.customer_id != customer_id.unwrap() {
                (false, None, Some("Error: Wrong customer id for account".to_string()))
            }
            else {
                (true, Some(DataKind::GetAccountResponse { account: account.clone() }), None)
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to get account".to_string()))
        }
    }
}

pub async fn update_balance(client: &Client, account_number: String, balance: f64, customer_id: Option<Uuid>) -> (bool, Option<DataKind>, Option<String>) {
    let account_result = get_account_db(client, account_number.clone()).await;
    if let Err(e) = account_result {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Failed to get account".to_string()));
    }
    let account = account_result.unwrap();
    if customer_id.is_some() && account.customer_id != customer_id.unwrap() {
        return (false, None, Some("Error: Wrong customer id for account".to_string()));
    }
    match update_balance_db(client, balance, account_number).await {
        Ok(_) => (true, None, None),
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Cannot update balance".to_string()))
        },
    }
}