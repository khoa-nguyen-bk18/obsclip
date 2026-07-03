# Image OCR Design

**Date:** 2026-07-03  
**Status:** Approved  
**Platforms:** macOS, Windows (same as Obsclip v1)

## Summary

Add optional OCR to the existing clipboard **image** clip flow. When enabled, Obsclip runs bundled **Tesseract** on the clipped image, extracts text, and appends it indented under the image wikilink in today's daily note. English training data ships with the installer; all other Tesseract languages are available on demand from Settings.

## Goals

- Extract text from clipboard images and append it alongside the saved image link
- Work fully offline after language packs are installed (no cloud APIs)
- Bundle Tesseract statically — no system Tesseract install required
- Ship English (`eng`) in the installer; download any other Tesseract language from Settings
- Let users enable up to **two** languages at a time (combined, e.g. `eng+vie`)
- New Settings toggle for OCR (on by default)
- On OCR failure: still clip the image (green tray), show a brief toast, and surface the error + suggested fix in Settings

## Non-Goals (v1)

- Screen capture / region screenshot (clipboard image flow only)
- OCR on text clips
- Cloud OCR services
- Linux support
- Showing recognized text in the annotation dialog preview
- More than two active languages per clip
- Bundling all language packs in the installer
- OCR quality tuning UI (PSM, DPI, preprocessing sliders)

## User Requirements

