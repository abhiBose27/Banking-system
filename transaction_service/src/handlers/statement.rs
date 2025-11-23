use object::interfaces::{io::DataKind, statement::StatementRequest};
use tokio_postgres::Client;

use crate::database::statement::get_statement_db;

pub async fn get_statement(
    client: &Client,
    statement_request: StatementRequest,
) -> (bool, Option<DataKind>, Option<String>) {
    let mut executed = false;
    let mut data = None;
    let mut error_message = None;
    match get_statement_db(client, statement_request.clone()).await {
        Ok(statement) => {
            executed = true;
            data = Some(DataKind::Statement { statement });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to get statement".to_string());
        },
    };
    (executed, data, error_message)
}