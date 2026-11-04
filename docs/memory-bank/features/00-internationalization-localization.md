# Internationalization & Localization — Implementation Memory

**Spec:** [docs/features/00-internationalization-localization.md](../../features/00-internationalization-localization.md) · **Status:** implemented

## 1. Summary
The application is fully bilingual from day 0, offering complete feature parity between English (`en`) and Russian (`ru`). It provides specialized linguistic handling for Slavic naming traditions, including patronymic (*отчество*) derivation, grammatical gender agreement, and Russian numeric pluralization rules.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Frontend context | `frontend/src/i18n/index.tsx` | `I18nProvider`, `useI18n`, `t()`, `currentLanguage`, `setLanguage` |
| Dictionaries | `frontend/src/i18n/{en,ru}.json` | Synchronized translation keys; parity verified by test |
| Name logic | `frontend/src/lib/patronymic.ts` | `suggestPatronymic(fatherName, gender)`, Slavic declensions |
| Formatting | `frontend/src/lib/formatters.ts` | Localized date formatting, person name formatting |
| Error formatting | `frontend/src/lib/error-formatter.ts` | `getLocalizedErrorMessage(err, t)` maps API codes to UI text |
| Backend patronymic | `backend/src/routes/rbac.rs` | `get_patronymic_suggestion` |
| Backend email | `backend/src/services/email.rs` | Localized transactional HTML email templates |
| Verification | `frontend/tests/i18n/parity.test.ts` | Enforces exact 1:1 key parity and bans icons/emojis in strings |

## 3. Key Flows
1. **Language Switching:** User clicks `LanguageToggleButton` in header → `setLanguage('ru')` updates React context and persists to `localStorage.getItem('preferred_language')`. All UI strings re-render immediately.
2. **Patronymic Suggestion:** In `PersonEditDialog`, when selecting a father and child's gender, `suggestPatronymic` proposes Russian patronymics (e.g. `Иван` + male → `Иванович`, female → `Ивановна`).

## 4. Invariants & Business Rules
- **No Embedded Icons in Strings:** Localization strings in `en.json` and `ru.json` must never contain icon glyphs (`+ `, `← `, `✓ `) or emojis (`GOT-003`).
- **Strict Parity:** Missing a key in either dictionary fails CI (`parity.test.ts`).
- **Pluralization:** Russian plural forms (`one`, `few`, `many`) are selected using Slavic grammatical rules for item counts (e.g. 1 персона, 2 персоны, 5 персон).

## 5. Decisions
- **Client-Side Dictionaries:** Dictionaries are bundled directly into the frontend bundle rather than dynamically fetched, ensuring zero-latency language switching and offline PWA capability.

## 6. Gotchas & Known Limitations
- `GOT-003`: Icon duplication if translation strings include literal symbols.

## 7. Extension Points
- Adding a new language requires creating `frontend/src/i18n/<lang>.json` and registering it in `src/i18n/index.tsx`.

_Last curated: 2026-11-04 (memory bank bootstrap)_
