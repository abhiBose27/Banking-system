use chrono::Utc;
use object::interfaces::{transaction::{Transaction, TransactionRequest, TransactionStatus}};
use tokio_postgres::Client;
use anyhow::Result;
use ulid::Ulid;
use uuid::Uuid;


pub async fn make_transaction_db(
    client: &Client, 
    transaction_request: TransactionRequest,
    transaction_status: TransactionStatus
) -> Result<Ulid> {
    let reference_id = Ulid::new();
    let transaction = Transaction {
        id: Uuid::new_v4(),
        reference_id,
        from_account_number: transaction_request.from_account_number.as_deref(),
        to_account_number: transaction_request.to_account_number.as_deref(),
        transaction_status,
        transaction_timestamp: Utc::now(),
    };
    let result = client.execute("INSERT INTO transaction (
        id, reference_id, from_acc, to_acc, transaction_status, transaction_timestamp
    ) VALUES ($1, $2, $3, $4, $5, $6)",
    &[
        &transaction.id,
        &transaction.reference_id.to_string(),
        &transaction.from_account_number,
        &transaction.to_account_number,
        &serde_json::to_string(&transaction.transaction_status).unwrap(),
        &transaction.transaction_timestamp
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(reference_id)
}