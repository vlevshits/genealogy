# Conventions & Patterns

Proven, recurring architectural and code patterns across the Genealogy codebase.

---

## 1. Backend Patterns (Rust / Axum / SQLx)

### 1.1 Standard Error Envelope & Stable Codes
Every failing request returns the `ErrorResponse` envelope defined in [`backend/src/error.rs`](../../backend/src/error.rs):
```json
{
  "error": {
    "code": "record_not_found",
    "message": "Person with ID ... was not found",
    "details": null,
    "field_errors": null
  }
}
```
- **Rule:** Never return plain text or raw database errors. Use `AppError` variants (`AppError::NotFound`, `AppError::Forbidden`, `AppError::Conflict`, etc.).
- The `code` string is a stable identifier corresponding to a frontend localization key under `errors.*` (e.g. `errors.record_not_found`).

### 1.2 Authentication & Request Context
The middleware `request_context_middleware` in [`backend/src/services/security/context.rs`](../../backend/src/services/security/context.rs) inspects incoming requests and populates `RequestContext`:
```rust
pub async fn some_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    // ...
) -> Result<impl IntoResponse, AppError> {
    if !ctx.has_role(UserRole::Editor) {
        return Err(AppError::Forbidden);
    }
    // ...
}
```

### 1.3 Living-Person Privacy Filter & Privacy-404
In [`backend/src/services/privacy/living.rs`](../../backend/src/services/privacy/living.rs):
```rust
// Privacy check for individual access
if privacy::is_living(&person) && !ctx.is_authenticated() {
    // Return 404 rather than 403 to prevent living-person existence disclosure
    return Err(AppError::NotFound("Person not found".to_string()));
}
```
When retrieving collections or trees, apply `privacy::redact_person(&mut person, &ctx)` to redact birth dates, places, and names into localized "Living" placeholders.

### 1.4 CSRF Protection
Mutating HTTP methods (`POST`, `PUT`, `PATCH`, `DELETE`) are guarded by `csrf_protection_middleware` in [`backend/src/services/security/context.rs`](../../backend/src/services/security/context.rs):
- Must supply header `X-Requested-With: genealogy`.
- Browser `Origin` header must match configured `APP_ORIGIN`.

### 1.5 Optimistic Locking & Change History
Every table subject to concurrency control includes a `version INT NOT NULL DEFAULT 1` column.
Mutating operations query the current version and perform:
```rust
let rows_affected = sqlx::query!(
    "UPDATE persons SET first_name = $1, version = version + 1 WHERE id = $2 AND version = $3",
    first_name, id, expected_version
).execute(&mut *tx).await?.rows_affected();

if rows_affected == 0 {
    return Err(AppError::VersionConflict);
}
```
Inside the same transaction, record the audit entry in `change_log` via [`backend/src/services/history/mod.rs`](../../backend/src/services/history/mod.rs).

### 1.6 Runtime SQLx Queries (No Macros)
To allow reproducible, zero-network Docker and sandbox compilation without a running database, never use `query!` macros that require offline prepared metadata (`sqlx-data.json`).
- Use `sqlx::query("SELECT ... WHERE id = $1").bind(id).fetch_optional(&state.db)`.
- Use `sqlx::query_as::<_, PersonRow>("SELECT ...")` with `#[derive(sqlx::FromRow)]`.

### 1.7 Dual Identifier Lookup (`gid` or `UUID`)
Public endpoints accept either a canonical UUID or a human-friendly alphanumeric genealogical ID (`gid`, e.g. `I0001`):
```rust
let person = if let Ok(uuid) = Uuid::parse_str(&id_or_gid) {
    sqlx::query_as!(Person, "SELECT * FROM persons WHERE id = $1", uuid)
} else {
    sqlx::query_as!(Person, "SELECT * FROM persons WHERE gid = $1", id_or_gid)
};
```

---

## 2. Frontend Patterns (React 19 / TypeScript / Tailwind v4)

### 2.1 Typed API Client & Error Handling
All network calls go through [`frontend/src/api/client.ts`](../../frontend/src/api/client.ts):
- Requests automatically attach `X-Requested-With: genealogy` and send session cookies (`credentials: 'same-origin'`).
- Failed calls reject with an `ApiError` instance containing `status`, `code`, and `details`.
- Use [`frontend/src/lib/error-formatter.ts`](../../frontend/src/lib/error-formatter.ts) to translate errors:
```tsx
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';

try {
  await api.updatePerson(id, data);
} catch (err) {
  const message = getLocalizedErrorMessage(err, t);
  toast.error(message);
}
```

### 2.2 Design System Primitives Only
Never write raw HTML `<button>`, `<input>`, or `<select>` tags in pages or features.
Use components from [`frontend/src/components/ui/`](../../frontend/src/components/ui/):
- `<Button variant="primary" size="md" leftIcon={<Plus className="h-4 w-4" />}>`
- `<TextInput label={t('person.firstName')} value={firstName} onChange={...} />`
- `<Select options={genderOptions} value={gender} onChange={...} />`
- `<Modal isOpen={isOpen} onClose={handleClose} title={t('...')}>`

### 2.3 Styling with Semantic Tokens
Use semantic CSS utility tokens from `frontend/src/index.css`. Never use hardcoded arbitrary hex values (`#fff`, `#1a202c`):
- Backgrounds: `bg-surface`, `bg-canvas`, `bg-subtle`, `bg-surface-elevated`
- Text: `text-primary`, `text-secondary`, `text-muted`
- Borders: `border-subtle`, `border-strong`
- Combine classes using `cn(...)` from [`frontend/src/lib/cn.ts`](../../frontend/src/lib/cn.ts).

### 2.4 Localization (i18n) Rules
- Every user-facing label, placeholder, tooltip, and error message must be passed through `t('namespace.key')`.
- All keys must be present in both [`frontend/src/i18n/en.json`](../../frontend/src/i18n/en.json) and [`frontend/src/i18n/ru.json`](../../frontend/src/i18n/ru.json). Parity is strictly verified by `npm run test:i18n`.
- **Never include icons or emojis in localization strings** (e.g. no `"+ Add"`, `"← Back"`). Icons are rendered solely via React icon components (`<Button leftIcon={<Plus className="h-4 w-4" />}>`).

---

## 3. Testing Patterns

### 3.1 Backend Integration Tests
Tests in `backend/tests/` spin up a test database pool or mock state and exercise handlers via Axum's `oneshot`:
```rust
let app = create_router(test_state);
let response = app
    .oneshot(
        Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

### 3.2 Frontend Unit & Component Tests
Vitest tests in `frontend/tests/components/` use React Testing Library to verify component mounting, role rendering, and accessibility:
- Test bilingual key resolution.
- Test keyboard navigation (Enter, Tab, Escape).
- Test role-based visibility (Editor vs Viewer).

_Last curated: 2026-11-04 (modular paths & Milestone 3.2)_
