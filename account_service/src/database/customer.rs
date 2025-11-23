use chrono::Utc;
use object::interfaces::customer::{Customer, CustomerRequest};
use tokio_postgres::Client;
use uuid::Uuid;
use anyhow::Result;

pub async fn get_customer_id_from_pan_id(client: &Client, pan_id: String, first_name: String, last_name: String) -> Result<Uuid> {
    let row_result = client
        .query_one("SELECT id FROM customer WHERE pan_id = $1 AND first_name = $2 AND last_name = $3",
        &[&pan_id, &first_name, &last_name]
    ).await;
    if let Err(e) = row_result {
        return Err(e.into());
    }
    let row = row_result?;
    let id = row.get("id");
    Ok(id)
}

pub async fn add_customer(client: &Client, customer_request: CustomerRequest) -> Result<()> {
    let customer = Customer {
        id: Uuid::new_v4(),
        first_name: &customer_request.first_name,
        last_name: &customer_request.last_name,
        pan_id: &customer_request.pan_id,
        email_id: &customer_request.email_id,
        age: customer_request.age,
        date_of_birth: customer_request.date_of_birth,
        contact_number: &customer_request.contact_number,
        creation_timestamp: Utc::now(),
    };
    let result = client.execute("INSERT INTO customer (
        id, first_name, last_name, pan_id, email_id, age,
        date_of_birth, contact_number, account_ids
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)", 
    &[
        &customer.id,
        &customer.first_name,
        &customer.last_name,
        &customer.pan_id,
        &customer.email_id,
        &(customer.age as i64),
        &customer.date_of_birth,
        &customer.contact_number,
    ]).await;
    if let Err(e) = result {
        return Err(e.into());
    }
    Ok(())
}
