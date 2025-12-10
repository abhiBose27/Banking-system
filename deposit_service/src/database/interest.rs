use tokio_postgres::Client;
use anyhow::Result;

use object::interfaces::deposit::Deposit;

pub async fn get_deposit_for_interest(client: &Client) -> Result<Vec<Deposit>> {
    let result = client.query(
    "
        SELECT * FROM deposit_account
        WHERE next_interest_date <= CURRENT_DATE AND
        status = '\"Active\"'
        FOR UPDATE SKIP LOCKED
        LIMIT 500;
    ", &[]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    let rows = result.unwrap();
    let deposits = rows.into_iter().map(|row| {
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
            auto_renewal: row.get("auto_renewal"),
            renewed_deposit_tenure: serde_json::from_value(row.get("renewed_deposit_tenure")).unwrap(),
            creation_timestamp: row.get("creation_timestamp"),
            next_interest_date: row.get("next_interest_date"),
            maturity_date: row.get("maturity_date"),
            interest_amounts: row.get("interest_amounts"),
            nb_payouts: row.get("nb_payouts")
        }
    }).collect::<Vec<Deposit>>();
    Ok(deposits)
}