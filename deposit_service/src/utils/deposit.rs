use chrono::{DateTime, Duration, Utc};

use object::interfaces::deposit::DepositTenure;


pub fn is_valid_deposit_tenure(deposit_tenure: Option<DepositTenure>) -> bool {
    match deposit_tenure {
        Some(d) => !(d.days == 0 && d.months == 0 && d.years == 0),
        None => true
    }
}

pub fn get_premature_withdrawal_amount(
    principal_amount: f64, 
    interest_rate: f64, 
    total_interest_paid: f64,
    creation_timestamp: DateTime<Utc>,
) -> f64 {
    let current_date = Utc::now().date_naive();
    let creation_date = creation_timestamp.date_naive();
    let days_spanned = (current_date - creation_date).num_days();
    let premature_interest = principal_amount * (interest_rate / 100.0) * (days_spanned as f64) / 360.0;
    let difference = premature_interest - total_interest_paid;
    let total_to_pay = principal_amount + difference;
    return total_to_pay;
}

pub fn get_maturity_timestamp(current_timestamp: DateTime<Utc>, deposit_tenure: &DepositTenure) -> DateTime<Utc> {
    let mut maturity_timestamp = current_timestamp;
    if deposit_tenure.years != 0 {
        maturity_timestamp = maturity_timestamp + Duration::days((deposit_tenure.years * 12 * 30) as i64);
    }
    if deposit_tenure.months != 0 {
        maturity_timestamp = maturity_timestamp + Duration::days((deposit_tenure.months * 30) as i64);
    }
    if deposit_tenure.days != 0 {
        maturity_timestamp = maturity_timestamp + Duration::days(deposit_tenure.days as i64);
    }
    maturity_timestamp
}