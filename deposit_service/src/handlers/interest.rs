use chrono::{TimeZone, Utc};
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{service_job::ServiceJob, transaction::{TransactionRequest}};

use crate::{database::{
    deposit::update_deposit, 
    interest::get_deposit_for_interest}, 
    requests::transaction::make_transaction, utils::interest::{get_interest_payout_amount, get_next_interest_timestamp}
};

pub async fn process_interests(client: &Client, tx_dealer: &Sender<ServiceJob>) {
    let deposits_to_process = get_deposit_for_interest(&client).await.unwrap();
    for deposit in deposits_to_process.iter() {
        let interest_amount = get_interest_payout_amount(
            deposit.total_interest_paid, 
            deposit.total_interest_amount, 
            &deposit.interest_payout, 
            &deposit.deposit_tenure
        );
        if interest_amount.is_none() {
            continue;
        }
        let transaction_request = TransactionRequest {
            amount: interest_amount.unwrap(),
            from_account_number: None,
            to_account_number: Some(deposit.linked_account_number.clone()),
        };
        let transaction_response = make_transaction(tx_dealer, transaction_request, None).await;
        if let None = transaction_response {
            eprintln!("Error: Unable to process interest for {} to {}", deposit.deposit_number, deposit.linked_account_number);
            continue;
        }
        
        let total_interest_paid = deposit.total_interest_paid + interest_amount.unwrap();

        let interest_date = deposit.next_interest_date.unwrap();
        let maturity_date = deposit.maturity_date;
        let interest_timestamp = Utc.from_utc_datetime(&interest_date.and_hms_opt(0, 0, 0).unwrap());
        let maturity_timestamp = Utc.from_utc_datetime(&maturity_date.and_hms_opt(0, 0, 0).unwrap());
        let next_interest_timestamp = get_next_interest_timestamp(
            interest_timestamp, 
            maturity_timestamp, 
            &deposit.interest_payout
        );
        match update_deposit(client, deposit.id, total_interest_paid, next_interest_timestamp).await {
            Ok(_) => println!("Interest paid for deposit id: {:?}", deposit.id),
            Err(e) => eprintln!("Error {e}: Cannot pay for deposit id: {:?}", deposit.id),
        };
    }
}