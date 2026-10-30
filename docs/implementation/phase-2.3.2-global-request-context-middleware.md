# Phase 2.3.2 Implementation Report: Global Forwarded / RequestContext Middleware

**Date:** 2026-10-28  
**Scope:** Introducing centralized, strongly-typed `RequestContext` middleware across the Axum routing pipeline; eliminating duplicated and fragmented header extraction (`extract_client_meta`, `resolve_effective_rp_id`, `apply_forwarded_proto_secure_cookie`); hardening IP and User-Agent sanitization against injection attacks; ensuring dynamic WebAuthn RP ID resolution and HTTPS `; Secure` cookie issuance work automatically behind reverse proxies.  
**Architect Review Verdict:** APPROVED (Mode 1 & Mode 2)  
**QA Verification Verdict:** PASS (57 backend tests, 186 frontend tests passing)  
**UX Review Verdict:** N/A (Backend-only middleware infrastructure refactoring)

---

## 1. Problem Statement
Previously, transport metadata was parsed ad-hoc across 12 separate auth endpoints and middlewares:
1. `extract_client_meta(&headers)` parsed `X-Forwarded-For` and `User-Agent` without strict `IpAddr` validation, leaving rate-limiter keys and session logs exposed to malformed or injection payloads.
2. `resolve_effective_rp_id(&headers, ...)` was called independently in challenge generation and passkey verification endpoints.
3. Session cookie generation in `AuthService` relied on a post-hoc string replace patch (`apply_forwarded_proto_secure_cookie`) rather than setting the `; Secure` flag at session creation time.
4. CSRF protection middleware re-parsed `Host`, `X-Forwarded-Host`, and `X-Forwarded-Proto` independently on each mutating request.

---

## 2. Implemented Architecture

### 1. `RequestContext` Struct & Sanitization Helpers (`backend/src/services/security.rs`)
- **`RequestContext` Fields:**
  - `client_ip`: Validated IPv4/IPv6 address via `std::net::IpAddr`. First hop of `X-Forwarded-For`, fallback `X-Real-IP`, `ConnectInfo`, and defaulting to `"127.0.0.1"`. Malicious payloads (e.g. `<script>`, SQL injection, non-IP strings) are discarded and fallback safely to `"127.0.0.1"`.
  - `user_agent`: Sanitized string with Unicode control characters (`\r`, `\n`, `\t`, `\0`) removed, capped at 512 characters.
  - `host`: Lowercase effective hostname from `X-Forwarded-Host`, `Host`, or URI authority.
  - `port`: Optional port parsed from authority (including bracketed IPv6 `[::1]:8443`).
  - `is_https`: Evaluates `X-Forwarded-Proto == "https"`, `X-Forwarded-Ssl == "on"`, or URI scheme `https`.
  - `effective_rp_id`: WebAuthn Relying Party ID. Respects `config.rp_id` unless it is `localhost` / `127.0.0.1`, matching incoming effective host or origin.
  - `origin`: Sanitized `Origin` header.
- **`FromRequestParts` & Infallible Middleware:**
  - Implemented `FromRequestParts` with `type Rejection = Infallible` so handlers can extract `Extension(ctx): Extension<RequestContext>` or `ctx: RequestContext` with zero boilerplate.
  - `request_context_middleware` runs at the root router level, populating `RequestContext` into `req.extensions_mut()` before CSRF middleware and route handlers execute.
- **CSRF Integration:**
  - Refactored `csrf_protection_middleware` to use `ctx` and `is_same_origin(origin, &ctx)`, eliminating redundant header parsing.

### 2. Cookie Generation (`backend/src/services/auth.rs`)
- `create_session` now receives `is_https: bool` and generates cookies directly with `; Secure` when `is_https` is true or `app_origin` uses HTTPS.
- Upstream auth entry points (`setup_initial_admin`, `register_with_invitation`, `verify_passkey_login`, `password_login`, `verify_magic_link`, `login_with_recovery_code`) forward `ctx.is_https`.

### 3. Route Handlers Refactoring (`backend/src/routes/auth.rs`)
- Replaced ad-hoc `extract_client_meta` and `resolve_effective_rp_id` calls across all 12 auth endpoints with `Extension(ctx): Extension<RequestContext>`.
- Removed redundant handler-level `apply_forwarded_proto_secure_cookie` calls.

---

## 3. Verification & Evidence
- **Backend Clippy:** `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings` passed with 0 warnings.
- **Backend Tests:** `cargo test --manifest-path backend/Cargo.toml` passed 57/57 tests (including new end-to-end pipeline tests in `auth_test.rs`).
- **Frontend Tests:** `npm --prefix frontend test -- --run` passed 186/186 tests.
- **Frontend Build:** `npm --prefix frontend run build` passed cleanly.

