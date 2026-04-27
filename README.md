# 🏦 Banking Backend System

A distributed microservices-based banking backend, built with Rust, ZeroMQ, Tokio, Actix-Web, Redis and PostgreSQL.
This system powers key banking operations such as customer management, account management, and transaction processing, all connected via a central Controller/Router service.

## 🧱 Architecture

```
                        ┌────────────────────────── ┐
                        │         API Service       │
                        │ (Actix — external entry)  │
                        └──────────────┬────────────┘
                                       │ ZMQ
                                       ▼
___________________       ┌──────────────────────────┐
|  User Service    |      │     Controller Service   │
|  • SignIn        |_____ │   (ZeroMQ Router/Dealer) │ ◄ ── ─ ┌──────────────────────────┐
|  • Login         |      │ Routes all internal msgs |        |       Deposit Serivce    |
|__________________|      └───────┬─────────┬────────┘        │  • Open / Close Deposit  │
                                  │         │                 |  • Interest Accrual      |
                                  ▼         ▼                 │  • Maturity Handling     |
                                                              │  • Get Deposit detail    |
                                                              └──────────────────────────┘
       ┌──────────────────────────┐   ┌─────────────────────────┐
       │     Account Service      │   │   Transaction Service   │
       │  • Add Customer          │   │  • Create Transaction   │
       │  • Create Account        │   │  • Store Transaction    │
       │  • Update Balance        |   |  • Generate Statement   | 
       |  • Update Customer       |   └─────────────────────────┘
       |  • Get Account detail    |
       |  • Get Customer detail   |        
       └──────────────────────────┘   
```

# Documentation
## Admin Routes
### /admin/api
Requests under this route is authenticated using an API key which is declared as an environment variable. This route is for admin purposes and hence does not require login/logout mechanism.

#### POST /account
Create an account
```
Request: {
    customer_reference_id: String<Ulid>
}

Respone: {
       account_number: String,
       balance: f64,
       creation_timestamp: DateTime<Utc>
}

```

#### GET /account/{account_number}
Get the account details
```
Response {
       account_number: String,
       balance: f64,
       creation_timestamp: Datetime<Utc>
}
```

#### GET /accounts/{customer_reference_id}
Get all the accounts linked to customer reference id
```
Response: [{
       account_number: String,
       balance: f64,
       creation_timestamp: DateTime<Utc> 
}...]

```

#### POST /customer
Create a customer
```
Request: {
       first_name: String,
       last_name: String,
       pan_id: String,
       email_id: String,
       age: i64,
       date_of_birth: NaiveDate,
       contact_number: String
}

Response: {
       customer_reference_id: Ulid,
       first_name: String,
       last_name: String,
       pan_id: String,
       email_id: String,
       age: i64,
       date_of_birth: NaiveDate,
       contact_number: String,
       creation_timestamp: DateTime<Utc>
}
```

#### GET /customer/{customer_reference_id}
Get the customer details linked to customer reference id
```
Response: {
       customer_reference_id: Ulid,
       first_name: String,
       last_name: String,
       pan_id: String,
       email_id: String,
       age: i64,
       date_of_birth: NaiveDate,
       contact_number: String,
       creation_timestamp: DateTime<Utc>
}
```

#### GET /statement/{account_number}
Get the statement of an account
```
Request: {
       from_date: Option<NaiveDate>,
       to_date: Option<NaiveDate>
}

Response: [{
       date: NaiveDate,
       amount: f64,
       reference_id: Ulid,
       from_account_number: Option<String>,
       to_account_number: Option<String>,
       transaction_type: TransactionType,
}....]
```

#### POST /transaction
Make a transaction
```
Request: {
       amount: f64,
       from_account_number: Option<String>,
       to_account_number: Option<String>,
}

Response: {
       reference_id: Ulid,
       amount: f64,
       from_account_number: Option<String>,
       to_account_number: Option<String>,
       transaction_timestamp: DateTime<Utc>
}
```

#### POST /deposit
```
Request: {
       linked_account_number: String,
       principal_amount: f64,
       deposit_tenure: DepositTenure,
       interest_payout: InterestPayout,
       auto_renewal: bool,
       renewed_deposit_tenure: Option<DepositTenure>,
}

Response: {
       deposit_number: String,
       linked_account_number: String,
       principal_amount: f64,
       interest_rate: f64,
       deposit_tenure: DepositTenure,
       interest_payout: InterestPayout,
       total_interest_amount: f64,
       auto_renewal: bool,
       renewed_deposit_tenure: Option<DepositTenure>,
       maturity_date: NaiveDate,
       creation_timestamp: DateTime<Utc>,
}

DepositTenure {
       years: int,
       months: int,
       days: int
}

enum InterestPayout {
       Daily,
       Monthly,
       Quaterly,
       Maturity,
       Renew
}
```

