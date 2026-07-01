# One-Line Install Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan step-by-step. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let end users install and launch Obsclip with one curl command per platform, backed by GitHub Releases (DMG + MSI) and platform install scripts.

**Architecture:** A tag-triggered GitHub Actions workflow (`tauri-action`) builds and uploads release artifacts. `scripts/install.sh` (macOS) and `scripts/install.ps1` (Windows) fetch the latest (or pinned) release from the GitHub API, install silently to the system location, clear macOS quarantine, and launch the app.

**Tech Stack:** GitHub Actions, `tauri-apps/tauri-action@v1`, bash, PowerShell, Tauri 2 bundle outputs (DMG, MSI)

**Spec:** `docs/superpowers/specs/2026-07-01-one-line-install-design.md`

---

## File map

| File | Responsibility |
|------|----------------|
| `.github/workflows/release.yml` | Build DMG (macOS arm64) + MSI (Windows x64) on `v*` tag push; create GitHub Release |
| `.github/workflows/ci.yml` | PR checks: `shellcheck` on `scripts/install.sh` |
| `scripts/install.sh` | macOS download → `/Applications` → `xattr` → launch |
| `scripts/install.ps1` | Windows download → `msiexec` → launch |
| `README.md` | Promote one-liner install; demote build-from-source |

---

### Task 1: Release workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Add release workflow**

Create `.github/workflows/release.yml`:

```yaml
name: release

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
            args: '--target aarch64-apple-darwin --bundles dmg'
          - platform: windows-latest
            args: '--bundles msi'

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin' || '' }}

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: ./src-tauri -> target

      - name: Install frontend dependencies
        run: npm ci

      - name: Build and release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: Obsclip ${{ github.ref_name }}
          releaseBody: |
            See [README](https://github.com/khoa-nguyen-bk18/obsclip#install-recommended) for install instructions.

            **macOS (Apple Silicon):** `.dmg`
            **Windows:** `.msi`

            Builds are unsigned. macOS install script clears quarantine automatically.
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

- [ ] **Step 2: Verify workflow YAML syntax locally**

Run: `python3 -c "import yaml, pathlib; yaml.safe_load(pathlib.Path('.github/workflows/release.yml').read_text()); print('OK')"`

Expected: `OK` (install PyYAML if missing: `pip install pyyaml`, or eyeball the file)

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add tag-triggered release workflow for DMG and MSI"
```

---

### Task 2: shellcheck CI

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Add CI workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: ci

on:
  pull_request:
  push:
    branches:
      - master
      - main

jobs:
  shellcheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run shellcheck
        uses: koalaman/shellcheck-action@v0.9
        with:
          scandir: scripts
          severity: error
```

Note: `scripts/install.sh` does not exist yet — this job will pass on an empty `scripts/` dir until Task 3 lands. That's fine; add the script in the next task on the same branch.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: shellcheck install scripts on PR"
```

---

### Task 3: macOS install script

**Files:**
- Create: `scripts/install.sh`

- [ ] **Step 1: Create `scripts/install.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="khoa-nguyen-bk18/obsclip"
INSTALL_DIR="/Applications/Obsclip.app"
APP_NAME="Obsclip"

info() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

# --- guards ---
[[ "$(uname -s)" == "Darwin" ]] || die "Obsclip install is supported on macOS and Windows only."

arch="$(uname -m)"
case "$arch" in
  arm64)  target="aarch64" ;;
  x86_64) target="x64" ;;
  *) die "unsupported macOS architecture: $arch" ;;
esac

# --- version ---
version="${OBSCLIP_VERSION:-}"
if [[ -n "$version" ]]; then
  api_url="https://api.github.com/repos/${REPO}/releases/tags/v${version}"
else
  api_url="https://api.github.com/repos/${REPO}/releases/latest"
fi

info "resolving release…"
release_json="$(curl -fsSL "$api_url")" || die "failed to fetch release metadata from $api_url"

if [[ -z "$version" ]]; then
  version="$(printf '%s' "$release_json" | grep -m1 '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')"
fi
[[ -n "$version" ]] || die "could not determine release version"

info "installing Obsclip v${version} (${target})"

asset_pattern="Obsclip_${version}_${target}.dmg"
download_url="$(printf '%s' "$release_json" | grep -o "https://github.com[^\"]*${asset_pattern}" | head -1)"
[[ -n "$download_url" ]] || die "no asset matching ${asset_pattern} — see https://github.com/${REPO}/releases"

# --- download ---
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/obsclip-install.XXXXXX")"
cleanup() { rm -rf "$work_dir"; }
trap cleanup EXIT

dmg_path="${work_dir}/${asset_pattern}"
info "downloading ${asset_pattern}…"
curl -fL --progress-bar -o "$dmg_path" "$download_url" || die "download failed: $download_url"

# --- quit running app ---
if pgrep -xq "$APP_NAME" 2>/dev/null; then
  info "quitting running ${APP_NAME}…"
  osascript -e "quit app \"${APP_NAME}\"" || true
  sleep 1
  pkill -x "$APP_NAME" 2>/dev/null || true
fi

# --- install ---
mount_point="$(mktemp -d "${TMPDIR:-/tmp}/obsclip-mount.XXXXXX")"
info "mounting DMG…"
hdiutil attach -nobrowse -readonly -mountpoint "$mount_point" "$dmg_path" >/dev/null

if [[ ! -d "${mount_point}/${APP_NAME}.app" ]]; then
  hdiutil detach "$mount_point" >/dev/null || true
  die "DMG does not contain ${APP_NAME}.app"
fi

info "copying to ${INSTALL_DIR}…"
rm -rf "$INSTALL_DIR"
cp -R "${mount_point}/${APP_NAME}.app" "$INSTALL_DIR"
hdiutil detach "$mount_point" >/dev/null

info "clearing quarantine attributes…"
xattr -cr "$INSTALL_DIR"

# --- launch ---
info "launching ${APP_NAME}…"
open -a "$APP_NAME"

info "done — ${APP_NAME} v${version} is installed and running."
```

