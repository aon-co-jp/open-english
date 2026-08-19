; open-english Windowsインストーラー(Inno Setup)。
;
; ユーザー指示「Windows用とAndroidスマホ様にインストーラー付きアプリに
; して、アンインストーラーとバージョン管理機能も付けて」への対応。
;
; 正直な開示: このインストーラー自体が同梱するのは本リポジトリ
; (open-english)の静的フロントエンド+配信サーバー(server/、RPoemベース)
; のみ。`aruaru-llm`(AI推論本体、既定同梱)・`aruaru-db`(任意の
; PostgreSQLワイヤ互換ミラーDB)・`open-easy-web`/`open-web-server`
; (いずれも任意、open-englishの動作には機能的に無関係な独立
; エコシステム別製品)はいずれも別リポジトリの実行ファイルのため、
; インストーラー本体には含めず、[Tasks]でチェックされた場合のみ各
; fetch-*.ps1がGitHub Releasesから追加ダウンロードする(2026-08-19、
; open-easy-web/open-web-server取得を追加)。手動で取得する場合は
; それぞれ https://github.com/aon-co-jp/aruaru-llm/releases /
; https://github.com/aon-co-jp/aruaru-db/releases /
; https://github.com/aon-co-jp/open-easy-web/releases /
; https://github.com/aon-co-jp/open-web-server/releases から取得する
; (インストール後に表示するREADMEに明記)。
;
; ビルド方法: `server/`で`cargo build --release`を実行した後、
; このディレクトリで`ISCC.exe open-english.iss`を実行する。

; MyAppVersion: 実際のリリースタグと手動で同期させる必要があったため
; 2026-08-17時点で0.6.3のまま何度もリリースを跨いで取り残されていた
; (ユーザー指摘「install.exeと分かりやすい名前にすべきでは」で発覚した
; 副次的な実バグ)。CI(release.yml)が`/DMyAppVersion=...`でタグの
; バージョン番号を渡すため、それが無い場合(手元でのローカルビルド等)の
; フォールバック値としてのみこの値を使う。
#define MyAppName "open-english"
#ifndef MyAppVersion
  #define MyAppVersion "0.0.0-local-build"
#endif
#define MyAppPublisher "aon-co-jp"
#define MyAppURL "https://github.com/aon-co-jp/open-english"
#define MyAppExeName "open-english-server.exe"

