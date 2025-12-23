use tokio_postgres::Client;

use object::interfaces::{io::DataKind, statement::StatementRequest};

use crate::database::statement::get_statement_db;

pub async fn get_statement(
    client: &Client,
    statement_request: StatementRequest,
) -> (bool, Option<DataKind>, Option<String>) {
    match get_statement_db(client, statement_request.clone()).await {
        Ok(statement) => {
            let data = Some(DataKind::GetStatementResponse { statement });
            (true, data, None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to get statement".to_string()))
        },
    }
}