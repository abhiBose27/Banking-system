use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::interfaces::{
    account::{Account, AccountRequest, AccountResponse}, customer::{Customer, CustomerRequest, CustomerResponse}, deposit::{DepositRequest, DepositResponse}, statement::{StatementRequest, StatementResponse}, transaction::{TransactionRequest, TransactionResponse}, user::{User, UserRequest}
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
    Deposit,
    User
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Ping,
    Request {
        id: Uuid,
        customer_id: Option<Uuid>,
        data: DataKind,
    },
    Response {
        id: Uuid,
        success: bool,
        customer_id: Option<Uuid>,
        error_message: Option<String>,
        data: Option<DataKind>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataKind {
    // Creator IO
    CreateAccount { account_request: AccountRequest },
    CreateCustomer { customer_request: CustomerRequest },
    CreateTransaction { transaction_request: TransactionRequest },
    CreateTransactionResponse { transaction: TransactionResponse },
    CreateDeposit { deposit_request: DepositRequest },
    CreateUser { user_request: UserRequest },

    // Get IO
    GetAccount { account_number: String },
    GetCustomer { customer_reference_id: Ulid },
    GetUser { username: String, password: String },
    GetStatement { statement_request: StatementRequest },

    // Update IO
    CloseDeposit { deposit_number: String },
    UpdateBalance { account_number: String, balance: f64 },

    // Response IO
    GetUserResponse { user: User },
    GetAccountResponse { account: Account },
    GetCustomerResponse { customer: Customer },
    GetStatementResponse { statement: Vec<StatementResponse> },
    CreateAccountResponse { account: AccountResponse },
    CreateCustomerResponse { customer: CustomerResponse },
    CreateDepositResponse { deposit: DepositResponse },
    CreateUserResponse,
    CloseDepositResponse,
}