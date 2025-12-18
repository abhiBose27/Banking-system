use chrono::Utc;
use tokio_postgres::{Client};
use uuid::Uuid;
use anyhow::Result;

use object::interfaces::account::Account;


fn generate_account_number() -> String {
    let id = Uuid::new_v4();
    let short = &id.to_string()[0..12]; // truncate to 12 chars
    return format!("ACC{}", short.replace("-", "").to_uppercase())
}

pub async fn add_account(client: &Client, customer_id: Uuid) -> Result<Account> {
    let account_id = Uuid::new_v4();
    let account_number = generate_account_number();
    let account = Account {
        id: account_id,
        customer_id,
        account_number,
        balance: 0.0,
        creation_timestamp: Utc::now(),
    };
    let result = client.execute("INSERT INTO account (
        id, customer_id, account_number, balance, creation_timestamp
    ) VALUES ($1, $2, $3, $4, $5)", 
    &[
        &account.id,
        &account.customer_id,
        &account.account_number,
        &account.balance,
        &account.creation_timestamp
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(account)
}

pub async fn get_account(client: &Client, account_number: String) -> Result<Account> {
    let row_result = client.query_one(
        "SELECT * FROM account WHERE account_number = $1",
        &[&account_number]
    ).await;
    if let Err(e) = row_result {
        return Err(e.into());
    }
    let row = row_result.unwrap();
    let account = Account {
        id: row.get("id"),
        customer_id: row.get("customer_id"),
        account_number: row.get("account_number"),
        balance: row.get("balance"),
        creation_timestamp: row.get("creation_timestamp"),
    };
    Ok(account)
}

pub async fn update_balance(
    client: &Client,
    balance: f64,
    account_number: String
) -> Result<()> {
    let result = client
        .execute("UPDATE account SET balance = $1 WHERE account_number = $2",
        &[&balance, &account_number]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(())
}