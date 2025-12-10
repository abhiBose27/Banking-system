use chrono::{TimeZone, Utc};
use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{deposit::InterestPayout, service_job::ServiceJob, transaction::{TransactionRequest, TransactionStatus}};

use crate::{database::{
    deposit::{get_next_interest_timestamp, update_interest_date}, 
    interest::get_deposit_for_interest}, 
    handlers::deposit::make_transaction
};

pub async fn process_interests(client: &Client, tx_dealer: &Sender<ServiceJob>) {
    let deposits_to_process = get_deposit_for_interest(&client).await.unwrap();
    for deposit in deposits_to_process.iter() {
        if deposit.interest_payout == InterestPayout::Renew {
            continue;
        }
        let interest_amounts = deposit.interest_amounts.clone();
        let nb_payouts = deposit.nb_payouts as usize;
        let transaction_request = TransactionRequest {
            amount: interest_amounts[nb_payouts],
            from_account_number: None,
            to_account_number: Some(deposit.linked_account_number.clone()),
        };
        let transaction_response = make_transaction(tx_dealer, transaction_request).await;
        if let None = transaction_response {
            eprintln!("Error: Unable to process interest for {} to {}", deposit.deposit_number, deposit.linked_account_number);
            continue;
        }
        let transaction = transaction_response.unwrap();
        if transaction.transaction_status == TransactionStatus::Reject {
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
        let new_nb_payouts = (nb_payouts + 1) as i64;
        update_interest_date(client, deposit.id, new_nb_payouts, next_interest_timestamp).await.unwrap();
        println!("Interest Paid for deposit: {:?}", deposit);
    }
}