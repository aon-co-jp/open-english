; open-english Windowsインストーラー(Inno Setup)。
;
; ユーザー指示「Windows用とAndroidスマホ様にインストーラー付きアプリに
; して、アンインストーラーとバージョン管理機能も付けて」への対応。
;
; 正直な開示: このインストーラーが同梱するのは本リポジトリ(open-english)
; の静的フロントエンド+配信サーバー(server/、RPoemベース)のみ。
; `aruaru-llm`(AI推論本体)は別リポジトリのため同梱しない——
; https://github.com/aon-co-jp/aruaru-llm/releases から別途取得する
; 必要がある(インストール後に表示するREADMEに明記)。
;
; ビルド方法: `server/`で`cargo build --release`を実行した後、
; このディレクトリで`ISCC.exe open-english.iss`を実行する。

#define MyAppName "open-english"
#define MyAppVersion "0.3.0"
#define MyAppPublisher "aon-co-jp"
#define MyAppURL "https://github.com/aon-co-jp/open-english"
#define MyAppExeName "open-english-server.exe"

[Setup]
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
OutputBaseFilename=open-english-setup-{#MyAppVersion}
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
Source: "..\..\icons\*"; DestDir: "{app}\icons"; Flags: ignoreversion recursesubdirs
Source: "README-INSTALLED.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent
Filename: "http://127.0.0.1:4601/"; Flags: shellexec nowait postinstall skipifsilent runasoriginaluser

[UninstallDelete]
; ログ等の生成物が残っていても正常にアンインストールできるよう、
; インストールディレクトリ自体を明示的に削除対象にする。
Type: filesandordirs; Name: "{app}"
