use chrono::Utc;
use tokio_postgres::Client;
use anyhow::Result;
use ulid::Ulid;
use uuid::Uuid;

use object::interfaces::{transaction::{Transaction, TransactionRequest, TransactionStatus}};


pub async fn make_transaction_db(
    client: &Client, 
    transaction_request: TransactionRequest,
    transaction_status: TransactionStatus
) -> Result<Transaction> {
    let reference_id = Ulid::new();
    let transaction = Transaction {
        id: Uuid::new_v4(),
        reference_id,
        from_account_number: transaction_request.from_account_number,
        to_account_number: transaction_request.to_account_number,
        transaction_status,
        transaction_timestamp: Utc::now(),
        amount: transaction_request.amount,
    };
    let result = client.execute("INSERT INTO transaction (
        id, amount, reference_id, from_acc, to_acc, transaction_status, transaction_timestamp
    ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    &[
        &transaction.id,
        &transaction.amount,
        &transaction.reference_id.to_string(),
        &transaction.from_account_number,
        &transaction.to_account_number,
        &serde_json::to_string(&transaction.transaction_status).unwrap(),
        &transaction.transaction_timestamp
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(transaction)
}