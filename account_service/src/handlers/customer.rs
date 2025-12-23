use tokio_postgres::Client;

use object::interfaces::{customer::{CustomerResponse, CustomerRequest}, io::DataKind};

use crate::database::customer::add_customer;

pub async fn create_customer(client: &Client, customer_request: CustomerRequest) -> (bool, Option<DataKind>, Option<String>) {
    match add_customer(&client, customer_request).await {
        Ok(customer) => {
            let data = Some(DataKind::CreateCustomerResponse { 
                customer: CustomerResponse {
                    customer_reference_id: customer.customer_reference_id,
                    first_name: customer.first_name,
                    last_name: customer.last_name,
                    pan_id: customer.pan_id,
                    email_id: customer.email_id,
                    age: customer.age,
                    date_of_birth: customer.date_of_birth,
                    contact_number: customer.contact_number,
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