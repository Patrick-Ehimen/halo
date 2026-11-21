#!/bin/sh
# Renders each design/canvas/*.dc.html screen to docs/screenshots/<name>.jpg with
# headless Chrome. support.js is a static stand-in for the canvas runtime: it
# evaluates each component's renderVals() once and expands the templates.
#
#   sh scripts/screenshots/render.sh
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
CHROME=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cp "$ROOT"/design/canvas/*.dc.html "$WORK"/
cp "$ROOT"/scripts/screenshots/support.js "$WORK"/
OUT="$ROOT/docs/screenshots"
mkdir -p "$OUT"

# name:height (viewport is 1440 wide, rendered at 2x)
for entry in Main:940 Plan:940 Stats:940 Trading:1310 Blocker:940 Overlays:940 MenuBar:940 Brand:940; do
  name=${entry%%:*}; height=${entry#*:}
  "$CHROME" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 \
    --window-size=1440,"$height" --virtual-time-budget=4000 \
    --screenshot="$WORK/$name.png" "file://$WORK/$name.dc.html" >/dev/null 2>&1
  slug=$(echo "$name" | tr '[:upper:]' '[:lower:]')
  sips -s format jpeg -s formatOptions 85 --resampleWidth 2000 "$WORK/$name.png" --out "$OUT/$slug.jpg" >/dev/null
  echo "docs/screenshots/$slug.jpg"
done