#### DELETE /deposit/{deposit_number}
Close a deposit


#### GET /deposits/{customer_reference_id}
Get all the active deposits linked to a customer reference id

```
Response: [{
       deposit_number: String,
       linked_account_number: String,
       principal_amount: f64,
       interest_rate: f64,
       deposit_tenure: DepositTenure,
       interest_payout: InterestPayout,
       total_interest_amount: f64,
       auto_renewal: bool,
       renewed_deposit_tenure: Option<DepositTenure>,
       maturity_date: NaiveDate,
       creation_timestamp: DateTime<Utc>,
}...]

DepositTenure {
       years: int,
       months: int,
       days: int
}

enum InterestPayout {
       Daily,
       Monthly,
       Quaterly,
       Maturity,
       Renew
}
```

## Client Routes
### /client/auth

#### POST /signin
Sign up client
```
Request: {
       "username": String,
       "password": String,
       "customer_reference_id": String<Ulid>
}
```

#### POST /login
Login in to system as a client.
```
Request {
       "username": String,
       "password": String
}

Response {
       access_token: String,
       token_type: String
}
```

### /client/api
All the requests under this route has a AuthContext header carrying the bearer token which is verified using the middleware.

#### POST /logout
Logout of system as a client.

#### GET /account/{account_number}
Get the account details
```
Response {
       account_number: String,
       balance: f64,
       creation_timestamp: Datetime<Utc>
}
```

#### GET /accounts
Get all the accounts linked to session customer id
```
Response: [{
       account_number: String,
       balance: f64,
       creation_timestamp: DateTime<Utc> 
}...]

```

### GET /customer
Get the customer details linked to session customer id
```
Response: {
       customer_reference_id: Ulid,
       first_name: String,
       last_name: String,
       pan_id: String,
       email_id: String,
       age: i64,
       date_of_birth: NaiveDate,
       contact_number: String,
       creation_timestamp: DateTime<Utc>
}
```

#### GET /statement/{account_number}
Get the statement of an account
```
Request: {
       from_date: Option<NaiveDate>,
       to_date: Option<NaiveDate>
}

Response: [{
       date: NaiveDate,
       amount: f64,
       reference_id: Ulid,
       from_account_number: Option<String>,
       to_account_number: Option<String>,
       transaction_type: TransactionType,
}....]
```

#### POST /transaction
Make a transaction
```
Request: {
       amount: f64,
       from_account_number: Option<String>,
       to_account_number: Option<String>,
}

Response: {
       reference_id: Ulid,
       amount: f64,
       from_account_number: Option<String>,
       to_account_number: Option<String>,
       transaction_timestamp: DateTime<Utc>
}
```

#### POST /deposit
```
Request: {
       linked_account_number: String,
       principal_amount: f64,
       deposit_tenure: DepositTenure,
       interest_payout: InterestPayout,
       auto_renewal: bool,
       renewed_deposit_tenure: Option<DepositTenure>,
}

Response: {
       deposit_number: String,
       linked_account_number: String,
       principal_amount: f64,
       interest_rate: f64,
       deposit_tenure: DepositTenure,
       interest_payout: InterestPayout,
       total_interest_amount: f64,
       auto_renewal: bool,
       renewed_deposit_tenure: Option<DepositTenure>,
       maturity_date: NaiveDate,
       creation_timestamp: DateTime<Utc>,
}

DepositTenure {
       years: int,
       months: int,
       days: int
}

enum InterestPayout {
       Daily,
       Monthly,
       Quaterly,
       Maturity,
       Renew
}
```

#### DELETE /deposit/{deposit_number}
Close a deposit


#### GET /deposits
Get all the active deposits linked to a session customer id

```
Response: [{
       deposit_number: String,
       linked_account_number: String,
       principal_amount: f64,
       interest_rate: f64,
       deposit_tenure: DepositTenure,
       interest_payout: InterestPayout,
       total_interest_amount: f64,
       auto_renewal: bool,
       renewed_deposit_tenure: Option<DepositTenure>,
       maturity_date: NaiveDate,
       creation_timestamp: DateTime<Utc>,
}...]

DepositTenure {
       years: int,
       months: int,
       days: int
}

enum InterestPayout {
       Daily,
       Monthly,
       Quaterly,
       Maturity,
       Renew
}
```