- [ ] **Step 2: Make executable**

```bash
chmod +x scripts/install.sh
```

- [ ] **Step 3: Run shellcheck**

Run: `shellcheck scripts/install.sh`

Expected: no errors (install shellcheck via `brew install shellcheck` if needed)

- [ ] **Step 4: Dry-run API resolution (no install)**

Run:

```bash
curl -fsSL "https://api.github.com/repos/khoa-nguyen-bk18/obsclip/releases/latest" | head -5
```

Expected before first release: `404 Not Found` — confirms we need Task 6 (first release) before end-to-end test works.

- [ ] **Step 5: Commit**

```bash
git add scripts/install.sh
git commit -m "feat: add macOS curl install script"
```

---

### Task 4: Windows install script

**Files:**
- Create: `scripts/install.ps1`

- [ ] **Step 1: Create `scripts/install.ps1`**

```powershell
#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

$Repo = 'khoa-nguyen-bk18/obsclip'
$AppName = 'Obsclip'

function Write-Info([string]$Message) {
    Write-Host "==> $Message"
}

function Write-Err([string]$Message) {
    Write-Error $Message
    exit 1
}

if (-not $IsWindows -and $env:OS -ne 'Windows_NT') {
    Write-Err 'Obsclip install is supported on macOS and Windows only.'
}

$version = $env:OBSCLIP_VERSION
if ($version) {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/tags/v$version"
} else {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
}

Write-Info 'resolving release…'
try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ 'User-Agent' = 'obsclip-installer' }
} catch {
    Write-Err "failed to fetch release metadata from $apiUrl"
}

if (-not $version) {
  $version = $release.tag_name -replace '^v', ''
}
if (-not $version) {
    Write-Err 'could not determine release version'
}

Write-Info "installing $AppName v$version (x64)"

$assetName = "Obsclip_${version}_x64_en-US.msi"
$asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
if (-not $asset) {
    Write-Err "no asset matching $assetName — see https://github.com/$Repo/releases"
}

$workDir = Join-Path $env:TEMP 'obsclip-install'
New-Item -ItemType Directory -Force -Path $workDir | Out-Null
$msiPath = Join-Path $workDir $assetName

Write-Info "downloading $assetName…"
try {
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $msiPath -UseBasicParsing
} catch {
    Write-Err "download failed: $($asset.browser_download_url)"
}

Write-Info 'running MSI installer…'
$msiArgs = @('/i', $msiPath, '/quiet', '/norestart')
$proc = Start-Process -FilePath 'msiexec.exe' -ArgumentList $msiArgs -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    Write-Err "msiexec failed with exit code $($proc.ExitCode). Try running without /quiet or check UAC/SmartScreen."
}

Remove-Item -Force $msiPath -ErrorAction SilentlyContinue

$candidates = @(
    (Join-Path $env:ProgramFiles "$AppName\obsclip.exe"),
    (Join-Path ${env:ProgramFiles(x86)} "$AppName\obsclip.exe"),
    (Join-Path $env:LOCALAPPDATA "Programs\$AppName\obsclip.exe")
)

$exePath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($exePath) {
    Write-Info "launching $AppName…"
    try {
        Start-Process -FilePath $exePath
    } catch {
        Write-Warning "installed but could not launch — open $AppName from the Start Menu."
        exit 0
    }
} else {
    Write-Warning 'installed but could not find obsclip.exe — open Obsclip from the Start Menu.'
    exit 0
}

Write-Host ''
Write-Host 'If Windows blocked the app, click More info → Run anyway.'
Write-Info "done — $AppName v$version is installed and running."
```

- [ ] **Step 2: Syntax check**

Run (on macOS during dev — optional; required on Windows before release test):

