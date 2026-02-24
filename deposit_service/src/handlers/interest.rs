use chrono::{TimeZone, Utc};
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{deposit::InterestPayout, service_job::ServiceJob, transaction::{TransactionRequest}};

use crate::{database::{
    deposit::{get_next_interest_timestamp, update_deposit}, 
    interest::get_deposit_for_interest}, 
    requests::transaction::make_transaction
};

pub async fn process_interests(client: &Client, tx_dealer: &Sender<ServiceJob>) {
    let deposits_to_process = get_deposit_for_interest(&client).await.unwrap();
    for deposit in deposits_to_process.iter() {
        if deposit.interest_payout == InterestPayout::Renew {
            continue;
        }
        let interest_amount_str = deposit.interest_amount_to_frequency
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, _)| k.clone()).unwrap();

        let interest_amount = interest_amount_str.parse::<f64>().unwrap();
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

        let total_interest_paid = deposit.total_interest_paid + interest_amount;
        let interest_date = deposit.next_interest_date.unwrap();
        let maturity_date = deposit.maturity_date;
        let interest_timestamp = Utc.from_utc_datetime(&interest_date.and_hms_opt(0, 0, 0).unwrap());
        let maturity_timestamp = Utc.from_utc_datetime(&maturity_date.and_hms_opt(0, 0, 0).unwrap());
        let next_interest_timestamp = get_next_interest_timestamp(
            interest_timestamp, 
            maturity_timestamp, 
            deposit.interest_payout.clone()
        );
        match update_deposit(client, deposit.id, interest_amount, total_interest_paid, next_interest_timestamp).await {
            Ok(_) => println!("Interest paid for deposit id: {:?}", deposit.id),
            Err(e) => eprintln!("Error {e}: Cannot pay for deposit id: {:?}", deposit.id),
        };
    }
}