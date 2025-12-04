# Re:Earth Flow Release Guide

> 💡 **Quick Start**: Stage from `main` → Release from `release` → Verify on GitHub

---

## Prerequisites Checklist

- [ ] All changes merged to `main` branch
- [ ] All CI tests passing ✅
- [ ] GitHub App credentials configured
- [ ] You have write access to the repository

---

## Determine Version Number

Re:Earth Flow uses **semantic versioning** (semver) with pre-release support.

### Version Format

`Major.Minor.Patch[-prerelease]`

**Examples:**
- Alpha: `0.1.0-alpha.1`, `0.1.0-alpha.2`
- Beta: `0.1.0-beta.1`, `0.1.0-beta.2`
- Stable: `0.1.0`, `0.2.0`, `1.0.0`

### Version Selection Guide

| Release Type | When to Use | Example |
|--------------|-------------|---------|
| **Alpha** | Unstable, testing features | `0.1.0-alpha.1` → `0.1.0-alpha.2` |
| **Beta** | Feature-complete, needs testing | `0.1.0-beta.1` → `0.1.0-beta.2` |
| **Patch** | Bug fixes only | `0.1.0` → `0.1.1` |
| **Minor** | New features (backward compatible) | `0.1.0` → `0.2.0` |
| **Major** | Breaking changes | `0.9.0` → `1.0.0` |

> 💡 For regular releases, increase the **Minor** version

> ⚠️ Don't increase Major version without team discussion

