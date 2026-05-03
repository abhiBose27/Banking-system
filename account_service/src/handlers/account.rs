use tokio_postgres::Client;
use ulid::Ulid;
use uuid::Uuid;

use object::interfaces::{
    account::{AccountDetail, AccountRequest}, 
    io::DataKind
};

use crate::database::{
    account::{
        add_account, get_account_from_account_number, get_accounts_from_customer_id, update_balance as update_balance_db
    }, 
    customer::{get_customer_from_customer_id, get_customer_from_customer_reference_id}
};


pub async fn create_account(client: &Client, account_request: AccountRequest) -> (bool, Option<DataKind>, Option<String>) {
    let result_customer = get_customer_from_customer_reference_id(&client, account_request.customer_reference_id).await;
    if let Err(e) = result_customer {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Invalid customer reference id".to_string()));
    }

    let customer = result_customer.unwrap();
    match add_account(client, customer.id).await {
        Ok(account) => {
            let data = Some(DataKind::CreateAccountResponse { 
                account_detail: AccountDetail {
                    account_number: account.account_number,
                    creation_timestamp: account.creation_timestamp,
                    balance: account.balance,
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

pub async fn get_account(
    client: &Client, 
    account_number: String, 
    session_customer_id: Option<Uuid>, 
    is_detail: bool
) -> (bool, Option<DataKind>, Option<String>) {
    match get_account_from_account_number(client, account_number.clone()).await {
        Ok(account) => {
            if session_customer_id.is_some() && account.customer_id != session_customer_id.unwrap() {
                return (false, None, Some("Error: Invalid session customer id".to_string()))
            }
            if is_detail {
                let account_detail = AccountDetail {
                    account_number,
                    balance: account.balance,
                    creation_timestamp: account.creation_timestamp,
                };
                return (true, Some(DataKind::GetAccountDetailResponse { account_detail }), None) ;  
            }
            return (true, Some(DataKind::GetAccountResponse { account }), None);
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to get account".to_string()))
        }
    }
} 

pub async fn get_accounts(
    client: &Client, 
    customer_reference_id: Option<Ulid>, 
    session_customer_id: Option<Uuid>
) -> (bool, Option<DataKind>, Option<String>) {
    let mut customer_id = None;
    if let Some(cust_id) = session_customer_id {
        if let Err(e) = get_customer_from_customer_id(client, cust_id).await {
            eprintln!("Error: {e}");
            return (false, None, Some("Error: Invalid session customer id".to_string()));
        }
        customer_id = Some(cust_id);
    }
    if let Some(cust_reference_id) = customer_reference_id {
        let customer_result = get_customer_from_customer_reference_id(client, cust_reference_id).await;
        if let Err(e) = customer_result {
            eprintln!("Error: {e}");
            return (false, None, Some("Error: Invalid customer reference id".to_string()))
        }
        customer_id = Some(customer_result.unwrap().id);
    }
    if let Some(cust_id) = customer_id {
        match get_accounts_from_customer_id(client, cust_id).await {
            Ok(accounts) => {
                let accounts_detail = accounts.iter().map(|account| AccountDetail {
                    account_number: account.account_number.clone(),
                    balance: account.balance,
                    creation_timestamp: account.creation_timestamp,
                }).collect::<Vec<AccountDetail>>();
                return (true, Some(DataKind::GetAccountsDetailResponse { accounts_detail }), None);
            }
            Err(e) => {
                eprint!("Error {e}");
                return (false, None, Some("Error: Failed to get accounts".to_string()));
            }
        };
    }
    (false, None, Some("Error: Parameters not provided".to_string()))
}

pub async fn update_balance(client: &Client, account_number: String, balance: f64, session_customer_id: Option<Uuid>) -> (bool, Option<DataKind>, Option<String>) {
    let account_result = get_account_from_account_number(client, account_number.clone()).await;
    if let Err(e) = account_result {
        eprintln!("Error: {e}");
        return (false, None, Some("Error: Failed to get account".to_string()));
    }
    let account = account_result.unwrap();
    if session_customer_id.is_some() && account.customer_id != session_customer_id.unwrap() {
        return (false, None, Some("Error: Invalid session customer id".to_string()));
    }
    match update_balance_db(client, balance, account_number).await {
        Ok(_) => (true, Some(DataKind::UpdateBalanceResponse), None),
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Cannot update balance".to_string()))
        },
    }
}