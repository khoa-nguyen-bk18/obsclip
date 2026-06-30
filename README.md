# Obsclip

Obsclip is a small menu-bar / system-tray utility that appends your current clipboard (text or image) to today's Obsidian daily note. It reads vault settings from disk and writes files directly — no Obsidian URI and no need to launch Obsidian.

![Obsclip tray menu on macOS](docs/screenshots/tray-menu.png)

## Features

- **One-action clip** — global shortcut or tray menu
- **Text and images** — images are saved to your vault attachment folder and linked with `![[...]]`
- **Obsidian-aware** — reads `obsidian.json`, daily-notes config, and attachment folder from `.obsidian/`
- **Auto vault detection** — uses Obsidian's last-open vault, with optional manual override
- **Tray-only on macOS** — stays in the menu bar, not the Dock
- **Visual feedback** — tray icon turns green on success, red on error

## Requirements

- **Obsidian** with **Daily notes** enabled
- **macOS** or **Windows**

> **Linux** is planned for v1.1. Platform paths are abstracted, but Linux is not supported yet.

## Build from source

Obsclip is not distributed as pre-built installers. Clone the repo, install the prerequisites for your OS, then build locally.

### Prerequisites

Install these before building:

| Tool | macOS | Windows |
|------|-------|---------|
| [Node.js LTS](https://nodejs.org/) | ✅ | ✅ |
| [Rust](https://rustup.rs/) (`rustup`) | ✅ | ✅ |
| Xcode Command Line Tools (`xcode-select --install`) | ✅ | — |
| [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** | — | ✅ |
| [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) | — | ✅ |

See the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) if anything is missing.

#### Windows setup (one time)

1. **Node.js LTS** — download from [nodejs.org](https://nodejs.org/) and install. Restart PowerShell, then check:
   ```powershell
   node -v
   npm -v
   ```
2. **Rust** — download and run [rustup-init.exe](https://rustup.rs/), accept defaults, then restart PowerShell:
   ```powershell
   rustc -V
   cargo -V
   ```
3. **Microsoft C++ Build Tools** — install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/), check **Desktop development with C++**, finish install, restart if prompted.
4. **WebView2 Runtime** — install the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section) if it is not already on your system (Windows 11 usually has it).

Open **PowerShell** or **Developer PowerShell for VS** for the build steps below.

### macOS

```bash
git clone https://github.com/khoa-nguyen-bk18/obsclip.git
cd obsclip
npm install
npm run tauri build -- --bundles dmg
```

**Install:** open the DMG under `src-tauri/target/release/bundle/dmg/` (name includes your version and CPU, e.g. `Obsclip_0.1.0_aarch64.dmg`), drag **Obsclip** to **Applications**, then launch it.

**Apple Silicon + Intel:** on Apple Silicon, the default build is `aarch64`. For a universal binary:

```bash
rustup target add x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin --bundles dmg
```

**Unsigned app:** macOS may block the app because it is not notarized. Either right-click **Obsclip** → **Open** → **Open** again, or run:

```bash
xattr -cr /Applications/Obsclip.app
```

### Windows

From PowerShell:

```powershell
git clone https://github.com/khoa-nguyen-bk18/obsclip.git
cd obsclip
npm install
npm run tauri build -- --bundles msi
```

The first build can take several minutes while Rust compiles dependencies.

**Install:** run the MSI installer:

```
src-tauri\target\release\bundle\msi\Obsclip_0.1.0_x64_en-US.msi
```

(File name includes your version from `src-tauri/tauri.conf.json`.)

**Portable `.exe` installer** (no MSI):

```powershell
npm run tauri build -- --bundles nsis
```

Output: `src-tauri\target\release\bundle\nsis\`.

**Run without installing:** after any release build, the app binary is also at:

```
src-tauri\target\release\obsclip.exe
```

**MSI build fails (`light.exe` / VBSCRIPT):** enable **VBSCRIPT** under **Settings → Apps → Optional features → More Windows features**, then rebuild.

**SmartScreen:** unsigned builds may show “Windows protected your PC”. Click **More info** → **Run anyway**, or sign the installer with your own code signing certificate.

### Development

Run the app with hot reload while you work on it.

macOS / Linux:

```bash
npm install
npm run tauri dev
```

Windows (PowerShell):

```powershell
npm install
npm run tauri dev
```

## Usage

1. Copy text or an image to the clipboard.
2. Press the global shortcut or choose **Clip to daily note** from the tray menu.
3. Obsclip appends to today's daily note (creating it from your template if needed).

### Default shortcut

| Platform | Shortcut |
|----------|----------|
| macOS | `⌘⇧V` |
| Windows | `Ctrl+Shift+V` |

### Tray menu

- **Clip to daily note** — append clipboard to today's note
- **Settings…** — open the settings window
- **Quit** — exit Obsclip

### Clip feedback

After each clip, the tray icon briefly changes color — green for success, red for error (see bottom-right of the tray screenshot above).

## Settings

Open **Settings…** from the tray menu:

![Obsclip settings](docs/screenshots/settings.png)

| Setting | Description |
|---------|-------------|
| **Vault** | Optional folder override, or **Use Obsidian default** to follow Obsidian's active vault |
| **Global shortcut** | Three pickers: primary modifier, extra modifier, and key (with live preview) |
| **Text format** | Timestamped (default), blockquote, bullet, or checkbox |

Click **Save** to apply shortcut and settings changes.

### Example text output (timestamped)

```markdown
- 16:27 — Pasted text from clipboard
```

### Example image output

Image saved to your configured attachment folder (e.g. `attachments/clip-2026-06-29-143052.png`):

```markdown
- 14:32 — ![[clip-2026-06-29-143052.png]]
```

## How vault detection works

Obsclip resolves the vault in this order:

1. Manual path from settings (if set)
2. `last_open` in Obsidian config:
   - macOS: `~/Library/Application Support/obsidian/obsidian.json`
   - Windows: `%APPDATA%\obsidian\obsidian.json`
3. Vault marked `"open": true`
4. Only vault in the list
5. Most recently used vault (`ts`)

## Project structure

```
src-tauri/src/
  clip/          # format, image save, clip orchestration
  clipboard/     # read text/image from OS clipboard
  vault/         # Obsidian config + daily note paths
  tray.rs        # menu bar / tray UI
docs/screenshots/  # README images
```

## Tests

macOS / Linux:

```bash
cd src-tauri && cargo test
```

Windows (PowerShell):

```powershell
cd src-tauri; cargo test
```

Live vault integration test (optional):

```bash
cargo test --test live_clip -- --nocapture
```

## Regenerate screenshots

```bash
# Settings + tray menu mockups
npx playwright screenshot --viewport-size="420,480" \
  file://$PWD/docs/screenshots/settings-mockup.html docs/screenshots/settings.png
npx playwright screenshot --viewport-size="520,220" \
  file://$PWD/docs/screenshots/tray-mockup.html docs/screenshots/tray-menu.png

# Tray icon state PNGs
cd src-tauri && cargo test export_readme_icons -- --ignored --nocapture
```

## Contact

For bug reports, feature requests, or general questions, email [khoa.nguyen.bk18@gmail.com](mailto:khoa.nguyen.bk18@gmail.com).
