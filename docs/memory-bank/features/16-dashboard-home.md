# Dashboard & Home Person — Implementation Memory

**Spec:** [docs/features/16-dashboard-home.md](../../features/16-dashboard-home.md) · **Status:** implemented

## 1. Summary
Personalized landing dashboard, tree statistics, upcoming family anniversaries, and first-run empty-tree onboarding. Allows each user to select their personal "Home Person" (`home_person_id`) which serves as their default focal point for tree views, kinship calculations, and personal metric widgets.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | `home_person_id` column in `users`, onboarding configuration table |
| Routes | `backend/src/routes/rbac.rs` | `/dashboard`, `/anniversaries`, `/stats`, `/onboarding/status`, `/onboarding/start-with-self` |
| Service | `backend/src/services/dashboard.rs` | `get_dashboard_summary`, anniversary calendar calculator |
| Dashboard Page | `frontend/src/pages/DashboardPage.tsx` | Overview cards, upcoming birthdays/anniversaries feed, quick shortcuts |
| Onboarding | `frontend/src/components/onboarding/`| `EmptyTreeOnboardingCard.tsx` (wizard for brand new family trees) |
| Dashboard UI | `frontend/src/components/dashboard/` | Stats counters, anniversary list items, home person profile card |
| Tests | `backend/tests/onboarding_home_person_test.rs`| "Start with self" creation, anniversary rolling window math |
| Frontend Tests | `frontend/tests/components/` | `onboarding-and-home-person.test.tsx` |

## 3. Key Flows
1. **Empty Tree Onboarding ("Start with Self"):**
   - Brand new instance with zero persons displays `EmptyTreeOnboardingCard`.
   - First user provides their own name, gender, and birth date.
   - `POST /api/v1/onboarding/start-with-self` creates the initial person record and links it as the user's `home_person_id`.
2. **Anniversary Calendar Feed:**
   - Client requests `GET /api/v1/anniversaries?days=30`.
   - Backend scans all living and deceased individuals for births, weddings, and deaths within the upcoming 30-day window, adjusted for leap years.

## 4. Invariants & Business Rules
- **Privacy for Anniversaries:** Unauthenticated visitors do not receive birth date notifications for living individuals.
- **Home Person Fallback:** If a user has not designated a Home Person, the system defaults to the root tree ancestor or first created individual.

## 5. Decisions
- **Per-User Home Person:** Home Person is stored per user account rather than globally, so each relative logs in to see themselves at the center of the tree.

## 6. Gotchas & Known Limitations
- Deleting a person who is currently designated as another user's Home Person must reset that user's `home_person_id` to NULL.

_Last curated: 2026-11-04 (memory bank bootstrap)_
