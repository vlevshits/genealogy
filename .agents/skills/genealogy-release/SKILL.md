---
name: genealogy-release
description: Step-by-step workflow for creating a new Genealogy release, bumping versions across frontend and backend, running verification test batteries, and publishing an annotated git tag that triggers the GitHub Actions release workflow.
---

# Genealogy Release Creation Skill

This skill guides the operator or agent through publishing a new official release of the Genealogy application.

## 1. Release Philosophy & Cadence

- **Releases are NOT created on every commit.**
- Day-to-day development commits land on `main` via normal feature/bugfix workflows.
- Releases (`vX.Y.Z`) are cut deliberately when:
  - A roadmap milestone or major feature phase is closed.
  - A critical security patch or bug fix is ready for production rollout.
  - Production LXC or self-hosted instances need an update package.

---

## 2. Pre-Flight Verification Checklist

Before bumping versions or cutting a tag, all automated suites must pass cleanly:

```bash
# 1. Backend format, clippy & tests
cd backend
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib

# 2. Frontend verification & build
cd ../frontend
npm test -- --run
npm run test:i18n
npm run build

# 3. Memory bank integrity check
cd ..
./scripts/check-memory-bank.sh
```

Ensure the working tree is clean (`git status -s`). Any uncommitted functional changes must be committed before the release commit.

---

## 3. Version Bumping Procedure

We adhere to Semantic Versioning (`MAJOR.MINOR.PATCH`):
- `PATCH`: bug fixes, small visual polish, security updates (`0.1.0` -> `0.1.1`).
- `MINOR`: new roadmap features, new tools, new pages (`0.1.0` -> `0.2.0`).
- `MAJOR`: breaking schema/API migrations or paradigm shifts (`1.0.0`).

### Step 3.1: Bump `frontend/package.json`
```bash
cd frontend
npm version <NEW_VERSION> --no-git-tag-version
cd ..
```

### Step 3.2: Bump `backend/Cargo.toml`
Update line 3 of `backend/Cargo.toml`:
```toml
[package]
name = "genealogy-backend"
version = "<NEW_VERSION>"
```
Then verify cargo lockfile:
```bash
cd backend && cargo check && cd ..
```

### Step 3.3: Update Active Context
In `docs/memory-bank/active-context.md`, record the release version under the current milestone.

---

## 4. Release Commit & Annotated Tag

Create an explicit release commit and an annotated tag:

```bash
# 1. Commit the version bump
git add backend/Cargo.toml backend/Cargo.lock frontend/package.json frontend/package-lock.json docs/memory-bank/active-context.md
git commit -m "chore(release): bump version to v<NEW_VERSION>"

# 2. Create the annotated git tag
git tag -a v<NEW_VERSION> -m "Release v<NEW_VERSION>"
```

---

## 5. Publishing the Release

Pushing the annotated tag to GitHub triggers `.github/workflows/release.yml`:

```bash
# Push branch and tag
git push origin main
git push origin v<NEW_VERSION>
```

---

## 6. GitHub Actions Release Verification

1. Navigate to `https://github.com/vlevshits/genealogy/actions/workflows/release.yml`.
2. Ensure the following jobs finish successfully:
   - `build-frontend`: compiles the production web app.
   - `build-backend`: cross-compiles for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.
   - `package-and-publish`: packages unified `.tar.gz` archives and creates the GitHub Release with `SHA256SUMS`.
3. Check the published release at `https://github.com/vlevshits/genealogy/releases/tag/v<NEW_VERSION>`.

---

## 7. Verifying LXC Deployment / Update

To verify or deploy on an LXC container:

```bash
# Fresh install:
curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/install.sh | sudo bash

# Or update existing instance:
sudo /opt/genealogy/scripts/install.sh
```
