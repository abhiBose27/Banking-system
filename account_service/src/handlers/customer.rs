use tokio_postgres::Client;
use ulid::Ulid;

use object::interfaces::{customer::{CustomerResponse, CustomerRequest}, io::DataKind};

use crate::database::customer::{add_customer, get_customer as get_customer_db};


pub async fn create_customer(client: &Client, customer_request: CustomerRequest) -> (bool, Option<DataKind>, Option<String>) {
    match add_customer(&client, customer_request).await {
        Ok(customer) => {
            let data = Some(DataKind::CreateCustomerResponse { 
                customer: CustomerResponse {
                    customer_reference_id: customer.customer_reference_id,
                    creation_timestamp: customer.creation_timestamp,
                }
            });
            (true, data, None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to add customer".to_string()))
        },
    }
}

pub async fn get_customer(client: &Client, customer_reference_id: Ulid) -> (bool, Option<DataKind>, Option<String>) {
    match get_customer_db(client, customer_reference_id).await {
        Ok(customer) => {
            (true, Some(DataKind::GetCustomerResponse { customer }), None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to get customer".to_string()))
        }
    }
}