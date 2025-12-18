use object::interfaces::service_config::ServiceConfig;
use tokio_postgres::{Error, NoTls};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    match envy::prefixed("DATABASE_TEMPLATE_").from_env::<ServiceConfig>() {
        Ok(config) => {
            let config_str = format!(
                "host={0} user={1} password={2} dbname={3}",                    
                config.db_host,
                config.db_user,
                config.db_password,
                config.db_database
            );
            let (client, connection) = tokio_postgres::connect(&config_str, NoTls).await?;

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {e}");
                }
            });
            client.batch_execute(
                "CREATE TABLE IF NOT EXISTS account(
                    id UUID PRIMARY KEY,
                    customer_id UUID NOT NULL,
                    account_number TEXT NOT NULL,
                    balance DOUBLE PRECISION NOT NULL,
                    creation_timestamp TIMESTAMPTZ NOT NULL
                );"
            ).await.unwrap();
            
            client.batch_execute("
                CREATE TABLE IF NOT EXISTS deposit_account(
                    id UUID PRIMARY KEY,
                    status TEXT NOT NULL,
                    customer_id UUID NOT NULL,
                    deposit_number TEXT NOT NULL,
                    linked_account_number TEXT NOT NULL,
                    principal_amount DOUBLE PRECISION NOT NULL,
                    interest_rate DOUBLE PRECISION NOT NULL,
                    deposit_tenure JSONB NOT NULL,
                    interest_payout TEXT NOT NULL,
                    interest_amount_to_frequency JSONB NOT NULL,
                    total_interest_paid DOUBLE PRECISION NOT NULL,
                    auto_renewal BOOLEAN NOT NULL,
                    renewed_deposit_tenure JSONB,
                    creation_timestamp TIMESTAMPTZ NOT NULL,
                    maturity_date DATE NOT NULL,
                    next_interest_date DATE
                );"
            ).await.unwrap();
            
            client.batch_execute(
                "CREATE TABLE IF NOT EXISTS customer(
                    id UUID PRIMARY KEY,
                    customer_reference_id TEXT NOT NULL UNIQUE,
                    first_name TEXT NOT NULL,
                    last_name TEXT NOT NULL,
                    pan_id CHAR(10) NOT NULL UNIQUE,
                    email_id TEXT NOT NULL,
                    age BIGINT NOT NULL,
                    date_of_birth DATE NOT NULL,
                    contact_number TEXT NOT NULL,
                    creation_timestamp TIMESTAMPTZ NOT NULL
                );"
            ).await.unwrap();

            client.batch_execute("
                CREATE TABLE IF NOT EXISTS transaction(
                    id UUID PRIMARY KEY,
                    amount DOUBLE PRECISION NOT NULL,
                    reference_id TEXT NOT NULL UNIQUE,
                    from_acc TEXT,
                    to_acc TEXT,
                    transaction_timestamp TIMESTAMPTZ NOT NULL
                );"
            ).await.unwrap();
        }
        Err(e) => eprintln!("Error: {e}")
    }
    Ok(())
}
