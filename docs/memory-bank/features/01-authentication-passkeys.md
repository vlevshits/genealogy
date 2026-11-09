# Authentication & Passkeys — Implementation Memory

**Spec:** [docs/features/01-authentication-passkeys.md](../../features/01-authentication-passkeys.md) · **Status:** implemented

## 1. Summary
Passwordless, modern authentication system designed for ease of use by non-technical family members. Supports WebAuthn / FIDO2 Passkeys (biometric TouchID/FaceID or security keys), email magic links, traditional password fallback, one-time invitation links, and emergency recovery codes. Enforces account lockouts and sliding-window rate limits.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261026000003_auth_users_sessions.sql` | `users`, `user_roles`, `sessions`, `passkeys`, `magic_links`, `invitations`, `recovery_codes` |
| Routes | `backend/src/routes/auth/mod.rs`, `backend/src/routes/auth/passkeys_sessions.rs` | Login/register challenge endpoints, magic link request/verify, passkey CRUD, session management (`get_current_user` dead cookie eviction on 401) |
| Service | `backend/src/services/auth/mod.rs` | WebAuthn challenge verification, Argon2/PBKDF2 passwords, token generation, lockout counters |
| Security | `backend/src/services/security/mod.rs` | `RateLimiter` (10 req/min/IP for auth, 3 req/15min for magic links, 5-failure 15min lockout) |
| Client Core | `frontend/src/api/core.ts` | `cleanToken`, `getAuthToken`, `setAuthToken` defensive sanitization, automatic token purge on 401 / `session_expired` |
| Frontend context | `frontend/src/lib/auth-context.tsx` | `AuthProvider`, `useAuth`, `login`, `logout`, `user`, `role`, `isAuthenticated`, token purge on uninitialized/expired session |
| WebAuthn helper | `frontend/src/lib/webauthn.ts` | Base64URL challenge decoding, `navigator.credentials.create`/`get` wrappers |
| Auth UI | `frontend/src/pages/AuthPages.tsx` | Multi-mode card router (`login`, `magic-link`, `recovery-code`, `register-invite`) |
| Components | `frontend/src/components/auth/` | `LoginCard`, `MagicLinkVerifyCard`, `RecoveryCodesCard`, `RegisterInviteCard` |
| Settings UI | `frontend/src/components/settings/` | `SecurityPasskeysPanel.tsx` (manage registered devices, add passkey, revoke session) |
| Tests | `backend/tests/auth_test.rs`, `backend/tests/session_panic_test.rs`, `frontend/tests/api/auth-token-resilience.test.ts` | Passkey flows, magic link invalidation, brute-force lockout, session revocation, 401 cookie eviction, client token sanitization |

## 3. Key Flows
1. **Passkey Login:**
   - User inputs email or selects passkey login → Frontend requests `/auth/login/challenge`.
   - Browser prompts for biometric verification (`navigator.credentials.get`).
   - Frontend sends signed assertion to `/auth/login/verify` → Backend verifies signature and sets `HttpOnly` session cookie.
2. **Magic Link Flow:**
   - User submits email → `/auth/magic-link/request` creates short-lived HMAC token and sends HTML email via `services::email`.
   - Relative clicks link in email (`/auth/magic-link/verify?token=...`) → Backend validates single-use token and issues session cookie.
3. **Invitation Registration:**
   - Admin generates invite link (`/admin/invites`) → Recipient visits `/auth/register?token=...` to set up initial passkey or password and join family tree.

## 4. Invariants & Business Rules
- **Lockout Policy:** 5 consecutive failed login attempts lock the account for 15 minutes.
- **Session Revocation:** Users can view active sessions and revoke "all other sessions" from security settings.
- **Invite Single-Use:** Invitations expire after 7 days and become invalid immediately upon registration.
- **Dead Cookie Eviction (`ADR-013`):** When `GET /api/v1/auth/me` fails with `NotAuthenticated` or `SessionExpired`, the backend returns `Set-Cookie: session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax` (with `; Secure` when HTTPS is active) to actively purge corrupted/expired session cookies.
- **Defensive Token Sanitization:** Frontend storage helpers normalize `"null"`, `"undefined"`, and whitespace strings to `null`, avoiding poisoned `Authorization: Bearer null` headers.

## 5. Decisions
- **`HttpOnly` Cookie Storage (`ADR-003`):** Session tokens are never stored in `localStorage` to protect against XSS token theft.
- **Panics & Dead Cookie Eviction (`ADR-013`):** 500 error envelopes for caught panics via `CatchPanicLayer` and automatic 401 dead cookie eviction.

## 6. Gotchas & Known Limitations
- `GOT-002`: Unauthenticated page load requesting user profile can trigger noisy 401 logs if not silenced.
- `GOT-013`: Stale cookies could block anonymous visitors on public trees before `PublicOrAuth` fallback.
- `GOT-014`: Serialized `"null"` / `"undefined"` in browser storage caused infinite 401 authorization loops until defensive sanitization.

_Last curated: 2026-11-09 (CatchPanicLayer, dead cookie eviction on 401, token sanitization, session resilience)_