[Setup]
; PrivilegesRequired=lowest: このアプリは管理者権限を必要とする操作
; (システム領域への書き込み・サービス登録等)を一切行わないため、
; UAC昇格プロンプトを不要にする(ユーザー体験の改善、かつ非対話的な
; 自動インストール検証を可能にするための実用上の理由もある)。
PrivilegesRequired=lowest
AppId={{8F2C9B1A-6E44-4B7E-9C10-2C3E7B4A9D01}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
; アンインストーラーはInno Setupが自動生成・レジストリ登録する
; (Windows「アプリと機能」から標準の方法でアンインストール可能)。
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
OutputDir=dist
; 2026-08-17変更(ユーザー指摘「install.exeと分かりやすい名前にすべき
; では」への対応): 従来はバージョン番号入りのファイル名
; (`open-english-setup-{version}.exe`)だったが、(a) バージョン部分が
; 実際のリリースと同期されず何度も取り残されるバグの温床になっていた、
; (b) ユーザーにとって「これがインストーラーだ」と一目で分かる名前では
; なかった。エコシステム全体の命名規則として`<アプリ名>-install.exe`
; (バージョン番号なし、常に同じファイル名)に統一する(ユーザー承認、
; 2026-08-17)——`server/src/self_update.rs`のアセット検出ロジックも
; "setup"ではなく"install"を含むファイル名を探すよう合わせて変更済み。
OutputBaseFilename=open-english-install
ArchitecturesInstallIn64BitMode=x64compatible
DisableProgramGroupPage=yes

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\..\server\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\index.html"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\style.css"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\app.js"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\auto-update.js"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\version.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\manifest.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\exam-prep-questions.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\icons\*"; DestDir: "{app}\icons"; Flags: ignoreversion recursesubdirs
Source: "README-INSTALLED.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "fetch-aruaru-llm.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "fetch-aruaru-db.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "fetch-open-easy-web.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "fetch-open-web-server.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
; 「まとめてインストール」機能(ユーザー指示「他にも必要な関連リポジトリ
; やプロジェクトもインストールする時にまとめてインストールしますか？
; などの機能が欲しい」への対応、2026-08-11)。既定でチェック済み——
; open-englishはaruaru-llm無しでは動作しない(AI応答が返らない)ため。
; 正直な開示: これが取得するのはaruaru-llm本体(実行ファイル)のみ。
; GPT-2/DistilGPT-2の実モデル重み(数百MB〜数GB)は含まない(初回起動後に
; 別途取得が必要)。aruaru-db本体は下記の別タスク(installaruarudb、
; 2026-08-19追加)で任意に同梱可能——ただしこちらも任意機能である旨は
; 変わらない(それぞれ別途セットアップが必要、インストール後に表示する
; README-INSTALLED.txtに手順を明記)。
Name: "installaruarullm"; Description: "Also install aruaru-llm (AI backend) / aruaru-llm(AI応答エンジン)も一緒にインストール"; Flags: checkedonce
; aruaru-db同梱タスク(ユーザー指示「aruaru-dbも同梱して」への対応、
; 2026-08-19)。既定は未チェック——aruaru-llmと違いopen-english本体の
; 動作に必須ではないため(会話履歴はSQLite単体で完結。aruaru-dbは
; `OPEN_ENGLISH_DATABASE_URL`を設定した場合のみ有効になるオプションの
; PostgreSQLワイヤ互換ミラー先)。正直な開示: 取得するのは`aruaru-server`
; (単体で完結するPure Rustバイナリ、PostgreSQLワイヤプロトコルを自前実装
; しており外部PostgreSQL本体のインストールは不要)の実行ファイルのみ。
; Tauri管理GUI・Raft分散クラスタ構成・バックアップ運用等は含まれず、
; 取得後も自動起動はしない(ユーザー自身が`aruaru-server.exe`を起動し
; `OPEN_ENGLISH_DATABASE_URL`を設定する必要がある、手順はfetch-aruaru-db.ps1
; の出力メッセージとREADME-INSTALLED.txtに記載)。
Name: "installaruarudb"; Description: "Also install aruaru-db (optional PostgreSQL-wire mirror DB, not required) / aruaru-db(任意のPostgreSQL互換ミラーDB、必須ではありません)も一緒にインストール"; Flags: unchecked
; open-easy-web/open-web-server同梱タスク(ユーザー指示「open-easy-web も
; open-web-serverもopen-englishの同梱に含めて」への対応、2026-08-19)。
; 既定は両方とも未チェック——正直な開示: これら2つはopen-english自身の
; CLAUDE.md「アーキテクチャ」節に明記の通り、open-englishの実際の動作
; (英会話AI機能)に技術的な依存関係が無い、独立したエコシステム別製品
; (open-easy-web=VPS側アプリ配布・管理ツール、open-web-server=汎用
; リバースプロキシ/Webサーバー)。無関係なサーバーサービスが利用者の
; 意図に反して勝手に追加インストールされることを避けるため、aruaru-db
; と同様に既定オフの任意タスクとした。取得後も自動起動はしない。
Name: "installopeneasyweb"; Description: "Also install open-easy-web (optional, unrelated standalone web-server manager, not required for open-english to work) / open-easy-web(任意、open-englishの動作には無関係な独立Webサーバー管理ツール、必須ではありません)も一緒にインストール"; Flags: unchecked
Name: "installopenwebserver"; Description: "Also install open-web-server (optional, unrelated standalone reverse-proxy/web server, not required for open-english to work) / open-web-server(任意、open-englishの動作には無関係な独立リバースプロキシ/Webサーバー、必須ではありません)も一緒にインストール"; Flags: unchecked

[Run]
Filename: "powershell.exe"; \
    Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\fetch-aruaru-llm.ps1"" -DestDir ""{app}\aruaru-llm"""; \
    StatusMsg: "Downloading aruaru-llm... / aruaru-llmをダウンロード中..."; \
    Flags: runhidden waituntilterminated; \
    Tasks: installaruarullm
Filename: "powershell.exe"; \
    Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\fetch-aruaru-db.ps1"" -DestDir ""{app}\aruaru-db"""; \
    StatusMsg: "Downloading aruaru-db (optional)... / aruaru-db(任意)をダウンロード中..."; \
    Flags: runhidden waituntilterminated; \
    Tasks: installaruarudb
Filename: "powershell.exe"; \
    Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\fetch-open-easy-web.ps1"" -DestDir ""{app}\open-easy-web"""; \
    StatusMsg: "Downloading open-easy-web (optional)... / open-easy-web(任意)をダウンロード中..."; \
    Flags: runhidden waituntilterminated; \
    Tasks: installopeneasyweb
Filename: "powershell.exe"; \
    Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\fetch-open-web-server.ps1"" -DestDir ""{app}\open-web-server"""; \
    StatusMsg: "Downloading open-web-server (optional)... / open-web-server(任意)をダウンロード中..."; \
    Flags: runhidden waituntilterminated; \
    Tasks: installopenwebserver
; 実機E2E検証(2026-08-12)で発覚した実バグ: self_update.rsは
; `/VERYSILENT`でこのインストーラーを起動して無人での自動更新を行う
; ため、サーバー起動エントリに`skipifsilent`が付いていると更新後に
; サーバーが自動起動しない(GUIウィザードでの「起動する」チェックボックス
; 相当の分岐がサイレント時は常にスキップされるため)。ブラウザを開く方は
; 無人更新中にユーザーの意図しないウィンドウが突然開くのを避けるため
; `skipifsilent`を維持する(通常のGUIインストールでは表示、サイレント
; 自動更新では非表示、のままで問題ない)。
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall
Filename: "http://127.0.0.1:4601/"; Flags: shellexec nowait postinstall skipifsilent runasoriginaluser

[UninstallDelete]
; ログ等の生成物が残っていても正常にアンインストールできるよう、
; インストールディレクトリ自体を明示的に削除対象にする。
Type: filesandordirs; Name: "{app}"
