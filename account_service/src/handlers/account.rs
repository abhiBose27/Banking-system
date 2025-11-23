use object::interfaces::{account::AccountRequest, io::{DataKind}, transaction::TransactionRequest};
use tokio_postgres::Client;

use crate::database::{account::{add_account, update_balance as update_balance_db}, customer::get_customer_id_from_pan_id};

pub async fn create_account(client: &Client, account_request: AccountRequest) -> (bool, Option<DataKind>, Option<String>) {
    let mut executed = false;
    let mut data = None;
    let result_customer_id = get_customer_id_from_pan_id(&client, account_request.pan_id, account_request.first_name, account_request.last_name).await;
    if let Err(e) = result_customer_id {
        eprintln!("Error: {e}");
        return (executed, data, Some("Error: Invalid credentials".to_string()));
    }

    let customer_id = result_customer_id.unwrap();
    let mut error_message = None;
    match add_account(client, customer_id).await {
        Ok(account_number) => {
            executed = true;
            data = Some(DataKind::Account { account_number: account_number.to_string() });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to create account".to_string());
        }
    }
    (executed, data, error_message)
}

pub async fn update_balance(client: &Client, transaction_request: TransactionRequest) -> (bool, Option<DataKind>, Option<String>) {
    let mut executed = false;
    let mut error_message = None;
    match update_balance_db(client, transaction_request).await {
        Ok(_) => executed = true,
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to update balance".to_string());
        },
    }
    (executed, None, error_message)
}