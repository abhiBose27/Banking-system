use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::interfaces::{
    account::{Account, AccountDetail, AccountRequest}, 
    customer::{Customer, CustomerDetail, CustomerRequest, CustomerUpdate}, 
    deposit::{DepositDetail, DepositRequest}, 
    statement::{StatementDetail, StatementRequest}, 
    transaction::{TransactionDetail, TransactionRequest}, 
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
    GetAccountDetail { account_number: String },
    GetCustomerDetail { customer_reference_id: Option<Ulid> },
    GetAccountsDetail { customer_reference_id: Option<Ulid> },
    GetDepositsDetail { customer_reference_id: Option<Ulid> },
    GetStatementDetail { account_number: String, statement_request: StatementRequest },
    UpdateCustomer { customer_reference_id: Ulid, customer_update: CustomerUpdate },

    // Private IO
    GetUser { username: String, password: String },
    GetAccount { account_number: String },
    GetCustomer { customer_reference_id: Option<Ulid> },
    UpdateBalance { account_number: String, balance: f64 },

    // Response IO
    GetUserResponse { user: User },
    GetAccountResponse { account: Account },
    GetCustomerResponse { customer: Customer },
    GetAccountDetailResponse { account_detail: AccountDetail },
    GetCustomerDetailResponse { customer_detail: CustomerDetail },
    GetAccountsDetailResponse { accounts_detail: Vec<AccountDetail> },
    GetDepositsDetailResponse { deposits_detail: Vec<DepositDetail> },
    GetStatementDetailResponse { statement_detail: Vec<StatementDetail> },
    CreateAccountResponse { account_detail: AccountDetail },
    CreateCustomerResponse { customer_detail: CustomerDetail },
    CreateDepositResponse { deposit_detail: DepositDetail },
    CreateTransactionResponse { transaction_detail: TransactionDetail },
    CreateUserResponse,
    UpdateBalanceResponse,
    UpdateCustomerResponse,
    CloseDepositResponse,
}