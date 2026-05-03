use chrono::{Duration, Utc};
use deposit_service::utils::interest::{get_interest_payout_amount, get_next_interest_timestamp, get_total_interest_amount, is_valid_interest_payout};
use object::interfaces::deposit::{DepositTenure, InterestPayout};

#[test]
fn test_is_valid_interest_payout_daily() {
    let deposit_tenure = DepositTenure {
        years: 1,
        months: 1,
        days: 10
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Daily, Some(deposit_tenure));
    assert_eq!(is_valid, true);
}

#[test]
fn test_is_valid_interest_payout_monthly() {
    let deposit_tenure = DepositTenure {
        years: 1,
        months: 1,
        days: 10
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Monthly, Some(deposit_tenure));
    assert_eq!(is_valid, true);
}

#[test]
fn test_is_valid_interest_payout_monthly_invalid() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 0,
        days: 29
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Monthly, Some(deposit_tenure));
    assert_eq!(is_valid, false);
}

#[test]
fn test_is_valid_interest_payout_quaterly() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 1,
        days: 60
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Quaterly, Some(deposit_tenure));
    assert_eq!(is_valid, true);
}

#[test]
fn test_is_valid_interest_payout_quaterly_invalid() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 0,
        days: 89
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Quaterly, Some(deposit_tenure));
    assert_eq!(is_valid, false);
}

#[test]
fn test_is_valid_interest_payout_maturity() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 0,
        days: 0
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Maturity, Some(deposit_tenure));
    assert_eq!(is_valid, true);
}

#[test]
fn test_is_valid_interest_payout_renew() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 0,
        days: 0
    };
    let is_valid = is_valid_interest_payout(&InterestPayout::Renew, Some(deposit_tenure));
    assert_eq!(is_valid, true);
}

#[test]
fn test_is_valid_interest_payout_deposit_tenure_none() {
    let is_valid = is_valid_interest_payout(&InterestPayout::Renew, None);
    assert_eq!(is_valid, true);
}

#[test]
fn test_next_interest_timestamp_same_timestamp() {
    let current_timestamp = Utc::now();
    let maturity_timestamp = current_timestamp.clone();
    let next_interest_timestamp = get_next_interest_timestamp(current_timestamp, maturity_timestamp, &InterestPayout::Daily);
    assert_eq!(next_interest_timestamp, Some(maturity_timestamp));
}

#[test]
fn test_next_interest_timestamp_daily() {
    let current_timestamp = Utc::now();
    let maturity_timestamp = current_timestamp + Duration::days(30);
    let next_interest_timestamp = get_next_interest_timestamp(current_timestamp, maturity_timestamp, &InterestPayout::Daily);
    assert_eq!(next_interest_timestamp, Some(current_timestamp + Duration::days(1)));
}

#[test]
fn test_next_interest_timestamp_monthly() {
    let current_timestamp = Utc::now();
    let maturity_timestamp = current_timestamp + Duration::days(1 * 12 * 30);
    let next_interest_timestamp = get_next_interest_timestamp(current_timestamp, maturity_timestamp, &InterestPayout::Monthly);
    assert_eq!(next_interest_timestamp, Some(current_timestamp + Duration::days(30)));

    let next_interest_timestamp = get_next_interest_timestamp(next_interest_timestamp.unwrap(), maturity_timestamp, &InterestPayout::Monthly);
    assert_eq!(next_interest_timestamp, Some(current_timestamp + Duration::days(60)));
}

#[test]
fn test_next_interest_timestamp_quaterly() {
    let current_timestamp = Utc::now();
    let maturity_timestamp = current_timestamp + Duration::days(1 * 12 * 30);
    let next_interest_timestamp = get_next_interest_timestamp(current_timestamp, maturity_timestamp, &InterestPayout::Quaterly);
    assert_eq!(next_interest_timestamp, Some(current_timestamp + Duration::days(3 * 30)));

    let next_interest_timestamp = get_next_interest_timestamp(next_interest_timestamp.unwrap(), maturity_timestamp, &InterestPayout::Quaterly);
    assert_eq!(next_interest_timestamp, Some(current_timestamp + Duration::days(2 * 3 * 30)));
}

#[test]
fn test_next_interest_timestamp_maturity() {
    let current_timestamp = Utc::now();
    let maturity_timestamp = current_timestamp + Duration::days(1 * 12 * 30);
    let next_interest_timestamp = get_next_interest_timestamp(current_timestamp, maturity_timestamp, &InterestPayout::Maturity);
    assert_eq!(next_interest_timestamp, Some(maturity_timestamp));
}

#[test]
fn test_next_interest_timestamp_none() {
    let current_timestamp = Utc::now();
    let maturity_timestamp = current_timestamp + Duration::days(1 * 12 * 30);
    let next_interest_timestamp = get_next_interest_timestamp(current_timestamp, maturity_timestamp, &InterestPayout::Renew);
    assert_eq!(next_interest_timestamp, None);
}

#[test]
fn test_interest_payout_amount_daily() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(50.0, 1000.0, &InterestPayout::Daily, &deposit_tenure);
    assert_eq!(interest_payout_amount, Some(3.3333333333333335))
}

#[test]
fn test_interest_payout_amount_monthly_1() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(50.0, 1000.0, &InterestPayout::Monthly, &deposit_tenure);
    assert_eq!(interest_payout_amount, Some(100.0))
}

#[test]
fn test_interest_payout_amount_monthly_2() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(910.0, 1000.0, &InterestPayout::Monthly, &deposit_tenure);
    assert_eq!(interest_payout_amount, Some(90.0))
}

#[test]
fn test_interest_payout_amount_quaterly_1() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(50.0, 1000.0, &InterestPayout::Quaterly, &deposit_tenure);
    assert_eq!(interest_payout_amount, Some(300.0))
}

#[test]
fn test_interest_payout_amount_quaterly_2() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(720.0, 1000.0, &InterestPayout::Quaterly, &deposit_tenure);
    assert_eq!(interest_payout_amount, Some(280.0))
}

#[test]
fn test_interest_payout_amount_maturity() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(0.0, 1000.0, &InterestPayout::Maturity, &deposit_tenure);
    assert_eq!(interest_payout_amount, Some(1000.0))
}

#[test]
fn test_interest_payout_amount_renew() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let interest_payout_amount = get_interest_payout_amount(0.0, 1000.0, &InterestPayout::Renew, &deposit_tenure);
    assert_eq!(interest_payout_amount, None)
}

#[test]
fn test_total_interest_amount() {
    let deposit_tenure = DepositTenure {
        years: 0,
        months: 10,
        days: 0
    };
    let total_interest_amount = get_total_interest_amount(6.5, 1000.0, &deposit_tenure);
    assert_eq!(total_interest_amount, 54.166666666666664);
}