| Requirement | Decision |
|-------------|----------|
| Trigger | Existing clipboard image clip (shortcut or tray) |
| OCR toggle | Settings checkbox **Extract text from images**; default **on** |
| Default languages | English only (`eng`) |
| Max active languages | 2, user-selected; Tesseract `lang` string e.g. `eng+vie` |
| Language catalog | Full [tessdata](https://github.com/tesseract-ocr/tessdata) set; searchable in Settings |
| Bundled data | `eng.traineddata` only |
| Other languages | Download on demand; stored in app data |
| Output format | Indented lines under image entry (same as multi-line text clips) |
| No text detected | Image entry only — no placeholder |
| OCR error | Image still clips; green tray; toast + persisted error in Settings with fix hint |
| Annotation dialog | Show `OCR: on` / `OCR: off` only; OCR runs after user confirms clip |
| Text format setting | Does not change image/OCR layout (images always timestamped wikilink + indented OCR) |

### Example output

```markdown
- 14:32 — ![[clip-2026-07-03-143052.png]]
  Meeting notes from whiteboard
  Action item: ship OCR v1
```

With optional user note (`follow up`):

```markdown
- 14:32 — ![[clip-2026-07-03-143052.png]] — follow up
  Meeting notes from whiteboard
  Action item: ship OCR v1
```

## Architecture

### Approach

**Static-link Tesseract** via `kreuzberg-tesseract` (Approach A). App-managed tessdata directory; English copied from app resources on first run.

```
┌──────────────────────────────────────────────────────────────┐
│  Settings UI                                                 │
│  ├─ [x] Extract text from images (OCR)                       │
│  ├─ Languages (max 2): [search box]                        │
│  │    ☑ English (bundled)                                    │
│  │    ☐ Vietnamese  [Download]                               │
│  └─ OCR status banner (error + fix, when present)          │
└──────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼────────────────────────────────┐
│  Clip flow (image)                                           │
│  1. Read clipboard image                                     │
│  2. [If annotation_prompt] Dialog: preview + OCR on/off      │
│  3. Save PNG to vault attachment folder                      │
│  4. If image_ocr && ≥1 language installed: Tesseract OCR     │
│  5. Format entry → append to daily note                      │
│  6. Green tray; on OCR failure → toast + store OcrHealth     │
└──────────────────────────────────────────────────────────────┘
```

### New Rust modules

| Module | Responsibility |
|--------|----------------|
| `ocr/mod.rs` | Public `recognize_text(rgba, w, h, langs) -> OcrResult` |
| `ocr/tesseract.rs` | Tesseract init, RGBA→Leptonica image, run OCR |
| `ocr/languages.rs` | Catalog, install state, download/remove, max-2 validation |
| `ocr/health.rs` | `OcrHealth` state: last error, suggested fix, clear on success |
| `toast.rs` | Brief non-blocking toast window (~3s) for OCR failures |

### Data layout

```
{app_bundle}/Resources/tessdata/eng.traineddata   # macOS (path via Tauri resource API)
{app_bundle}/tessdata/eng.traineddata             # Windows (bundled resource)

{app_data}/obsclip/tessdata/eng.traineddata       # copied on first run if missing
{app_data}/obsclip/tessdata/{code}.traineddata    # downloaded languages
```

`TESSDATA_PREFIX` → `{app_data}/obsclip/tessdata/`

On first OCR use (or app setup), copy bundled `eng.traineddata` into app data if not present.

### Config changes (`AppConfig`)

```rust
#[serde(default = "default_image_ocr")]
pub image_ocr: bool,                    // default true

#[serde(default = "default_ocr_languages")]
pub ocr_languages: Vec<String>,         // default ["eng"], max 2
```

Persisted in existing `config.json`; auto-saved with other settings.

### Clip pipeline changes

`ClipInput` gains `image_ocr: bool` and `ocr_languages: Vec<String>` (from config at call site).

In `run_clip`, image branch:

1. Save PNG (unchanged)
2. If `image_ocr` and `ocr_languages` non-empty:
   - Verify all enabled `.traineddata` files exist; if missing, record `OcrHealth` error and skip OCR
   - Call `recognize_text` with `langs.join("+")`
   - On success with non-empty text → pass to formatter
   - On empty text → image line only
   - On error → image line only + `OcrHealth` + toast
3. `format_image_link_with_ocr(time, filename, ocr_text, annotation)` — user annotation (if any) appends to the image line with ` — `; OCR lines are indented with two spaces on subsequent lines below the image line

`annotation.rs`: extend `AnnotationShowPayload` with `ocr_enabled: bool`. Preview unchanged (image link only).

## Language Management

### Catalog source

Ship a static manifest `tessdata_manifest.json` generated at build time from the [tessdata](https://github.com/tesseract-ocr/tessdata) repo (filename → display name). Used for the searchable Settings list offline.

Download URL per language:

```
https://github.com/tesseract-ocr/tessdata/raw/main/{code}.traineddata
```

Use the standard `tessdata` repo (not `tessdata_best` — too large for casual download).

### Settings UI — Languages section

Inside a new **Image OCR** fieldset (below **Clip**):

| Element | Behavior |
|---------|----------|
| **Extract text from images** | Checkbox; maps to `image_ocr`; auto-save on change |
| **Search** | Filters manifest by display name or code |
| **Language rows** | Checkbox (enable), name + code, status badge |
| **Status badges** | `Bundled` (eng), `Installed`, `Not downloaded` |
| **Download** | Visible when not installed; shows progress; auto-save not required |
| **Remove** | Deletes `{code}.traineddata` from app data; disabled for `eng` |
| **Max 2** | Checking a 3rd language shows status error: *Select at most two languages.* |

English cannot be removed but can be unchecked. If OCR is on and zero languages are enabled, clip image only (same as no-text case) and set `OcrHealth` hint: *Enable at least one OCR language in Settings.*

### Tauri commands

| Command | Purpose |
|---------|---------|
| `get_ocr_languages` | Manifest + installed/enabled state |
| `download_ocr_language` | Download `{code}.traineddata` to app data |
| `remove_ocr_language` | Delete downloaded pack (not `eng`) |
| `get_ocr_health` | Last error + fix for Settings banner |

`save_config` already persists `image_ocr` and `ocr_languages`.

## Error Handling & Feedback

### OCR outcomes

| Outcome | Clip | Tray | Toast | Settings `OcrHealth` |
|---------|------|------|-------|----------------------|
| Success with text | Image + OCR lines | Green | — | Cleared |
| Success, empty text | Image only | Green | — | Cleared |
| Missing language file | Image only | Green | Yes | *Download {lang} language pack* + link action |
| Tesseract init/run error | Image only | Green | Yes | Error message + fix hint |
| OCR on, 0 languages enabled | Image only | Green | — | *Enable at least one language* |
| OCR off | Image only | Green | — | Unchanged |

### Toast

Small borderless Tauri window, bottom-right of primary display, auto-dismiss ~3 seconds:

> OCR failed — image saved. Open Settings for details.

Reuses existing WebView stack; no OS notification permission required.

### Settings error banner

When `OcrHealth` has an error, show a persistent warning panel inside the **Image OCR** fieldset with the message and suggested fix. Clears automatically after the next successful OCR.

Example fixes:

- *Download the Vietnamese language pack using the Download button below.*
- *English language data is missing. Restart Obsclip to restore bundled English, or reinstall.*
- *OCR failed unexpectedly. Try clipping again; if it persists, restart Obsclip.*

## Tesseract Build & Bundle

### Dependency

```toml
kreuzberg-tesseract = { version = "4", default-features = false, features = ["static-linking"] }
```

Static linking compiles Tesseract + Leptonica at build time. Expect longer first CI/local release builds; cache `OUT_DIR` in CI if needed.

### Bundled English

- Add `src-tauri/resources/tessdata/eng.traineddata` (standard tessdata, ~4 MB)
- Register as Tauri `bundle.resources` so it ships in `.app` / MSI
- Copy to `{app_data}/obsclip/tessdata/` on first need

### Image input

Convert clipboard RGBA to format Tesseract accepts (via Leptonica):

- `PIX` from raw RGBA buffer (width × height × 4)
- No preprocessing in v1 beyond default Tesseract behavior

## Files to Modify / Create

| Path | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add `kreuzberg-tesseract`, `reqwest` (blocking, for download) |
| `src-tauri/tauri.conf.json` | Bundle `resources/tessdata/eng.traineddata` |
| `src-tauri/resources/tessdata/eng.traineddata` | Bundled English data |
| `src-tauri/resources/tessdata_manifest.json` | Language catalog |
| `src-tauri/src/ocr/` | New OCR modules |
| `src-tauri/src/toast.rs` | Toast window |
| `src-tauri/src/config.rs` | `image_ocr`, `ocr_languages` |
| `src-tauri/src/clip/service.rs` | OCR integration |
| `src-tauri/src/clip/formatter.rs` | `format_image_link_with_ocr` |
| `src-tauri/src/annotation.rs` | `ocr_enabled` in payload |
| `src-tauri/src/tray.rs` | Pass OCR config; toast on OCR failure |
| `src-tauri/src/lib.rs` | Register commands, manage `OcrHealth` state |
| `src-tauri/src/platform.rs` | `tessdata_dir()` helper |
| `index.html` | Image OCR fieldset |
| `src/main.ts` | OCR settings UI, language list, health banner |
| `src/styles.css` | Language list, banner, toast styles |
| `src/annotation.ts` / `annotation.html` | OCR on/off indicator |
| `src-tauri/tests/formatter.rs` | OCR output formatting tests |
| `src-tauri/tests/ocr_languages.rs` | Max-2 validation, manifest parsing |
| `README.md` | Document OCR feature, language download, size note |

## Testing

### Unit tests (Rust)

- `format_image_link_with_ocr` — single/multi-line OCR, with annotation, empty OCR
- `ocr/languages` — max-2 enforcement, lang string building (`eng+vie`)
- `ocr/languages` — manifest parsing
- Mock tessdata dir fixture for "installed" checks (no live Tesseract in unit tests)

### Integration tests

- `recognize_text` against a small fixture PNG with known text (marked `#[ignore]` for CI optional; runs locally with bundled eng)
- Download command against a temp dir with mocked HTTP (or `#[ignore]` live test)

### Manual verification

1. Fresh install — OCR on, English enabled, clip screenshot with text → indented OCR under image
2. Clip image with no text → image line only
3. Disable OCR → image only; annotation shows `OCR: off`
4. Enable Vietnamese, download pack, select eng+vie, clip mixed text
5. Try enabling 3rd language → blocked with message
6. Simulate missing `vie.traineddata` → image clips, toast, Settings banner with download hint
7. Annotation dialog shows `OCR: on`, preview has no OCR text
8. Windows MSI + macOS DMG builds include Tesseract and eng data

## Size & Performance Notes

- Installer grows ~15–25 MB (static Tesseract) + ~4 MB (`eng.traineddata`)
- Each additional downloaded language ~1–15 MB depending on script
- OCR adds ~0.5–3 s per clip depending on image size; acceptable for tray workflow
- First release build with static Tesseract may take several extra minutes

## Open Questions (resolved)

| Question | Resolution |
|----------|------------|
| OS APIs vs bundled engine | Bundled Tesseract |
| Languages | Full tessdata catalog; max 2 active |
| Default | English only, OCR on |
| Output format | Indented under image line |
| Failure behavior | Graceful + toast + Settings health |
