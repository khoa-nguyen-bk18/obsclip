#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TESSDIR="$ROOT/src-tauri/resources/tessdata"
MANIFEST="$ROOT/src-tauri/resources/tessdata_manifest.json"
mkdir -p "$TESSDIR"
curl -fsSL -o "$TESSDIR/eng.traineddata" \
  "https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata"
python3 - <<'PY'
import json, urllib.request
files = json.load(urllib.request.urlopen(
    "https://api.github.com/repos/tesseract-ocr/tessdata/contents/"
))
langs = []
for item in files:
    name = item["name"]
    if not name.endswith(".traineddata"):
        continue
    code = name.removesuffix(".traineddata")
    langs.append({"code": code, "name": code})
langs.sort(key=lambda x: x["code"])
with open("src-tauri/resources/tessdata_manifest.json", "w") as f:
    json.dump(langs, f, indent=2)
    f.write("\n")
print(f"Wrote {len(langs)} languages")
PY
