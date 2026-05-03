use chrono::{DateTime, Duration, Utc};

use object::interfaces::deposit::{DepositTenure, InterestPayout};


pub fn is_valid_interest_payout(interest_payout: &InterestPayout, deposit_tenure: Option<DepositTenure>) -> bool {
    match deposit_tenure {
        Some(d) => {
            let total_days = d.days + d.months * 30 + d.years * 360;
            match interest_payout {
                InterestPayout::Monthly => total_days >= 30,
                InterestPayout::Quaterly => total_days >= 3 * 30,
                _ => true,
            }
        },
        None => true
    }
}

pub fn get_next_interest_timestamp(
    current_timestamp: DateTime<Utc>,
    maturity_timestamp: DateTime<Utc>, 
    interest_payout: &InterestPayout
) -> Option<DateTime<Utc>> {
    if current_timestamp == maturity_timestamp {
        return Some(maturity_timestamp);
    }
    let interest_timestamp = current_timestamp;
    match interest_payout {
        InterestPayout::Daily => Some(interest_timestamp + Duration::days(1 as i64)),
        InterestPayout::Monthly => Some(interest_timestamp + Duration::days(30 as i64)),
        InterestPayout::Quaterly => Some(interest_timestamp + Duration::days(3 * 30 as i64)),
        InterestPayout::Maturity => Some(maturity_timestamp),
        InterestPayout::Renew => None
    }
}

pub fn get_interest_payout_amount(
    total_interest_paid: f64,
    total_interest_amount: f64,
    interest_payout: &InterestPayout, 
    deposit_tenure: &DepositTenure
) -> Option<f64> {
    let total_days = deposit_tenure.days + deposit_tenure.months * 30 + deposit_tenure.years * 360;
    match interest_payout {
        InterestPayout::Daily => Some(total_interest_amount / total_days as f64),
        InterestPayout::Monthly => {
            let amount = (total_interest_amount / total_days as f64) * 30.0;
            if amount + total_interest_paid > total_interest_amount {Some(total_interest_amount - total_interest_paid)}
            else {Some(amount)}
        },
        InterestPayout::Quaterly => {
            let amount = (total_interest_amount / total_days as f64) * 3.0 * 30.0;
            if amount + total_interest_paid > total_interest_amount {Some(total_interest_amount - total_interest_paid)}
            else {Some(amount)}
        },
        InterestPayout::Maturity => Some(total_interest_amount),
        InterestPayout::Renew => None,
    }
}

pub fn get_total_interest_amount(interest_rate: f64, principal_amount: f64, deposit_tenure: &DepositTenure) -> f64 {
    let total_days = deposit_tenure.days + deposit_tenure.months * 30 + deposit_tenure.years * 360;
    let annual_interest_amount = principal_amount * (interest_rate / 100.0);
    (annual_interest_amount / 360.0) * total_days as f64
}

pub fn get_interest_rate(years: u32) -> f64 {
    match years {
        0 => 5.6,
        1 => 7.5,
        _ => 8.0
    }
}