#!/usr/bin/env bash
# open-english をLinuxのアプリケーションメニュー/デスクトップから起動する
# .desktopファイルをインストールする。open-english自体はサーバー不要の
# 静的HTML/JSアプリのため、既定のブラウザで index.html を直接開く形。
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INDEX_HTML="$REPO_ROOT/index.html"
ICON_PATH="$REPO_ROOT/icons/icon-192.png"
DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
DESKTOP_FILE="$DESKTOP_DIR/open-english.desktop"

mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=open-english
Comment=Maid Cafe English Trainer (Phase 0 prototype)
Exec=xdg-open "$INDEX_HTML"
Icon=$ICON_PATH
Terminal=false
Categories=Education;Network;
EOF

chmod +x "$DESKTOP_FILE"
echo "作成しました: $DESKTOP_FILE"
echo "デスクトップにも置きたい場合は、この .desktop ファイルを ~/Desktop へコピーしてください。"
