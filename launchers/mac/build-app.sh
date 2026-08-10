#!/usr/bin/env bash
# open-english.app (macOSのDock/Launchpadから起動できるアプリバンドル)を
# 作成する。**このスクリプトはmacOS上で実行する必要がある**
# (iconutil/sipsがmacOS専用コマンドのため、Windows/Linux上の開発環境では
# 実行できない — 正直な開示)。open-english自体はサーバー不要の静的
# HTML/JSアプリのため、バンドルの実体は既定のブラウザで index.html を
# 開くだけの薄いシェルランチャー。
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INDEX_HTML="$REPO_ROOT/index.html"
APP_DIR="$REPO_ROOT/launchers/mac/open-english.app"
CONTENTS="$APP_DIR/Contents"
ICONSET_DIR="$REPO_ROOT/launchers/mac/icon.iconset"

if [[ "$(uname)" != "Darwin" ]]; then
  echo "エラー: このスクリプトはmacOS上で実行してください(iconutil/sipsが必要)。" >&2
  exit 1
fi

rm -rf "$APP_DIR" "$ICONSET_DIR"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources" "$ICONSET_DIR"

# .icns を512pxのPNGから生成(sipsで各サイズへリサイズ→iconutilでicns化)。
SRC_PNG="$REPO_ROOT/icons/icon-512.png"
sips -z 16 16     "$SRC_PNG" --out "$ICONSET_DIR/icon_16x16.png" >/dev/null
sips -z 32 32     "$SRC_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null
sips -z 32 32     "$SRC_PNG" --out "$ICONSET_DIR/icon_32x32.png" >/dev/null
sips -z 64 64     "$SRC_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null
sips -z 128 128   "$SRC_PNG" --out "$ICONSET_DIR/icon_128x128.png" >/dev/null
sips -z 256 256   "$SRC_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
sips -z 256 256   "$SRC_PNG" --out "$ICONSET_DIR/icon_256x256.png" >/dev/null
sips -z 512 512   "$SRC_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null
sips -z 512 512   "$SRC_PNG" --out "$ICONSET_DIR/icon_512x512.png" >/dev/null
cp "$SRC_PNG" "$ICONSET_DIR/icon_512x512@2x.png"
iconutil -c icns "$ICONSET_DIR" -o "$CONTENTS/Resources/AppIcon.icns"

cat > "$CONTENTS/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>open-english</string>
  <key>CFBundleDisplayName</key><string>open-english</string>
  <key>CFBundleIdentifier</key><string>tokyo.aon.open-english</string>
  <key>CFBundleVersion</key><string>0.1.0</string>
  <key>CFBundleExecutable</key><string>launch.sh</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>10.13</string>
</dict>
</plist>
EOF

cat > "$CONTENTS/MacOS/launch.sh" <<EOF
#!/usr/bin/env bash
open "$INDEX_HTML"
EOF
chmod +x "$CONTENTS/MacOS/launch.sh"

echo "作成しました: $APP_DIR"
echo "Finderで開き、Applications フォルダやDockへドラッグしてください。"
