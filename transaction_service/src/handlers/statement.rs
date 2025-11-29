use tokio_postgres::Client;

use object::interfaces::{io::DataKind, statement::StatementRequest};

use crate::database::statement::get_statement_db;

pub async fn get_statement(
    client: &Client,
    statement_request: StatementRequest,
) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut data = None;
    let mut error_message = None;
    match get_statement_db(client, statement_request.clone()).await {
        Ok(statement) => {
            success = true;
            data = Some(DataKind::GetStatementResponse { statement });
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to get statement".to_string());
        },
    };
    (success, data, error_message)
}