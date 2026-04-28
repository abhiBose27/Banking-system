use chrono::{TimeZone, Utc};
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{deposit::InterestPayout, service_job::ServiceJob, transaction::{TransactionRequest}};

use crate::{database::{
    deposit::update_deposit, 
    interest::get_deposit_for_interest}, 
    requests::transaction::make_transaction, tools::tools::get_next_interest_timestamp
};

pub async fn process_interests(client: &Client, tx_dealer: &Sender<ServiceJob>) {
    let deposits_to_process = get_deposit_for_interest(&client).await.unwrap();
    for deposit in deposits_to_process.iter() {
        let total_days = deposit.deposit_tenure.days + deposit.deposit_tenure.months * 30 + deposit.deposit_tenure.years * 360;
        let interest_amount = match deposit.interest_payout {
            InterestPayout::Daily => deposit.total_interest_amount / total_days as f64,
            InterestPayout::Monthly => {
                let amount = (deposit.total_interest_amount / total_days as f64) * 30.0;
                if amount + deposit.total_interest_paid > deposit.total_interest_amount {deposit.total_interest_amount - deposit.total_interest_paid}
                else {amount}
            },
            InterestPayout::Quaterly => {
                let amount = (deposit.total_interest_amount / total_days as f64) * 90.0;
                if amount + deposit.total_interest_paid > deposit.total_interest_amount {deposit.total_interest_amount - deposit.total_interest_paid}
                else {amount}
            },
            InterestPayout::Maturity => deposit.total_interest_amount,
            InterestPayout::Renew => continue,
        };
        let transaction_request = TransactionRequest {
            amount: interest_amount,
            from_account_number: None,
            to_account_number: Some(deposit.linked_account_number.clone()),
        };
        let transaction_response = make_transaction(tx_dealer, transaction_request, None).await;
        if let None = transaction_response {
            eprintln!("Error: Unable to process interest for {} to {}", deposit.deposit_number, deposit.linked_account_number);
            continue;
        }
        let interest_date = deposit.next_interest_date.unwrap();
        let maturity_date = deposit.maturity_date;
        let interest_timestamp = Utc.from_utc_datetime(&interest_date.and_hms_opt(0, 0, 0).unwrap());
        let maturity_timestamp = Utc.from_utc_datetime(&maturity_date.and_hms_opt(0, 0, 0).unwrap());
        let next_interest_timestamp = get_next_interest_timestamp(
            interest_timestamp, 
            maturity_timestamp, 
            deposit.interest_payout.clone()
        );
        let total_interest_paid = deposit.total_interest_paid + interest_amount;
        match update_deposit(client, deposit.id, total_interest_paid, next_interest_timestamp).await {
            Ok(_) => println!("Interest paid for deposit id: {:?}", deposit.id),
            Err(e) => eprintln!("Error {e}: Cannot pay for deposit id: {:?}", deposit.id),
        };
    }
}