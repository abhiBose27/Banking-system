use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::interfaces::{
    account::{Account, AccountRequest, AccountResponse}, 
    customer::{Customer, CustomerRequest, CustomerResponse, CustomerUpdate}, 
    deposit::{DepositRequest, DepositResponse}, 
    statement::{StatementRequest, StatementResponse}, 
    transaction::{TransactionRequest, TransactionResponse}, 
    user::{User, UserRequest}
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    pub data: EventType,
    pub from: ServiceType,
    pub to: ServiceType,
    pub timestamp: DateTime<Utc>
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
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
        data: DataKind,
        session_customer_id: Option<Uuid>
    },
    Response {
        id: Uuid,
        success: bool,
        session_customer_id: Option<Uuid>,
        error_message: Option<String>,
        data: Option<DataKind>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataKind {
    // Public IO
    CreateUser { user_request: UserRequest },
    CreateDeposit { deposit_request: DepositRequest },
    CreateAccount { account_request: AccountRequest },
    CreateCustomer { customer_request: CustomerRequest },
    CreateTransaction { transaction_request: TransactionRequest },
    CloseDeposit { deposit_number: String },
    GetAccounts { customer_reference_id: Option<Ulid> },
    GetCustomer { customer_reference_id: Option<Ulid> },
    GetAccount { account_number: String },
    GetDeposits { customer_reference_id: Option<Ulid> },
    GetStatement { account_number: String, statement_request: StatementRequest },
    UpdateCustomer { customer_reference_id: Ulid, customer_update: CustomerUpdate },

    // Private IO
    GetUser { username: String, password: String },
    GetCustomerPvt { customer_reference_id: Option<Ulid> },
    GetAccountPvt{ account_number: String },
    UpdateBalance { account_number: String, balance: f64 },

    // Response IO
    GetUserResponse { user: User },
    GetAccountPvtResponse { account: Account },
    GetCustomerPvtResponse { customer: Customer },
    GetAccountResponse { account: AccountResponse },
    GetCustomerResponse { customer: CustomerResponse },
    GetAccountsResponse { accounts: Vec<AccountResponse> },
    GetDepositsResponse { deposits: Vec<DepositResponse> },
    GetStatementResponse { statement: Vec<StatementResponse> },
    CreateAccountResponse { account: AccountResponse },
    CreateCustomerResponse { customer: CustomerResponse },
    CreateDepositResponse { deposit: DepositResponse },
    CreateTransactionResponse { transaction: TransactionResponse },
    CreateUserResponse,
    UpdateBalanceResponse,
    UpdateCustomerResponse,
    CloseDepositResponse,
}