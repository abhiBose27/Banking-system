# 🏦 Banking Backend System

A distributed microservices-based banking backend, built with Rust, ZeroMQ, Tokio, Actix-Web, Redis and PostgreSQL.
This system powers key banking operations such as customer management, account management, and transaction processing, all connected via a central Controller/Router service.

## Routes
### /client/auth

#### POST /signin

An endpoint that signs up the client to the "user" database using "customer_reference_id" which is generated after creating a customer in the database.

```
Request: {
       "username": String,
       "password": String,
       "customer_reference_id": String<Ulid>
}
```

#### POST /login

An endpoint that logins the client, generating Bearer Token to provide access to service endpoints. The bearer token is stored in the Redis cache database for 5 minutes. 

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

This endpoint is used to logout before the token expires (5 * 60 ttl seconds). The token is deleted from the Redis cache aswell.
### /admin/api

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
                                  ▼         ▼                 │  • Maturity Handling     │
                                                              └──────────────────────────┘
       ┌──────────────────────────┐   ┌─────────────────────────┐
       │     Account Service      │   │   Transaction Service   │
       │  • Add Customer          │   │  • Create Transaction   │
       │  • Create Account        │   │  • Store Transaction    │
       │  • Update Balance        │   │  • Generate Statements  │
       └──────────────────────────┘   └─────────────────────────┘
```

