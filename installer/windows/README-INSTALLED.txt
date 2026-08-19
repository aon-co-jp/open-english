open-english をインストールしていただきありがとうございます。

【2026-08-19更新】インストーラーの「aruaru-llmも一緒にインストール」
オプション(既定でオン)を選んだ場合、コマンド操作は一切不要です。
インストール中に aruaru-llm(AI応答エンジン)本体を自動でダウンロード
し、このアプリの起動時(open-english-server.exeの起動時)に aruaru-llm
がまだ動いていなければ自動的に起動します。スタートメニュー/デスクトップ
のショートカットを実行するだけで、ブラウザが http://127.0.0.1:4601/ を
開き、そのままAIとチャットできます。

正直な開示・制約:
・「aruaru-llmも一緒にインストール」を外した場合、aruaru-llm本体は
  取得されず自動起動もできません。その場合は以下から手動で取得し、
  {インストール先}\aruaru-llm\aruaru-llm.exe として配置してから
  このアプリを起動し直してください:
  https://github.com/aon-co-jp/aruaru-llm/releases
・aruaru-llmの実際の応答生成に使うAIモデルの重み(GPT-2系、数百MB)は
  このインストーラーには含まれません。初回起動時に aruaru-llm 側の
  「POST /v1/models/install」、または aruaru-llm 自身のREADMEの手順で
  別途取得してください(未取得の場合、aruaru-llmはその旨を正直に
  返します)。
・GitHubのリリースにWindows向け aruaru-llm アセットが見つからない場合
  (例: 一時的な欠落)、自動取得はスキップされ、上記の手動手順が
  必要になります。

スマートフォン(Android)から使う場合:
Android版アプリ(open-english)は2026-08-11以降、PC不要の単体動作版
です。アプリ内に open-english のサーバーと aruaru-llm の両方を内蔵
して端末上で直接起動するため、PCのIPアドレスを入力する必要は一切
ありません。Google Playではなく直接配布のAPKのため、初回のみ
「提供元不明のアプリ」の許可が必要です。

(参考)PC上のこのサーバーへ、Androidアプリではなく普通のブラウザ
(スマホ・タブレット・別のPC等)で同じWi-Fi内からアクセスしたい場合の
み、そのブラウザで http://(このPCのIPアドレス):4601/ を開いてください
——その場合もAIサーバー接続先(api-base欄)はページを開いたホスト名
から自動的に補完されるため、手入力は基本的に不要です。

【任意】aruaru-db(PostgreSQLワイヤプロトコル互換ミラーDB)について
(2026-08-19追加): インストール時に「aruaru-dbも一緒にインストール」に
チェックを入れた場合、aruaru-server.exe が {app}\aruaru-db\ に取得
されます。ただしこれは完全に任意の機能です——open-english の会話履歴・
設定は SQLite(ローカルファイル)だけで通常通り動作するため、
aruaru-db を使わなくても問題ありません。使う場合は aruaru-server.exe
をご自身で起動し、環境変数 OPEN_ENGLISH_DATABASE_URL に
その接続先(PostgreSQLワイヤプロトコル)を設定してから
open-english-server.exe を起動してください(コマンド操作が必要です)。
Tauri管理GUI・複数ノードのRaftクラスタ構成等は含まれません(単体の
サーバー実行ファイルのみ)。

---

Thank you for installing open-english.

[Updated 2026-08-19] If you chose the installer's "Also install
aruaru-llm" option (checked by default), no command-line steps are
needed at all. The installer downloads the aruaru-llm (AI backend)
binary during setup, and open-english-server.exe automatically
launches it on startup if it isn't already running. Just run the
Start Menu / Desktop shortcut, and your browser will open
http://127.0.0.1:4601/ ready to chat.

Honest disclosure / limitations:
- If you unchecked "Also install aruaru-llm", the binary is neither
  fetched nor auto-launched. In that case, download it manually and
  place it at {install dir}\aruaru-llm\aruaru-llm.exe, then restart
  this app:
  https://github.com/aon-co-jp/aruaru-llm/releases
- The AI model weights aruaru-llm needs for actual response
  generation (GPT-2 family, hundreds of MB) are not bundled by this
  installer. Fetch them separately on first run via aruaru-llm's own
  "POST /v1/models/install", or follow its README (aruaru-llm will
  honestly report if weights are missing).
- If no Windows aruaru-llm release asset is found on GitHub (e.g. a
  temporary gap), the automatic fetch is skipped and the manual steps
  above are required.

To use this from a smartphone (Android):
Since 2026-08-11 the Android app (open-english) is a standalone build
that runs without a PC. It bundles both the open-english server and
aruaru-llm and launches them directly on the device, so entering this
PC's IP address is never required. Since it is distributed as a
direct APK (not via Google Play), you'll need to allow "install from
unknown sources" the first time.

(For reference) If you instead want to reach this PC's server from a
regular mobile/tablet/PC browser on the same Wi-Fi (not the Android
app), open http://(this PC's IP address):4601/ in that browser — the
AI server address field (api-base) there is also auto-filled from the
hostname you used to open the page, so manual entry is normally not
needed either.

[Optional] About aruaru-db (PostgreSQL-wire compatible mirror DB,
added 2026-08-19): If you checked "Also install aruaru-db" during
setup, aruaru-server.exe was downloaded to {app}\aruaru-db\. This is
entirely optional — open-english's conversation history and settings
work fine on SQLite (a local file) alone, so you do not need
aruaru-db. To use it, start aruaru-server.exe yourself and set the
OPEN_ENGLISH_DATABASE_URL environment variable to its endpoint
(PostgreSQL wire protocol) before starting open-english-server.exe
(this is a one-time command-line step). The Tauri admin GUI and
multi-node Raft clustering are not included (single server binary
only).
