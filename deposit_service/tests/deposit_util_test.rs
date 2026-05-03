use chrono::{Duration, Utc};
use deposit_service::utils::deposit::{get_maturity_timestamp, get_premature_withdrawal_amount, is_valid_deposit_tenure};
use object::interfaces::deposit::DepositTenure;

 #[test]
fn test_is_valid_deposit_tenure_invalid() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 0,
        days: 0,
    };
    let is_valid = is_valid_deposit_tenure(Some(deposit_tenure)); 
    assert_eq!(is_valid, false);
}

#[test]
fn test_is_valid_deposit_tenure_none() {
    let is_valid = is_valid_deposit_tenure(None); 
    assert_eq!(is_valid, true);
}

#[test]
fn test_is_valid_deposit_tenure_valid() {
    let deposit_tenure = DepositTenure {
        years: 1,
        months: 0,
        days: 10,
    };
    let is_valid = is_valid_deposit_tenure(Some(deposit_tenure)); 
    assert_eq!(is_valid, true);
}

#[test]
fn test_premature_withdrawal_amount_1() {
    let creation_timestamp = Utc::now() - Duration::days(360);
    let premature_withdrawal_amount = get_premature_withdrawal_amount(
        1000.0, 
        6.5,
        50.0,
        creation_timestamp
    );
    assert_eq!(premature_withdrawal_amount, 1015.0)
}

#[test]
fn test_premature_withdrawal_amount_2() {
    let creation_timestamp = Utc::now() - Duration::days(330);
    let premature_withdrawal_amount = get_premature_withdrawal_amount(
        1000.0, 
        6.5,
        50.0,
        creation_timestamp
    );
    assert_eq!(premature_withdrawal_amount, 1009.5833333333334)
}

#[test]
fn test_maturity_timestamp_1() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 0,
        days: 0,
    };
    let current_timestamp = Utc::now();
    let maturity_timestamp = get_maturity_timestamp(
        current_timestamp,
        &deposit_tenure
    );
    assert_eq!(maturity_timestamp, current_timestamp);
}

#[test]
fn test_maturity_timestamp_2() {
    let deposit_tenure = DepositTenure {
        years: 1,
        months: 0,
        days: 10,
    };
    let mut current_timestamp = Utc::now();
    let maturity_timestamp = get_maturity_timestamp(
        current_timestamp,
        &deposit_tenure
    );
    current_timestamp = current_timestamp + Duration::days(30 * 12 as i64);
    current_timestamp = current_timestamp + Duration::days(10 as i64);
    assert_eq!(maturity_timestamp, current_timestamp);
}