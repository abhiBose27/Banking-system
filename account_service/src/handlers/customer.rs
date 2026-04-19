use tokio_postgres::Client;
use ulid::Ulid;

use object::interfaces::{customer::{CustomerResponse, CustomerRequest}, io::DataKind};
use uuid::Uuid;

use crate::database::customer::{add_customer, get_customer_from_customer_id, get_customer_from_customer_reference_id};


pub async fn create_customer(client: &Client, customer_request: CustomerRequest) -> (bool, Option<DataKind>, Option<String>) {
    match add_customer(&client, customer_request).await {
        Ok(customer) => {
            let data = Some(DataKind::CreateCustomerResponse { 
                customer: CustomerResponse {
                    customer_reference_id: customer.customer_reference_id,
                    creation_timestamp: customer.creation_timestamp,
                    first_name: customer.first_name.clone(),
                    last_name: customer.last_name.clone(),
                    pan_id: customer.pan_id.clone(),
                    email_id: customer.email_id.clone(),
                    age: customer.age,
                    date_of_birth: customer.date_of_birth,
                    contact_number: customer.contact_number,
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

pub async fn get_customer(client: &Client, customer_reference_id: Option<Ulid>, session_customer_id: Option<Uuid>) -> (bool, Option<DataKind>, Option<String>) {
    if let Some(customer_id) = session_customer_id {
        match get_customer_from_customer_id(client, customer_id).await {
            Ok(customer) => {
                if customer_reference_id.is_some() && customer.customer_reference_id != customer_reference_id.unwrap() {
                    return (false, None, Some("Error: Invalid session customer id".to_string()));
                }
                let customer_response = CustomerResponse {
                    customer_reference_id: customer.customer_reference_id,
                    creation_timestamp: customer.creation_timestamp,
                    first_name: customer.first_name.clone(),
                    last_name: customer.last_name.clone(),
                    pan_id: customer.pan_id.clone(),
                    email_id: customer.email_id.clone(),
                    age: customer.age,
                    date_of_birth: customer.date_of_birth,
                    contact_number: customer.contact_number,
                };
                return (true, Some(DataKind::GetCustomerResponse { customer: customer_response }), None);
            },
            Err(e) => {
                eprintln!("Error: {e}");
                return (false, None, Some("Error: Failed to get customer".to_string()))
            }
        };
    }
    if let Some(cust_reference_id) = customer_reference_id {
        match get_customer_from_customer_reference_id(client, cust_reference_id).await {
            Ok(customer) => {
                let customer_response = CustomerResponse {
                    customer_reference_id: customer.customer_reference_id,
                    creation_timestamp: customer.creation_timestamp,
                    first_name: customer.first_name.clone(),
                    last_name: customer.last_name.clone(),
                    pan_id: customer.pan_id.clone(),
                    email_id: customer.email_id.clone(),
                    age: customer.age,
                    date_of_birth: customer.date_of_birth,
                    contact_number: customer.contact_number,
                };
                return (true, Some(DataKind::GetCustomerResponse { customer: customer_response }), None)
            },
            Err(e) => {
                eprintln!("Error: {e}");
                return (false, None, Some("Error: Failed to get customer".to_string()))
            }
        };
    }
    (false, None, Some("Error: Parameters not provided".to_string()))
}

pub async fn get_customer_pvt(client: &Client, customer_reference_id: Option<Ulid>, session_customer_id: Option<Uuid>) -> (bool, Option<DataKind>, Option<String>) {
    if let Some(customer_id) = session_customer_id {
        match get_customer_from_customer_id(client, customer_id).await {
            Ok(customer) => {
                if customer_reference_id.is_some() && customer.customer_reference_id != customer_reference_id.unwrap() {
                    return (false, None, Some("Error: Invalid session customer id".to_string()));
                }
                return (true, Some(DataKind::GetCustomerPvtResponse { customer }), None);
            },
            Err(e) => {
                eprintln!("Error: {e}");
                return (false, None, Some("Error: Failed to get customer".to_string()))
            }
        };
    }
    if let Some(cust_reference_id) = customer_reference_id {
        match get_customer_from_customer_reference_id(client, cust_reference_id).await {
            Ok(customer) => {
                return (true, Some(DataKind::GetCustomerPvtResponse { customer }), None)
            },
            Err(e) => {
                eprintln!("Error: {e}");
                return (false, None, Some("Error: Failed to get customer".to_string()))
            }
        };
    }
    (false, None, Some("Error: Parameters not provided".to_string()))
}