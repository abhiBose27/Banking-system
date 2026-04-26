use tokio_postgres::Client;
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};

use object::interfaces::{
    statement::{StatementResponse, StatementRequest}, 
    transaction::TransactionType
};
use ulid::Ulid;


pub async fn get_statement_db(
    client: &Client,
    statement_request: StatementRequest,
    account_number: String
) -> Result<Vec<StatementResponse>> {
    let db_response = match (statement_request.from_date, statement_request.to_date) {
        (None, None) => {
            let rows = client.query("SELECT * FROM transaction WHERE from_acc = $1 OR to_acc = $2 ORDER BY transaction_timestamp DESC LIMIT 10",
            &[&account_number, &account_number]).await;
            rows
        },
        (None, Some(to)) => {
            let end   = Utc.from_utc_datetime(&to.succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap());
            let rows = client.query("SELECT * FROM transaction WHERE (from_acc = $1 OR to_acc = $2) AND transaction_timestamp < $3",
            &[&account_number, &account_number, &end]).await;
            rows
        },
        (Some(from), None) => {
            let start = Utc.from_utc_datetime(&from.and_hms_opt(0, 0, 0).unwrap());
            let rows = client.query("SELECT * FROM transaction WHERE (from_acc = $1 OR to_acc = $2) AND transaction_timestamp >= $3",
            &[&account_number, &account_number, &start]).await;
            rows
        },
        (Some(from), Some(to)) => {
            let start = Utc.from_utc_datetime(&from.and_hms_opt(0, 0, 0).unwrap());
            let end   = Utc.from_utc_datetime(&to.succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap());
            let rows = client.query("SELECT * FROM transaction WHERE (from_acc = $1 OR to_acc = $2) AND transaction_timestamp >= $3 transaction_timestamp < $4",
            &[&account_number, &account_number, &start, &end]).await;
            rows
        },
    };

    if let Err(e) = db_response {
        return Err(e.into())
    }
    
    let statement = db_response.unwrap().into_iter().map(|row| {
        let ts: DateTime<Utc> = row.get("transaction_timestamp");
        let from_acc: Option<String> = row.get("from_acc");
        let to_acc: Option<String> = row.get("to_acc");
        let amount = row.get("amount");
        let reference_id_str = row.get("reference_id");
        let transaction_type = if let Some(from) = from_acc.clone() {
            if from == account_number { TransactionType::Debit }
            else { TransactionType::Credit }
        } else { TransactionType::Credit };

        StatementResponse {
            date: ts.date_naive(),
            from_account_number: from_acc.clone(),
            to_account_number: to_acc.clone(),
            reference_id: Ulid::from_string(reference_id_str).unwrap(),
            transaction_type,
            amount,
        }

    }).collect::<Vec<StatementResponse>>();

    Ok(statement)
}