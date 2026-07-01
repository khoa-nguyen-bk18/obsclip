# One-Line Install Design Spec

**Date:** 2026-07-01  
**Status:** Approved  
**Audience:** End users (not developers)  
**Approach:** Platform-native curl install scripts (Approach 2)

## Summary

End users install and launch Obsclip with a single terminal command per platform. Pre-built DMG (macOS) and MSI (Windows) artifacts are published to GitHub Releases via CI. Unsigned builds are acceptable for v1; install scripts handle macOS quarantine and print SmartScreen guidance on Windows.

## Goals

- One command to **install and launch** Obsclip on macOS and Windows
- No Rust, Node, or build-tool prerequisites for end users
- Reinstall over an existing installation without manual cleanup
- Pin to a specific version when needed (`OBSCLIP_VERSION`)

## Non-Goals (v1)

- Code signing / macOS notarization or Windows Authenticode
- Linux install script
- Homebrew, winget, scoop, or other package managers
- In-app auto-update
- Custom install domain (e.g. `obsclip.dev/install`) — use raw GitHub URLs
- Intel macOS DMG unless CI produces it with negligible extra cost (arm64-only is fine for v1)
- Uninstall script

## User-Facing Commands

### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
curl.exe -fsSL https://raw.githubusercontent.com/khoa-nguyen-bk18/obsclip/main/scripts/install.ps1 -o $env:TEMP\obsclip-install.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\obsclip-install.ps1
```

### Pin a version

```bash
OBSCLIP_VERSION=0.1.0 curl -fsSL …/install.sh | bash
```

```powershell
$env:OBSCLIP_VERSION="0.1.0"; curl.exe -fsSL …/install.ps1 -o $env:TEMP\obsclip-install.ps1; powershell -ExecutionPolicy Bypass -File $env:TEMP\obsclip-install.ps1
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Developer pushes tag v*                                     │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  GitHub Actions (release.yml)                                │
│  ├─ macos-latest  → Tauri build → aarch64 DMG               │
│  └─ windows-latest → Tauri build → x64 MSI                  │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  GitHub Releases (versioned assets)                          │
└──────────────────────────┬──────────────────────────────────┘
                           ▼
        ┌──────────────────┴──────────────────┐
        ▼                                      ▼
┌──────────────────┐                 ┌──────────────────┐
│  install.sh      │                 │  install.ps1     │
│  (macOS)         │                 │  (Windows)       │
└────────┬─────────┘                 └────────┬─────────┘
         ▼                                    ▼
  /Applications/Obsclip.app          Program Files or
         │                           %LOCALAPPDATA%\Programs\…
         └──────────────┬─────────────────────┘
                        ▼
                 Launch Obsclip
