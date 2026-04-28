use chrono::{DateTime, Utc};
use tokio_postgres::Client;
use uuid::Uuid;
use anyhow::Result;

use object::interfaces::deposit::{Deposit, DepositRequest, DepositStatus};

use crate::tools::tools::{get_interest_rate, get_maturity_timestamp, get_next_interest_timestamp, get_total_interest_amount};


fn generate_account_number() -> String {
    let id = Uuid::new_v4();
    let short = &id.to_string()[0..16]; // truncate to 12 chars
    return format!("ACC{}", short.replace("-", "").to_uppercase())
}

pub async fn close_deposit(
    client: &Client,
    deposit_id: Uuid
) -> Result<()> {
    let result = client
        .execute("UPDATE deposit_account SET status = $1 WHERE id = $2", 
        &[
            &serde_json::to_string(&DepositStatus::Matured).unwrap(),
            &deposit_id
        ]
    ).await;
    if let Err(e) = result {
        return  Err(e.into());
    }
    Ok(())
}

pub async fn get_deposit_from_deposit_number(
    client: &Client,
    deposit_number: String
) -> Result<Deposit> {
    let row_result = client.query_one(
        "SELECT * FROM deposit_account WHERE deposit_number = $1 AND status = '\"Active\"'", 
        &[&deposit_number]
    ).await;
    if let Err(e) = row_result {
        return Err(e.into());
    }
    let row = row_result.unwrap();
    let deposit = Deposit {
        id: row.get("id"),
        status: serde_json::from_str(row.get("status")).unwrap(),
        customer_id: row.get("customer_id"),
        deposit_number,
        linked_account_number: row.get("linked_account_number"),
        principal_amount: row.get("principal_amount"),
        interest_rate: row.get("interest_rate"),
        deposit_tenure: serde_json::from_value(row.get("deposit_tenure")).unwrap(),
        interest_payout: serde_json::from_str(row.get("interest_payout")).unwrap(),
        creation_timestamp: row.get("creation_timestamp"),
        next_interest_date: row.get("next_interest_date"),
        maturity_date: row.get("maturity_date"),
        auto_renewal: row.get("auto_renewal"),
        renewed_deposit_tenure: match serde_json::from_value(row.get("deposit_tenure")) {
            Ok(v) => Some(v),
            Err(_) => None,
        },
        total_interest_amount: row.get("total_interest_amount"),
        total_interest_paid: row.get("total_interest_paid"),
    };  
    Ok(deposit)
}

pub async fn get_deposits_from_customer_id(
    client: &Client,
    customer_id: Uuid
) -> Result<Vec<Deposit>> {
     let rows_result = client.query(
        "SELECT * FROM deposit_account WHERE customer_id = $1 AND status = '\"Active\"'", 
        &[&customer_id]
    ).await;
    if let Err(e) = rows_result {
        return Err(e.into());
    }
    let rows = rows_result.unwrap();
    Ok(rows.iter().map(|row| {
        Deposit {
            id: row.get("id"),
            status: serde_json::from_str(row.get("status")).unwrap(),
            customer_id: row.get("customer_id"),
            deposit_number: row.get("deposit_number"),
            linked_account_number: row.get("linked_account_number"),
            principal_amount: row.get("principal_amount"),
            interest_rate: row.get("interest_rate"),
            deposit_tenure: serde_json::from_value(row.get("deposit_tenure")).unwrap(),
            interest_payout: serde_json::from_str(row.get("interest_payout")).unwrap(),
            creation_timestamp: row.get("creation_timestamp"),
            next_interest_date: row.get("next_interest_date"),
            maturity_date: row.get("maturity_date"),
            auto_renewal: row.get("auto_renewal"),
            renewed_deposit_tenure: match serde_json::from_value(row.get("deposit_tenure")) {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            total_interest_amount: row.get("total_interest_amount"),
            total_interest_paid: row.get("total_interest_paid"),
        }
    }).collect::<Vec<Deposit>>())
}

pub async fn update_deposit(
    client: &Client, 
    deposit_id: Uuid,
    total_interest_paid: f64,
    next_interest_timestamp: Option<DateTime<Utc>>
) -> Result<()> {
    let next_interest_date = next_interest_timestamp.map(|d| d.date_naive());
    let result = client
        .execute("UPDATE deposit_account SET next_interest_date = $1, total_interest_paid = $2 WHERE id = $3",
        &[&next_interest_date, &total_interest_paid, &deposit_id]
    ).await;
    if let Err(e) = result {
        return  Err(e.into());
    }
    Ok(())
}

pub async fn add_deposit(client: &Client, customer_id: Uuid, deposit_request: DepositRequest) -> Result<Deposit> {
    let deposit_id = Uuid::new_v4();
    let creation_timestamp = Utc::now();
    let deposit_tenure = deposit_request.deposit_tenure;
    let interest_payout = deposit_request.interest_payout;
    let renewed_deposit_tenure = deposit_request.renewed_deposit_tenure;
    let interest_rate = get_interest_rate(deposit_tenure.clone());
    let maturity_timestamp = get_maturity_timestamp(creation_timestamp, deposit_tenure.clone());
    let total_interest_amount = get_total_interest_amount(deposit_request.principal_amount, interest_rate, deposit_tenure.clone());
    let next_interest_timestamp = get_next_interest_timestamp(creation_timestamp, maturity_timestamp, interest_payout.clone());
    let deposit = Deposit {
        id: deposit_id,
        customer_id,
        deposit_number: generate_account_number(),
        linked_account_number: deposit_request.linked_account_number,
        principal_amount: deposit_request.principal_amount,
        interest_rate,
        interest_payout,
        auto_renewal: deposit_request.auto_renewal,
        creation_timestamp,
        status: DepositStatus::Active,
        deposit_tenure,
        renewed_deposit_tenure,
        next_interest_date: next_interest_timestamp.map(|d| d.date_naive()),
        maturity_date: maturity_timestamp.date_naive(),
        total_interest_amount,
        total_interest_paid: 0.0,
    };

    let result = client.execute("INSERT INTO deposit_account (
        id,
        status,
        customer_id, 
        deposit_number, 
        linked_account_number, 
        principal_amount,
        interest_rate,
        deposit_tenure,
        interest_payout,
        total_interest_amount,
        total_interest_paid,
        auto_renewal,
        renewed_deposit_tenure,
        creation_timestamp,
        maturity_date,
        next_interest_date
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)", 
    &[
        &deposit.id,
        &serde_json::to_string(&deposit.status).unwrap(),
        &deposit.customer_id,
        &deposit.deposit_number,
        &deposit.linked_account_number,
        &deposit.principal_amount,
        &deposit.interest_rate,
        &serde_json::to_value(&deposit.deposit_tenure).unwrap(),
        &serde_json::to_string(&deposit.interest_payout).unwrap(),
        &deposit.total_interest_amount,
        &deposit.total_interest_paid,
        &deposit.auto_renewal,
        &serde_json::to_value(&deposit.renewed_deposit_tenure).unwrap(),
        &deposit.creation_timestamp,
        &deposit.maturity_date,
        &deposit.next_interest_date
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(deposit)
}