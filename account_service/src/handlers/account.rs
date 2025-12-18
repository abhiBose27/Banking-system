use tokio_postgres::Client;

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
    let mut success = false;
    let mut data = None;
    let mut error_message = None;
    let result_customer = get_customer(
        &client, 
        account_request.pan_id, 
        account_request.first_name, 
        account_request.last_name)
    .await;

    if let Err(e) = result_customer {
        eprintln!("Error: {e}");
        return (success, data, Some("Error: Invalid credentials".to_string()));
    }

    let customer = result_customer.unwrap();
    match add_account(client, customer.id).await {
        Ok(account) => {
            success = true;
            let account_api = AccountResponse {
                account_number: account.account_number,
                balance: account.balance,
                creation_timestamp: account.creation_timestamp,
            };
            data = Some(DataKind::CreateAccountResponse { account: account_api.clone() });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to create account".to_string());
        }
    }
    (success, data, error_message)
}

pub async fn get_account(client: &Client, account_number: String) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut error_message = None;
    let mut data = None;
    match get_account_db(client, account_number).await {
        Ok(account) => {
            success = true;
            data = Some(DataKind::GetAccountResponse { account: account.clone() })
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to get account".to_string())
        }
    }
    (success, data, error_message)
}

pub async fn update_balance(client: &Client, account_number: String, balance: f64) -> (bool, Option<DataKind>, Option<String>) {
    match update_balance_db(client, balance, account_number).await {
        Ok(_) => {
            (true, None, None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Cannot update balance".to_string()))
        },
    }
}