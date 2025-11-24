use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uuid::Uuid;

use crate::interfaces::{account::AccountRequest, customer::CustomerRequest, deposit::DepositRequest, statement::{Statement, StatementRequest}, transaction::TransactionRequest};

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
        executed: bool,
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
    UpdateBalance { transaction_request: TransactionRequest },
    GetStatement { statement_request: StatementRequest },
    Statement { statement: Vec<Statement> },
    Account { account_number: String },
    Transaction { reference_id: Ulid }
}