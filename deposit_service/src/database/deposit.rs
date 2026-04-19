use std::collections::HashMap;

use chrono::{DateTime, Duration, Months, Utc};
use tokio_postgres::Client;
use uuid::Uuid;
use anyhow::Result;

use object::interfaces::deposit::{Deposit, DepositRequest, DepositStatus, DepositTenure, InterestPayout};

fn generate_account_number() -> String {
    let id = Uuid::new_v4();
    let short = &id.to_string()[0..16]; // truncate to 12 chars
    return format!("ACC{}", short.replace("-", "").to_uppercase())
}

fn get_maturity_timestamp(current_timestamp: DateTime<Utc>, deposit_tenure: DepositTenure) -> DateTime<Utc> {
    let mut maturity_timestamp = current_timestamp;
    if deposit_tenure.years != 0 {
        maturity_timestamp = maturity_timestamp + Months::new((deposit_tenure.years * 12) as u32);
    }
    if deposit_tenure.months != 0 {
        maturity_timestamp = maturity_timestamp + Months::new(deposit_tenure.months as u32);
    }
    if deposit_tenure.days != 0 {
        maturity_timestamp = maturity_timestamp + Duration::days(deposit_tenure.days as i64);
    }
    maturity_timestamp
}

pub fn get_next_interest_timestamp(
    current_timestamp: DateTime<Utc>,
    maturity_timestamp: DateTime<Utc>, 
    interest_payout: InterestPayout
) -> Option<DateTime<Utc>> {
    if current_timestamp == maturity_timestamp {
        return Some(maturity_timestamp);
    }
    let interest_timestamp = current_timestamp;
    match interest_payout {
        InterestPayout::Daily => {
            if interest_timestamp + Duration::days(1 as i64) > maturity_timestamp {Some(maturity_timestamp)}
            else {Some(interest_timestamp + Duration::days(1 as i64))}
        },
        InterestPayout::Monthly => {
            if interest_timestamp + Months::new(1 as u32) > maturity_timestamp {Some(maturity_timestamp)}
            else {Some(interest_timestamp + Months::new(1 as u32))}
        },
        InterestPayout::Quaterly => {
            if interest_timestamp + Months::new(3 as u32) > maturity_timestamp {Some(maturity_timestamp)}
            else {Some(interest_timestamp + Months::new(3 as u32))}
        },
        InterestPayout::Maturity => Some(maturity_timestamp),
        InterestPayout::Renew => None,
    }
}

fn get_interest_amount_to_frequency(
    principal_amount: f64,
    interest_rate: f64,
    interest_payout: InterestPayout,
    deposit_tenure: Option<DepositTenure>
) -> HashMap<String, usize> {
    let mut hashmap = HashMap::new();
    if let None = deposit_tenure {
        return hashmap;
    }
    let d = deposit_tenure.unwrap();
    let total_days = d.days + d.months * 30 + d.years * 365;
    let annual_interest_amount = principal_amount * (interest_rate / 100.0);
    match interest_payout {
        InterestPayout::Daily => {
            let daily_interest_amount = annual_interest_amount / 365.0;
            let key = format!("{:.2}", daily_interest_amount);
            hashmap.insert(key, total_days.try_into().unwrap());
        },
        InterestPayout::Monthly => {
            let total_months = total_days / 30;
            let leftover_days = total_days % 30;
            let monthly_interest_amount = annual_interest_amount / 12.0;
            let leftover_interest_amount = annual_interest_amount * (leftover_days as f64 / 365.0);
            hashmap.insert(format!("{:.2}", monthly_interest_amount), total_months.try_into().unwrap());
            hashmap.insert(format!("{:.2}", leftover_interest_amount), 1);
        },
        InterestPayout::Quaterly => {
            let total_quaters = total_days / 91;
            let leftover_days = total_days % 91;
            let quaterly_interest_amount = annual_interest_amount / 4.0;
            let leftover_interest_amount = annual_interest_amount * (leftover_days as f64 / 365.0);
            hashmap.insert(format!("{:.2}", quaterly_interest_amount), total_quaters.try_into().unwrap());
            hashmap.insert(format!("{:.2}", leftover_interest_amount), 1);

        },
        _ => {
            let total_quaters = total_days / 91;
            let leftover_days = total_days % 91;
            let quarter_compound_interest = principal_amount * (1.0 + interest_rate / 4.0).powf(total_quaters.into());
            let simple_interest = quarter_compound_interest * leftover_days as f64 * (interest_rate / 100.0) / 365.0;
            let maturity_interest = quarter_compound_interest + simple_interest - principal_amount as f64;
            hashmap.insert(format!("{:.2}", maturity_interest), 1);
        },
    };
    hashmap
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
        "SELECT * FROM deposit_account WHERE deposit_number = $1", 
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
        interest_amount_to_frequency: serde_json::from_value(row.get("interest_amount_to_frequency")).unwrap(),
        total_interest_paid: row.get("total_interest_paid"),
        renewed_deposit_tenure: match serde_json::from_value(row.get("deposit_tenure")) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    };  
    Ok(deposit)
}

