# RBAC & Privacy Access Control — Implementation Memory

**Spec:** [docs/features/02-rbac-access-control.md](../../features/02-rbac-access-control.md) · **Status:** implemented

## 1. Summary
Provides role-based access control (Admin, Editor, Viewer, Public/Anonymous) and living-person privacy protection. The system enforces strict unauthenticated segregation: when public access is disabled, anonymous visitors are cleanly redirected to login. For living individuals, private details are automatically redacted, and direct URL access by unauthorized visitors yields a privacy-preserving 404.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261026000004_rbac_and_privacy.sql` | `user_roles`, `privacy_settings`, `living_override` |
| Route handlers | `backend/src/routes/rbac/mod.rs` | Modular handler package containing RBAC guards for all domain actions |
| Service (RBAC) | `backend/src/services/rbac.rs` | `can_edit_tree`, `can_view_person`, `can_manage_users`, role hierarchies |
| Service (Privacy) | `backend/src/services/privacy/masking.rs` | `redact_person_for_caller`, `redact_person_for_tree_view`, `redact_event_for_tree_view`, `redact_media_for_caller` |
| Service (MCP Redaction) | `backend/src/services/mcp/rpc_read.rs` | `mcp_privacy_access`, `redact_person_for_mcp`, `redact_family_for_mcp` |
| Middleware | `backend/src/services/security/mod.rs` | `request_context_middleware` sets user role in request extensions |
| Frontend components| `frontend/src/components/domain/` | `RoleBadge.tsx`, role-based button hiding |
| Frontend guards | `frontend/src/components/ui/` | `AccessDeniedCard.tsx`, public navigation segregation in `AppShell` |
| Tests | `backend/tests/rbac_test.rs` | Permissions matrix, living-person redaction, 404 on hidden records |

## 3. Key Flows
1. **Living Person Evaluation:**
   - Person record inspected for death event or `is_living` override flag.
   - If no death event exists, algorithm calculates elapsed years from birth date. If > 110 years, marked deceased; otherwise living.
   - If birth date is missing, estimates generation from spouse/child events (`GOT-006`).
2. **Access Denial via Privacy-404:**
   - Unauthorized user navigates to `/persons/I0042` (a living person).
   - Backend evaluates `is_living` and unauthenticated context → returns `404 Not Found` (`ADR-005`).

## 4. Invariants & Business Rules
- **Privacy-404 Invariant (`ADR-005`):** Never return 403 Forbidden for living person URLs when unauthenticated, as this confirms the person exists in the database.
- **Zero Edit Controls for Public:** Public visitors must never see disabled or active edit buttons, merge dialogs, or delete buttons.

## 5. Decisions
- **`ADR-009`: Strict Unauthenticated Segregation:** Clean slate navigation prevents public visitors from seeing administration or editing chrome.

## 6. Gotchas & Known Limitations
- `GOT-006`: Inaccurate living status when ancestral birth dates are completely absent.

_Last curated: 2026-11-07 (Batch 3 modularization, security hardening & staleness audit)_