```powershell
powershell -NoProfile -Command "& { \$null = [System.Management.Automation.Language.Parser]::ParseFile('scripts/install.ps1', [ref]\$null, [ref]\$errors); if (\$errors) { \$errors; exit 1 } else { 'OK' } }"
```

Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add scripts/install.ps1
git commit -m "feat: add Windows curl install script"
```

---

### Task 5: README install section

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add Install section after Features block**

Insert after the Features section (before `## Requirements`):

```markdown
## Install (recommended)

Pre-built releases for macOS (Apple Silicon) and Windows. No Rust or Node required.

### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
curl.exe -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.ps1 -o $env:TEMP\obsclip-install.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\obsclip-install.ps1
```

### Pin a version

```bash
OBSCLIP_VERSION=0.1.0 curl -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.sh | bash
```

```powershell
$env:OBSCLIP_VERSION="0.1.0"; curl.exe -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.ps1 -o $env:TEMP\obsclip-install.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\obsclip-install.ps1
```

> **Unsigned builds:** macOS Gatekeeper may block manually opened apps; the install script clears quarantine automatically. On Windows, SmartScreen may warn on first launch — click **More info** → **Run anyway**.

Requires a [GitHub Release](https://github.com/khoa-nguyen-bk18/obsclip/releases) for your platform. See [Build from source](#build-from-source) if no release is available yet.
```

- [ ] **Step 2: Rename and demote existing build section**

Change `## Build from source` heading text stays the same but add a one-line intro under it:

```markdown
## Build from source

For contributors or if no pre-built release exists for your platform.
```

Remove the duplicate manual DMG drag-and-open install instructions from the macOS build subsection (lines about opening DMG and dragging to Applications) — replace with:

```markdown
**Output:** `src-tauri/target/release/bundle/dmg/Obsclip_<version>_aarch64.dmg`
```

Similarly trim Windows MSI install paragraph to:

```markdown
**Output:** `src-tauri\target\release\bundle\msi\Obsclip_<version>_x64_en-US.msi`
```

Keep unsigned-app troubleshooting (`xattr`, SmartScreen) only under Build from source.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add one-line install instructions to README"
```

---

### Task 6: First release and manual verification

**Files:** none (operational)

- [ ] **Step 1: Ensure repo Actions permissions**

In GitHub → Settings → Actions → General → Workflow permissions: set **Read and write permissions**.

- [ ] **Step 2: Merge install scripts to `main`**

Push branch and merge PR so raw GitHub URLs resolve to the committed scripts.

- [ ] **Step 3: Tag and push release**

```bash
# version in src-tauri/tauri.conf.json should match tag
git tag v0.1.0
git push origin v0.1.0
```

Expected: `release` workflow runs on macOS + Windows (~15–25 min). Draft release appears with DMG + MSI assets.

- [ ] **Step 4: Publish draft release**

In GitHub → Releases → open the draft → verify assets:
- `Obsclip_0.1.0_aarch64.dmg`
- `Obsclip_0.1.0_x64_en-US.msi`

Click **Publish release**.

- [ ] **Step 5: macOS manual test**

On Apple Silicon Mac:

```bash
curl -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.sh | bash
```

Checklist:
- [ ] Script prints version and progress lines
- [ ] `/Applications/Obsclip.app` exists
- [ ] Obsclip launches; tray icon appears
- [ ] Re-run script over existing install succeeds (upgrade path)

Pin test:

```bash
OBSCLIP_VERSION=0.1.0 curl -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.sh | bash
```

- [ ] **Step 6: Windows manual test**

In PowerShell:

```powershell
curl.exe -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/master/scripts/install.ps1 -o $env:TEMP\obsclip-install.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\obsclip-install.ps1
```

Checklist:
- [ ] MSI installs without error
- [ ] Obsclip launches; tray icon appears
- [ ] SmartScreen note printed
- [ ] Re-run succeeds over existing install

- [ ] **Step 7: Record MSI install path (if fallback missed)**

If launch failed but install succeeded, note the actual `obsclip.exe` path and add it to the `$candidates` array in `scripts/install.ps1`.

---

## Spec coverage checklist

| Spec requirement | Task |
|------------------|------|
| GitHub Releases CI on `v*` tag | Task 1 |
| aarch64 DMG + x64 MSI artifacts | Task 1 |
| `install.sh` macOS flow | Task 3 |
| `install.ps1` Windows flow | Task 4 |
| `OBSCLIP_VERSION` pin | Tasks 3, 4, 5 |
| Unsigned + quarantine + SmartScreen note | Tasks 3, 4, 5 |
| Install + launch | Tasks 3, 4 |
| shellcheck CI | Task 2 |
| README install section | Task 5 |
| Manual test matrix | Task 6 |

## Out of scope (confirmed)

Code signing, Linux, package managers, auto-update, custom domain, Intel macOS DMG, uninstall script — not in this plan.
