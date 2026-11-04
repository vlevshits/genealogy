# API Security & Operations — Implementation Memory

**Spec:** [docs/features/17-api-security-operations.md](../../features/17-api-security-operations.md) · **Status:** implemented

## 1. Summary
Cross-cutting API security, operations, and infrastructure controls. Enforces uniform REST conventions, standardized error responses with stable machine-readable codes, CSRF/Origin protection on mutations, sliding-window rate limiting, upload safety checks, database advisory locks for sequential migrations, and periodic background maintenance jobs.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Error model | `backend/src/error.rs` | `AppError`, `ErrorResponse`, `ErrorDetail` with machine-readable `code` |
| Security Service | `backend/src/services/security/mod.rs` | `csrf_protection_middleware`, `request_context_middleware`, `RateLimiter` |
| Health & System | `backend/src/routes/system.rs` | `/health`, `/system/info`, database ping |
| Health Handler | `backend/src/routes/health.rs` | Minimal liveness probe |
| Scheduler | `backend/src/services/scheduler.rs` | Tokio background task runner for periodic maintenance and cleanup |
| DB Readiness | `backend/src/services/db_init.rs` | Advisory locking (`ADR-008`), retry loops on database connection |
| State | `backend/src/state.rs` | Shared state container `AppState` |
| Tests | `backend/tests/health_test.rs` | Liveness endpoints, uptime, status codes |
| Operations Tests | `backend/tests/operations_test.rs` | Concurrency locks, rate limit triggers |

## 3. Key Flows
1. **CSRF & Origin Verification:**
   - Any mutating HTTP request (`POST`, `PUT`, `PATCH`, `DELETE`) enters `csrf_protection_middleware`.
   - Checks presence of `X-Requested-With: genealogy`.
   - Checks browser `Origin` matches configured `APP_ORIGIN` (`ADR-003`). Rejects violations with `403 Forbidden` (`csrf_detected`).
2. **Sliding-Window Rate Limiting:**
   - Requests track client IP and endpoint bucket.
   - Enforces 10 req/min/IP on auth, 3 req/15min on magic links, and 60 req/min/IP on anonymous search. Exceeding triggers `429 Too Many Requests` (`rate_limited`).
3. **Database Advisory Lock Upgrade:**
   - On startup, `init_database` calls `SELECT pg_advisory_lock(94218731)`.
   - Applies pending SQLx migrations sequentially. Releases lock on completion (`ADR-008`).

## 4. Invariants & Business Rules
- **No PII or Secrets in Logs:** Passwords, recovery codes, passkey assertions, and personal identification details must never be written to stdout or tracing logs.
- **Max Upload Limit:** File upload endpoints enforce body limits (500MB for media/GEDCOM, 20MB for audio stories) to protect against denial of service.

## 5. Decisions
- **`ADR-003`: Header-Based CSRF Defense:** Combine `SameSite=Lax` session cookies with `X-Requested-With: genealogy` check.
- **`ADR-004`: Uniform Error Envelope:** All non-2xx responses adhere to the standard JSON error schema.

## 6. Gotchas & Known Limitations
- `GOT-005`: Docker reverse proxy forwarding incorrect `Host` or `Origin` headers causing false-positive CSRF rejections.

_Last curated: 2026-11-04 (memory bank bootstrap)_
