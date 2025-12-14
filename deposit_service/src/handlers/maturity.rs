use tokio::sync::mpsc::Sender;
use tokio_postgres::Client;

use object::interfaces::{deposit::{DepositRequest, DepositResponse, InterestPayout}, service_job::ServiceJob, transaction::{TransactionRequest, TransactionStatus}};

use crate::{
    database::{
        deposit::{add_deposit, close_deposit}, 
        maturity::get_deposit_for_maturity}, 
        handlers::deposit::{make_transaction
        }
    };

pub async fn process_maturity(client: &Client, tx_dealer: &Sender<ServiceJob>) {
    let deposit_for_maturity = get_deposit_for_maturity(client).await.unwrap();
    for deposit in deposit_for_maturity {
        let mut amount = 0.0;
        
        // Calculate the new amount
        if deposit.interest_payout == InterestPayout::Renew {
            /* let interest_amounts = deposit.interest_amounts.clone();
            let nb_payouts = deposit.nb_payouts as usize;
            amount += interest_amounts[nb_payouts] + deposit.principal_amount;  */
            let interest_amount_str = deposit.interest_amount_to_frequency
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.clone()).unwrap();

            let interest_amount = interest_amount_str.parse::<f64>().unwrap();
            amount += interest_amount + deposit.principal_amount;
        }
        else {
            amount += deposit.principal_amount;
        }

        // Credit the new amount
        let transaction_request = TransactionRequest {
            amount,
            from_account_number: None,
            to_account_number: Some(deposit.linked_account_number.clone()),
        };
        let transaction_response = make_transaction(tx_dealer, transaction_request).await;
        if let None = transaction_response {
            eprintln!("Error: Unable to credit principal amount {} to {}", amount, deposit.linked_account_number);
            continue;
        }
        let transaction = transaction_response.unwrap();
        if transaction.transaction_status == TransactionStatus::Reject {
            eprintln!("Error: Unable to credit principal amount {} to {}", amount, deposit.linked_account_number);
            continue;
        }

        // Close the current deposit
        close_deposit(client, deposit.id).await.unwrap();

        // If its auto renewable
        // Create a new deposit with the new amount
        if deposit.auto_renewal {

            // Renewed deposit request
            let new_deposit_request = DepositRequest {
                linked_account_number: deposit.linked_account_number,
                principal_amount: amount,
                deposit_tenure: deposit.renewed_deposit_tenure.clone().unwrap(),
                interest_payout: deposit.interest_payout,
                auto_renewal: deposit.auto_renewal,
                renewed_deposit_tenure: deposit.renewed_deposit_tenure,
            };

            let transaction_request_debit = TransactionRequest {
                amount,
                from_account_number: Some(new_deposit_request.linked_account_number.clone()),
                to_account_number: None
            };
            let transaction_response_debit = make_transaction(tx_dealer, transaction_request_debit).await;
            if let None = transaction_response_debit {
                eprintln!("Error: Unable to debit principal amount {} to {}", amount, new_deposit_request.linked_account_number);
                continue;
            }
            let transaction = transaction_response_debit.unwrap();
            if transaction.transaction_status == TransactionStatus::Reject {
                eprintln!("Error: Unable to debit principal amount {} to {}", amount, new_deposit_request.linked_account_number);
                continue;
            }

            // Add deposit to DB
            let new_deposit = add_deposit(client, new_deposit_request, deposit.customer_id).await.unwrap();
            let deposit_response = DepositResponse {
                deposit_number: new_deposit.deposit_number,
                linked_account_number: new_deposit.linked_account_number,
                principal_amount: new_deposit.principal_amount,
                interest_rate: new_deposit.interest_rate,
                interest_payout: new_deposit.interest_payout,
                auto_renewal: new_deposit.auto_renewal,
                maturity_date: new_deposit.maturity_date,
                deposit_tenure: new_deposit.deposit_tenure,
                renewed_deposit_tenure: new_deposit.renewed_deposit_tenure,
                creation_timestamp: new_deposit.creation_timestamp,
            };
            println!("Renewed deposit {:?}", deposit_response);
        }
    }
}