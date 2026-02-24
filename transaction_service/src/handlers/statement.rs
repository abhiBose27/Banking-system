use uuid::Uuid;
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{io::DataKind, service_job::ServiceJob, statement::StatementRequest};

use crate::{database::statement::get_statement_db, requests::account::get_account};

pub async fn get_statement(
    client: &Client,
    tx_dealer: &Sender<ServiceJob>,
    customer_id: Option<Uuid>,
    statement_request: StatementRequest,
) -> (bool, Option<DataKind>, Option<String>) {
    let account_result = get_account(tx_dealer, statement_request.account_number.clone(), customer_id).await;
    if let None = account_result {
        return (false, None, Some("Error: Cannot fetch account details".to_string()));
    }
    let account = account_result.unwrap();
    if customer_id.is_some() && customer_id.unwrap() != account.customer_id {
        return (false, None, Some("Error: Invalid customer id".to_string()));
    }
    match get_statement_db(client, statement_request.clone()).await {
        Ok(statement) => (true, Some(DataKind::GetStatementResponse { statement }), None),
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to get statement".to_string()))
        },
    }
}