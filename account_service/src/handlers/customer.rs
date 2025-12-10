use tokio_postgres::Client;

use object::interfaces::{customer::{CustomerResponse, CustomerRequest}, io::DataKind};

use crate::database::customer::add_customer;

pub async fn create_customer(client: &Client, customer_request: CustomerRequest) -> (bool, Option<DataKind>, Option<String>) {
    let mut success = false;
    let mut data = None;
    let mut error_message = None;
    match add_customer(&client, customer_request).await {
        Ok(customer) => {
            success = true;
            let customer_api = CustomerResponse {
                customer_reference_id: customer.customer_reference_id,
                first_name: customer.first_name,
                last_name: customer.last_name,
                pan_id: customer.pan_id,
                email_id: customer.email_id,
                age: customer.age,
                date_of_birth: customer.date_of_birth,
                contact_number: customer.contact_number,
                creation_timestamp: customer.creation_timestamp,
            };
            data = Some(DataKind::CreateCustomerResponse { customer: customer_api.clone() })
        },
        Err(e) => {
            eprintln!("Error: {e}");
            error_message = Some("Error: Failed to add customer".to_string());
        },
    }
    (success, data, error_message)
}