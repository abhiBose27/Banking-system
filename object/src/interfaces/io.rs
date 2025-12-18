use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::interfaces::{
    account::{Account, AccountRequest, AccountResponse}, 
    customer::{Customer, CustomerRequest, CustomerResponse}, 
    deposit::{DepositClose, DepositRequest, DepositResponse}, statement::{StatementRequest, StatementResponse}, 
    transaction::{TransactionRequest, TransactionResponse}
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    pub data: EventType,
    pub from: Service,
    pub to: Service,
    pub timestamp: DateTime<Utc>
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Service {
    Api,
    Account,
    Transaction,
    Controller,
    Deposit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Ping,
    Request {
        id: Uuid,
        data: DataKind,
    },
    Response {
        id: Uuid,
        success: bool,
        error_message: Option<String>,
        data: Option<DataKind>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataKind {
    CreateAccount { account_request: AccountRequest },
    CreateCustomer { customer_request: CustomerRequest },
    CreateTransaction { transaction_request: TransactionRequest },
    CreateDeposit { deposit_request: DepositRequest },
    CloseDeposit { deposit_close: DepositClose },
    GetStatement { statement_request: StatementRequest },

    CreateAccountResponse { account: AccountResponse },
    CreateCustomerResponse { customer: CustomerResponse },
    CreateTransactionResponse { transaction: TransactionResponse },
    CreateDepositResponse { deposit: DepositResponse },
    CloseDepositResponse,
    GetStatementResponse { statement: Vec<StatementResponse> },

    // Exclusive usage
    GetAccount { account_number: String },
    GetCustomer { first_name: String, last_name: String },
    UpdateBalance { account_number: String, balance: f64 },

    GetAccountResponse { account: Account },
    GetCustomerResponse { customer: Customer }
}