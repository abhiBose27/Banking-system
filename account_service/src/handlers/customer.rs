use tokio_postgres::Client;
use ulid::Ulid;
use uuid::Uuid;

use object::interfaces::{customer::{CustomerDetail, CustomerRequest}, io::DataKind};

use crate::database::customer::{add_customer, get_customer_from_customer_id, get_customer_from_customer_reference_id};


pub async fn create_customer(client: &Client, customer_request: CustomerRequest) -> (bool, Option<DataKind>, Option<String>) {
    match add_customer(&client, customer_request).await {
        Ok(customer) => {
            let customer_detail = CustomerDetail {
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
            (true, Some(DataKind::CreateCustomerResponse { customer_detail }), None)
        },
        Err(e) => {
            eprintln!("Error: {e}");
            (false, None, Some("Error: Failed to add customer".to_string()))
        },
    }
}

pub async fn get_customer(
    client: &Client,
    customer_reference_id: Option<Ulid>,
    session_customer_id: Option<Uuid>,
    is_detail: bool
) -> (bool, Option<DataKind>, Option<String>) {
    let mut customer_result = None;
    if let Some(customer_id) = session_customer_id {
        match get_customer_from_customer_id(client, customer_id).await {
            Ok(cust) => {
                if customer_reference_id.is_some() && cust.customer_reference_id != customer_reference_id.unwrap() {
                    return (false, None, Some("Error: Invalid session customer id".to_string()));
                }
                customer_result = Some(cust);
            },
            Err(e) => {
                eprintln!("Error: {e}");
                return (false, None, Some("Error: Failed to get customer".to_string()))
            }
        };
    }
    if let Some(cust_reference_id) = customer_reference_id {
        match get_customer_from_customer_reference_id(client, cust_reference_id).await {
            Ok(cust) => customer_result = Some(cust),
            Err(e) => {
                eprintln!("Error: {e}");
                return (false, None, Some("Error: Failed to get customer".to_string()))
            }
        };
    }
    if customer_result.is_none() {
        return (false, None, Some("Error: Parameters not provided".to_string()))
    }
    if is_detail {
        let customer = customer_result.unwrap();
        let customer_detail = CustomerDetail {
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
        return (true, Some(DataKind::GetCustomerDetailResponse { customer_detail }), None);
    }
    return (true, Some(DataKind::GetCustomerResponse { customer: customer_result.unwrap() }), None); 
}