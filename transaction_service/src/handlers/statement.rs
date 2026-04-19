use uuid::Uuid;
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{io::DataKind, service_job::ServiceJob, statement::StatementRequest};

use crate::{database::statement::get_statement_db, requests::account::get_account};

pub async fn get_statement(
    client: &Client,
    tx_dealer: &Sender<ServiceJob>,
    session_customer_id: Option<Uuid>,
    statement_request: StatementRequest,
) -> (bool, Option<DataKind>, Option<String>) {
    let account_result = get_account(tx_dealer, statement_request.account_number.clone(), session_customer_id).await;
    if let None = account_result {
        return (false, None, Some("Error: Invalid credentials".to_string()));
    }
    match get_statement_db(client, statement_request).await {
        Ok(statement) => (true, Some(DataKind::GetStatementResponse { statement }), None),
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to get statement".to_string()))
        },
    }
}