use chrono::Utc;
use object::interfaces::customer::{Customer, CustomerRequest};
use tokio_postgres::Client;
use ulid::Ulid;
use uuid::Uuid;
use anyhow::Result;

pub async fn get_customer(client: &Client, pan_id: String, first_name: String, last_name: String) -> Result<Customer> {
    let row_result = client
        .query_one("SELECT * FROM customer WHERE pan_id = $1 AND first_name = $2 AND last_name = $3",
        &[
            &pan_id,
            &first_name,
            &last_name
        ]).await;
    if let Err(e) = row_result {
        return Err(e.into());
    }
    let row = row_result.unwrap();
    let customer_reference_id = Ulid::from_string(row.get("customer_reference_id")).unwrap();
    let customer = Customer {
        id: row.get("id"),
        customer_reference_id,
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        pan_id: row.get("pan_id"),
        email_id: row.get("email_id"),
        age: row.get("age"),
        date_of_birth: row.get("date_of_birth"),
        contact_number: row.get("contact_number"),
        creation_timestamp: row.get("creation_timestamp"),
    };
    Ok(customer)
}

pub async fn add_customer(client: &Client, customer_request: CustomerRequest) -> Result<Customer> {
    let customer_id = Uuid::new_v4();
    let customer = Customer {
        id: customer_id,
        customer_reference_id: Ulid::new(),
        first_name: customer_request.first_name,
        last_name: customer_request.last_name,
        pan_id: customer_request.pan_id,
        email_id: customer_request.email_id,
        age: customer_request.age,
        date_of_birth: customer_request.date_of_birth,
        contact_number: customer_request.contact_number,
        creation_timestamp: Utc::now(),
    };
    let result = client.execute("INSERT INTO customer (
        id, customer_reference_id, first_name, last_name, pan_id, email_id, age,
        date_of_birth, contact_number, creation_timestamp
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)", 
    &[
        &customer.id,
        &customer.customer_reference_id.to_string(),
        &customer.first_name,
        &customer.last_name,
        &customer.pan_id,
        &customer.email_id,
        &customer.age,
        &customer.date_of_birth,
        &customer.contact_number,
        &customer.creation_timestamp
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(customer)
}
