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
                                  ▼         ▼                 │  • Maturity Handling     │
                                                              └──────────────────────────┘
       ┌──────────────────────────┐   ┌─────────────────────────┐
       │     Account Service      │   │   Transaction Service   │
       │  • Add Customer          │   │  • Create Transaction   │
       │  • Create Account        │   │  • Store Transaction    │
       │  • Update Balance        │   │  • Generate Statements  │
       └──────────────────────────┘   └─────────────────────────┘
```