**Check Current Version:**
👉 [CHANGELOG.md](https://github.com/reearth/reearth-flow/blob/main/CHANGELOG.md)

---

## Release Steps

### Step 1: Stage to Release Branch

**Purpose:** Sync latest `main` branch to `release` branch

1. Open: [Stage Workflow](https://github.com/reearth/reearth-flow/actions/workflows/stage.yml)
2. Click **"Run workflow"** button
3. **Branch**: `main` ✅ (should be selected by default)
4. Click **"Run workflow"**
5. Wait for completion (~1 minute)

> ⚠️ **MUST run from `main` branch**

> ✅ This step prepares the release branch with latest changes

---

### Step 2: Create Release

**Purpose:** Generate changelog, create tag, build binaries

1. Open: [Release Workflow](https://github.com/reearth/reearth-flow/actions/workflows/release.yml)
2. Click **"Run workflow"** button
3. **🚨 CRITICAL**: Switch branch to **`release`**

> ⚠️ The dropdown will ONLY show `release` - this is correct!

4. **Select Version Type:**

**For Alpha/Beta (Pre-release):**
   - Version: Select `custom`
   - Custom version: Enter `0.1.0-alpha.2` (no `v` prefix)

**For Stable Release:**
   - Version: Select `patch`, `minor`, or `major`
   - Leave custom version blank

5. Click **"Run workflow"**
6. Wait for completion (~5-10 minutes)

> ⚠️ **DO NOT** run from `main` branch - workflow will fail

> 💡 Enter version WITHOUT the `v` prefix (e.g., `0.1.0-alpha.1`, not `v0.1.0-alpha.1`)

---

### Step 3: Verify Release

**Check these after workflow completes:**

✅ **GitHub Release Created**
- Go to: [Releases Page](https://github.com/reearth/reearth-flow/releases)
- New release should appear with version tag
- Binaries attached (Linux, macOS, Windows)

✅ **CHANGELOG Updated**
- Go to: [CHANGELOG.md](https://github.com/reearth/reearth-flow/blob/main/CHANGELOG.md)
- New version section added at top
- Commits grouped by type (Features, Bug Fixes, etc.)

✅ **Git Tag Created**
- Go to: [Tags](https://github.com/reearth/reearth-flow/tags)
- New tag (e.g., `v0.1.0-alpha.1`) should exist

✅ **Test Binaries** (Optional)
- Download binaries from release page
- Run basic smoke tests

---

## Version Progression Examples

### Alpha → Beta → Stable

```
v0.1.0-alpha.1  ← First alpha (custom: 0.1.0-alpha.1)
v0.1.0-alpha.2  ← Bug fixes (custom: 0.1.0-alpha.2)
v0.1.0-alpha.3  ← More features (custom: 0.1.0-alpha.3)
v0.1.0-beta.1   ← First beta (custom: 0.1.0-beta.1)
v0.1.0-beta.2   ← Testing fixes (custom: 0.1.0-beta.2)
v0.1.0          ← Stable! (custom: 0.1.0)
```

### Stable Release Progression

```
v0.1.0  ← Initial stable (custom: 0.1.0)
v0.1.1  ← Patch: bug fixes (select: patch)
v0.1.2  ← Patch: more fixes (select: patch)
v0.2.0  ← Minor: new features (select: minor)
v0.3.0  ← Minor: more features (select: minor)
v1.0.0  ← Major: breaking changes (select: major)
```

---

## Troubleshooting

### Workflow fails: "must be run from release branch"

**Cause:** Running from `main` instead of `release`

**Solution:**
1. Cancel the current run
2. Start a new run
3. Switch branch dropdown to `release` before clicking "Run workflow"

---

### Error: "GH_APP_ID not found"

**Cause:** GitHub App credentials not configured

**Solution:**

Contact repository administrator to set up:
- `GH_APP_ID` (variable)
- `GH_APP_USER` (variable)
- `GH_APP_PRIVATE_KEY` (secret)

---

### GoReleaser build fails

**Possible Causes:**
- Syntax error in `server/api/go.mod`
- Go version incompatibility
- Missing dependencies

**Solution:**
1. Check workflow logs for specific error
2. Test locally: `cd server/api && go build ./cmd/reearth-flow`
3. Fix issues and re-run workflow

---

### CHANGELOG not generated or empty

**Cause:** No conventional commits since last release

**Solution:**

Ensure commits follow format:
- `feat: add new feature`
- `fix: resolve bug`
- `chore: update dependencies`
- `docs: update readme`

---

### Need to rollback a release

**Steps:**

1. **Delete Git Tag:**

```bash
git tag -d v0.1.0-alpha.1
git push origin :refs/tags/v0.1.0-alpha.1
```

2. **Delete GitHub Release:**
   - Go to [Releases](https://github.com/reearth/reearth-flow/releases)
   - Click on release → Delete

3. **Create New Release:**
   - Start from Step 1 again with correct version

---

## FAQ

### How does this affect deployments?

**Cloud Service (Production):**
- ✅ This process does **NOT** auto-deploy to production
- 🔧 Deployments are separate, manual processes

**Test Environment:**
- ✅ Continues to auto-deploy from `main` branch
- 🔄 Every commit to `main` triggers deployment

---

### Can I release only UI or only Server?

**No.** Re:Earth Flow is a monorepo - all components release together:
- UI (React frontend)
- Server (Go API)
- Engine (Rust runtime)

All use the same version number.

---

### Where can I see release history?

📦 [**GitHub Releases**](https://github.com/reearth/reearth-flow/releases) - Downloads & notes

📝 [**CHANGELOG.md**](https://github.com/reearth/reearth-flow/blob/main/CHANGELOG.md) - Detailed changes

🏷️ [**Tags**](https://github.com/reearth/reearth-flow/tags) - Version tags

---

### What happens when I run the workflow?

**Stage Workflow (`main` branch):**
1. Merges `main` → `release`
2. Pushes to remote
3. Done! (~1 min)

**Release Workflow (`release` branch):**
1. Generates CHANGELOG from commits
2. Updates version in `ui/package.json`
3. Creates git tag (e.g., `v0.1.0-alpha.1`)
4. Pushes tag to GitHub
5. Builds server binaries (Linux, macOS, Windows)
6. Creates GitHub Release with binaries
7. Syncs CHANGELOG to `main` branch
8. Done! (~5-10 min)

---

## Best Practices

### Before Releasing

- [ ] Test all features thoroughly
- [ ] Review and merge all PRs for this release
- [ ] Check CI is passing on `main`
- [ ] Confirm version number with team
- [ ] Prepare release announcement draft

### During Release

- [ ] Run **Stage** from `main` first
- [ ] Run **Release** from `release` only
- [ ] Monitor workflow for errors
- [ ] Don't interrupt the workflow

### After Release

- [ ] Verify release on GitHub
- [ ] Test download links work
- [ ] Update project documentation if needed
- [ ] Announce in team chat/Slack
- [ ] Update roadmap/milestones

---

## Quick Reference

### Workflow Links

| Workflow | Branch | Purpose |
|----------|--------|---------|
| [Stage](https://github.com/reearth/reearth-flow/actions/workflows/stage.yml) | `main` | Sync main → release |
| [Release](https://github.com/reearth/reearth-flow/actions/workflows/release.yml) | `release` | Create release & build |

### Version Input Examples

| Type | Input Field | Example Value |
|------|-------------|---------------|
| Alpha | custom | `0.1.0-alpha.1` |
| Beta | custom | `0.1.0-beta.1` |
| RC | custom | `0.1.0-rc.1` |
| Patch | patch | *(auto)* |
| Minor | minor | *(auto)* |
| Major | major | *(auto)* |

---

*Last Updated: 2025-11-15*

*Maintained by: DevOps Team*
