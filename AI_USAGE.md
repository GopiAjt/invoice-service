# AI Usage Declaration

## Overview

AI tools were used during the development of this project as an engineering assistant.

The final architecture, implementation decisions, testing strategy, database design, and code validation were reviewed and verified manually before submission.

---

## AI Tools Used

- ChatGPT (OpenAI)

---

## 2. Decisions made independently of AI suggestions

### Decision 1: Avoiding Postgres advisory locks

- **AI suggestion:** Use Postgres advisory locks to prevent race conditions during concurrent updates.
- **What I chose:** I did not implement advisory locks.
- **Why:** The service load expectations are low and the critical sections are already protected by idempotency keys and unique constraints. Adding advisory locks would have increased complexity and potential deadlock risk without clear benefit.

---

### Decision 2: Keeping a flat service structure instead of deep layering

- **AI suggestion:** Introduce multiple abstraction layers (domain services, application services, and separate orchestration layer).
- **What I chose:** A simpler 3-layer structure: handler → service → repository.
- **Why:** Deep layering made the code harder to navigate and unnecessary for the current project scale. I prioritized readability and maintainability over theoretical separation.

---

### Decision 3: Using sqlx over ORM-heavy approaches

- **AI suggestion:** Use a full ORM (e.g., Diesel or SeaORM) for type safety and abstraction.
- **What I chose:** Used sqlx with raw queries and compile-time checked SQL.
- **Why:** I wanted tighter control over query performance and clearer visibility into actual SQL being executed. ORMs would have added abstraction overhead without meaningful benefit for this service.

## How AI Was Used

### Architecture & Design

AI was used to:

- Discuss service architecture ideas
- Review API design approaches
- Validate database schema design
- Evaluate concurrency and idempotency strategies

All architectural decisions were reviewed and implemented manually.

---

### Implementation Assistance

AI was used to:

- Explain Rust concepts and Axum patterns
- Suggest SQLx query structures
- Review transaction handling approaches
- Discuss webhook implementation patterns
- Generate example code snippets

Generated suggestions were manually reviewed, modified, and tested before being added to the codebase.

---

### Documentation

AI was used to assist with:

- README.md drafting
- API documentation formatting
- Design document structure
- This AI usage declaration

All documentation was reviewed and edited before submission.

---

### Testing

AI was used to help design:

- Concurrency test scenarios
- Idempotency test scenarios
- PSP failure simulation tests

Test implementations were written, executed, and verified manually.

---

## Human Verification

Before submission:

- All code was compiled successfully
- Database migrations were executed and verified
- API endpoints were tested manually using curl
- Automated tests were executed successfully
- Concurrency behavior was verified
- Idempotency behavior was verified
- PSP failure handling was verified
- Documentation was reviewed manually

---

## Responsibility Statement

I take full responsibility for all submitted code, documentation, design decisions, and test results.

AI was used as a development assistant and learning tool, but the final implementation, validation, and submission decisions were made by me.