pub async fn get_deposits_from_customer_id(
    client: &Client,
    customer_id: Uuid
) -> Result<Vec<Deposit>> {
     let rows_result = client.query(
        "SELECT * FROM deposit_account WHERE customer_id = $1", 
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
            interest_amount_to_frequency: serde_json::from_value(row.get("interest_amount_to_frequency")).unwrap(),
            total_interest_paid: row.get("total_interest_paid"),
            renewed_deposit_tenure: match serde_json::from_value(row.get("deposit_tenure")) {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        }
    }).collect::<Vec<Deposit>>())
}

pub async fn update_deposit(
    client: &Client, 
    deposit_id: Uuid,
    interest_amount: f64,
    total_interest_paid: f64,
    next_interest_timestamp: Option<DateTime<Utc>>
) -> Result<()> {
    let next_interest_date = next_interest_timestamp.map(|d| d.date_naive());
    let interest_amount_str = format!("{:.2}", interest_amount);
    let result = client
        .execute("UPDATE deposit_account SET next_interest_date = $1, total_interest_paid = $2 WHERE id = $3",
        &[&next_interest_date, &total_interest_paid, &deposit_id]
    ).await;
    if let Err(e) = result {
        return  Err(e.into());
    }
    let result = client
        .execute(
            "UPDATE deposit_account
            SET interest_amount_to_frequency = jsonb_set(
                interest_amount_to_frequency,
                ARRAY[$1]::text[],
                to_jsonb((interest_amount_to_frequency->>$1)::int - 1),
                true
            )
            WHERE id = $2", 
        &[&interest_amount_str, &deposit_id]
    ).await;
    if let Err(e) = result {
        return  Err(e.into());
    }
    Ok(())
}

pub async fn add_deposit(client: &Client, customer_id: Uuid, deposit_request: DepositRequest) -> Result<Deposit> {
    let interest_rate = 5.6;
    let deposit_tenure = deposit_request.deposit_tenure;
    let renewed_deposit_tenure = deposit_request.renewed_deposit_tenure;
    let interest_payout = deposit_request.interest_payout;
    let deposit_id = Uuid::new_v4();
    let creation_timestamp = Utc::now();
    let deposit_number = generate_account_number();
    let maturity_timestamp = get_maturity_timestamp(creation_timestamp, deposit_tenure.clone());
    let next_interest_timestamp = get_next_interest_timestamp(creation_timestamp, maturity_timestamp, interest_payout.clone());
    let interest_amount_to_frequency = get_interest_amount_to_frequency(
        deposit_request.principal_amount, 
        interest_rate, interest_payout.clone(), 
        Some(deposit_tenure.clone())
    );
    let deposit = Deposit {
        id: deposit_id,
        customer_id,
        deposit_number,
        linked_account_number: deposit_request.linked_account_number,
        principal_amount: deposit_request.principal_amount,
        interest_rate,
        interest_payout,
        auto_renewal: deposit_request.auto_renewal,
        creation_timestamp,
        status: DepositStatus::Active,
        deposit_tenure,
        renewed_deposit_tenure,
        interest_amount_to_frequency,
        total_interest_paid: 0.0,
        next_interest_date: next_interest_timestamp.map(|d| d.date_naive()),
        maturity_date: maturity_timestamp.date_naive(),
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
        interest_amount_to_frequency,
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
        &serde_json::to_value(&deposit.interest_amount_to_frequency).unwrap(),
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