```

## Release Pipeline

| Piece | Detail |
|-------|--------|
| **Trigger** | Push git tag matching `v*` (e.g. `v0.1.0`) |
| **CI** | GitHub Actions matrix: `macos-latest`, `windows-latest` |
| **macOS artifact** | `aarch64` DMG (Apple Silicon default) |
| **Windows artifact** | MSI `x64` — silent install via `msiexec /quiet` |
| **Version source** | `src-tauri/tauri.conf.json` `version` field |
| **Asset names** | Predictable Tauri output, e.g. `Obsclip_0.1.0_aarch64.dmg`, `Obsclip_0.1.0_x64_en-US.msi` |

Install scripts resolve the **latest** release via GitHub API (`GET /repos/khoa-nguyen-bk18/obsclip/releases/latest`), unless `OBSCLIP_VERSION` is set (then fetch that tag’s release assets).

## Install Scripts

### macOS — `scripts/install.sh`

1. **Guard** — macOS only (`uname`); exit with message on other OS
2. **Resolve version** — GitHub API latest, or `OBSCLIP_VERSION` env var
3. **Detect arch** — `arm64` → `aarch64` DMG; `x86_64` → `x64` DMG if asset exists, else exit with unsupported-arch message
4. **Download** — DMG to `$TMPDIR/obsclip-install/`
5. **Quit if running** — `pkill -x Obsclip` or `osascript` quit before overwrite
6. **Install** — `hdiutil attach` → `cp -R Obsclip.app /Applications/` → `hdiutil detach`
7. **Quarantine** — `xattr -cr /Applications/Obsclip.app`
8. **Launch** — `open -a Obsclip`
9. **Cleanup** — remove temp DMG and mount dir

**Reinstall:** overwrite `/Applications/Obsclip.app`.

### Windows — `scripts/install.ps1`

1. **Guard** — Windows only
2. **Resolve version** — same API / `OBSCLIP_VERSION` logic as macOS
3. **Download** — MSI to `%TEMP%\obsclip-install\`
4. **Install** — `msiexec /i Obsclip_*.msi /quiet /norestart`
5. **Launch** — `Start-Process` on installed `obsclip.exe` (confirm exact path from Tauri MSI output during implementation; fallback search in `%LOCALAPPDATA%\Programs\Obsclip\` and `Program Files\Obsclip\`)
6. **Cleanup** — remove temp MSI
7. **SmartScreen note** — on success, print: *"If Windows blocked the app, click More info → Run anyway."*

**Reinstall:** MSI upgrade over existing install (same `identifier` in `tauri.conf.json`: `com.obsclip.app`).

### Shared UX

| Concern | Behavior |
|---------|----------|
| **Progress** | Short status lines: version, download, install, launch |
| **Failure** | Non-zero exit; clear error message |
| **Admin prompts** | macOS `/Applications` and Windows MSI may prompt for password/UAC — acceptable for v1 |

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Unsupported OS | Exit 1 — supported platforms message |
| Unsupported arch | Exit 1 — no matching release asset |
| GitHub API / download failure | Exit 1 — show URL; suggest retry or `OBSCLIP_VERSION` |
| No matching release asset | Exit 1 — expected filename pattern + link to Releases page |
| macOS: app running | Quit Obsclip, then continue |
| macOS: copy to `/Applications` fails | Exit 1 — permissions or disk space |
| macOS: DMG mount fails | Exit 1 — corrupt download; retry |
| Windows: `msiexec` failure | Exit 1 — suggest non-quiet run or check UAC/SmartScreen |
| Windows: launch path not found | Exit 0 with warning — installed but open from Start Menu manually |
| Install OK, launch failed | Exit 0 with warning |

Scripts use `set -euo pipefail` (bash) and `$ErrorActionPreference = 'Stop'` (PowerShell). No automatic retries in v1.

## Testing

| Layer | What |
|-------|------|
| **CI build smoke** | Release workflow produces DMG + MSI on tag push |
| **shellcheck** | `shellcheck scripts/install.sh` on PR |
| **Manual matrix** | macOS arm64 + Windows x64: fresh install, reinstall, version pin, tray appears after launch |
| **E2E automation** | Deferred — manual checklist in implementation plan |

## Documentation Changes

1. Add **Install (recommended)** section at top of README with both one-liners
2. Demote **Build from source** for contributors
3. Brief unsigned-app note (Gatekeeper / SmartScreen); macOS quarantine handled by script

## Files to Create / Modify

| File | Purpose |
|------|---------|
| `.github/workflows/release.yml` | Tag-triggered build + GitHub Release upload |
| `scripts/install.sh` | macOS install + launch |
| `scripts/install.ps1` | Windows install + launch |
| `.github/workflows/ci.yml` (optional) | shellcheck on PR |
| `README.md` | New install section, demote build-from-source |

## Security Notes

- Scripts are fetched over HTTPS from the project repo; users trust `main` branch content
- Unsigned binaries: macOS Gatekeeper bypass via `xattr -cr` is intentional for v1
- No `curl | bash` on Windows PowerShell without the two-step download + execute pattern (avoids piping untrusted content directly into an interactive shell without a temp file audit point)

## Future Work (post-v1)

- Apple notarization + Windows code signing
- `brew install --cask obsclip` and `winget install`
- Universal macOS DMG
- Custom short URL for install scripts
- In-app update checker
