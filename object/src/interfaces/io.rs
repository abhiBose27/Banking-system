use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::interfaces::{
    account::{Account, AccountResponse, AccountRequest}, 
    customer::{CustomerResponse, CustomerRequest}, 
    deposit::{DepositResponse, DepositRequest}, statement::{StatementResponse, StatementRequest}, 
    transaction::{TransactionResponse, TransactionRequest}
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
    GetStatement { statement_request: StatementRequest },

    CreateAccountResponse { account: AccountResponse },
    CreateCustomerResponse { customer: CustomerResponse },
    CreateTransactionResponse { transaction: TransactionResponse },
    CreateDepositResponse { deposit: DepositResponse },
    GetStatementResponse { statement: Vec<StatementResponse> },

    // Exclusive usage
    GetAccount { account_number: String },
    UpdateBalance { transaction_request: TransactionRequest },

    GetAccountResponse { account: Account },
}