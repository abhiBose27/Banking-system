use object::interfaces::{customer::CustomerRequest, io::{DataKind}};
use tokio_postgres::Client;

use crate::database::customer::add_customer;

pub async fn create_customer(client: &Client, customer_request: CustomerRequest) -> (bool, Option<DataKind>, Option<String>) {
    let mut executed = false;
    let mut error_message = None;
    match add_customer(&client, customer_request).await {
        Ok(_) => executed = true,
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to add customer".to_string());
        },
    }
    (executed, None, error_message)
}