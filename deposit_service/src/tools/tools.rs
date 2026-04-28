use chrono::{DateTime, Duration, Months, Utc};
use object::interfaces::deposit::{DepositTenure, InterestPayout};


pub fn is_valid_interest_payout(interest_payout: InterestPayout, deposit_tenure: Option<DepositTenure>) -> bool {
    match deposit_tenure {
        Some(d) => {
            let total_days = d.days + d.months * 30 + d.years * 360;
            match interest_payout {
                InterestPayout::Monthly => total_days >= 30,
                InterestPayout::Quaterly => total_days >= 90,
                _ => true,
            }
        },
        None => true
    }
}

pub fn is_valid_deposit_tenure(deposit_tenure: Option<DepositTenure>) -> bool {
    match deposit_tenure {
        Some(d) => !(d.days == 0 && d.months == 0 && d.years == 0),
        None => true
    }
}

pub fn get_maturity_timestamp(current_timestamp: DateTime<Utc>, deposit_tenure: DepositTenure) -> DateTime<Utc> {
    let mut maturity_timestamp = current_timestamp;
    if deposit_tenure.years != 0 {
        maturity_timestamp = maturity_timestamp + Months::new((deposit_tenure.years * 12) as u32);
    }
    if deposit_tenure.months != 0 {
        maturity_timestamp = maturity_timestamp + Months::new(deposit_tenure.months as u32);
    }
    if deposit_tenure.days != 0 {
        maturity_timestamp = maturity_timestamp + Duration::days(deposit_tenure.days as i64);
    }
    maturity_timestamp
}

pub fn get_next_interest_timestamp(
    current_timestamp: DateTime<Utc>,
    maturity_timestamp: DateTime<Utc>, 
    interest_payout: InterestPayout
) -> Option<DateTime<Utc>> {
    if current_timestamp == maturity_timestamp {
        return Some(maturity_timestamp);
    }
    let interest_timestamp = current_timestamp;
    match interest_payout {
        InterestPayout::Daily => Some(interest_timestamp + Duration::days(1 as i64)),
        InterestPayout::Monthly => Some(interest_timestamp + Months::new(1 as u32)),
        InterestPayout::Quaterly => Some(interest_timestamp + Months::new(3 as u32)),
        InterestPayout::Maturity => Some(maturity_timestamp),
        InterestPayout::Renew => None,
    }
}

pub fn get_total_interest_amount(interest_rate: f64, principal_amount: f64, deposit_tenure: DepositTenure) -> f64 {
    let total_days = deposit_tenure.days + deposit_tenure.months * 30 + deposit_tenure.years * 360;
    let annual_interest_amount = principal_amount * (interest_rate / 100.0);
    (annual_interest_amount / 360.0) * total_days as f64
}

pub fn get_interest_rate(deposit_tenure: DepositTenure) -> f64 {
    match deposit_tenure.years {
        0 => 5.6,
        1 => 7.5,
        _ => 8.0
    }
}