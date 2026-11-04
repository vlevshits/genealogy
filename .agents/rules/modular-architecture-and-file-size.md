# Modular Architecture & File Size Limits

## 1. File Size Ceilings
- **Recommended File Size:** 200–500 lines.
- **Soft Warning Threshold:** 600 lines.
- **Hard Maximum Ceiling:** 800 lines.

No single source code file in `backend/src/` (`.rs`) or `frontend/src/` (`.ts`, `.tsx`) should exceed 800 lines of code.

## 2. Directory-Module Refactoring Pattern
When an existing file approaches 600–800 lines or accumulates multiple distinct responsibilities:
- **In Rust:** Convert `path/module.rs` to `path/module/mod.rs` and decompose into cohesive submodules. Re-export all public symbols via `pub use submodule::*;` in `mod.rs` to maintain 100% backward compatibility with external callers and `utoipa` macros.
- **In TypeScript:** Convert `path/module.ts` into domain submodules and re-export via `export * from './submodule';` in the barrel file so existing imports continue working without breaking changes.

## 3. Cohesion & Layering Rules
- Handlers in `backend/src/routes/` must remain thin HTTP/RBAC adapters grouped by domain (`tree`, `people`, `families`, `events`, `places`, `media`, `quality`, `history`, `gedcom`, `pdf`, `ai_stories`, `mcp`); business logic belongs in `backend/src/services/`.
- Large React pages in `frontend/src/pages/` must delegate cards, sections, tables, and dialogs to domain sub-components under `frontend/src/components/<domain>/`.

