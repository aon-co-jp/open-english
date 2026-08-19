//! open-englishの静的フロントエンドをローカルで配信するRPoemサーバー。
//!
//! 従来`python3 -m http.server`に依存していた(`README.md`/`CLAUDE.md`
//! 記載の既知の制約——file://直接オープンだと一部ブラウザが`fetch()`を
//! ブロックし、`auto-update.js`のポーリングが無効化される)。このバイナリは
//! その代替として、リポジトリ内の静的ファイル群をディスクから配信する
//! (`open_runo_poem_compat::hyper_compat::static_file_handler`、既存実装を
//! そのまま再利用——新規パーサ・新規ロジックは書いていない)。
//!
//! 実行方法: `cargo run --release`(このディレクトリで)。既定で
//! `http://127.0.0.1:4601/`を配信する(`OPEN_ENGLISH_SERVER_BIND`環境変数で
//! 上書き可)。`aruaru-llm`(既定`http://localhost:4600`)とは別ポート。

use open_runo_poem_compat::hyper_compat::static_file_handler;
use open_runo_poem_compat::{get, handler_fn, post, Request, Response, Route, Server, StatusCode, TcpListener};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod db;
mod self_update;

use db::Db;

/// (URLパス, 実ファイル名, Content-Type)。リポジトリ直下に存在する
/// 静的アセットのみを列挙する(推測でパスを増やさない)。
const STATIC_FILES: &[(&str, &str, &str)] = &[
    ("/", "index.html", "text/html; charset=utf-8"),
    ("/index.html", "index.html", "text/html; charset=utf-8"),
    ("/facebook.html", "facebook.html", "text/html; charset=utf-8"),
    ("/style.css", "style.css", "text/css; charset=utf-8"),
    ("/app.js", "app.js", "application/javascript; charset=utf-8"),
    ("/auto-update.js", "auto-update.js", "application/javascript; charset=utf-8"),
    ("/version.json", "version.json", "application/json; charset=utf-8"),
    ("/manifest.json", "manifest.json", "application/manifest+json; charset=utf-8"),
    ("/exam-prep-questions.json", "exam-prep-questions.json", "application/json; charset=utf-8"),
    ("/provider-free-tiers.json", "provider-free-tiers.json", "application/json; charset=utf-8"),
    ("/icons/icon-32.png", "icons/icon-32.png", "image/png"),
    ("/icons/icon-180.png", "icons/icon-180.png", "image/png"),
    ("/icons/icon-192.png", "icons/icon-192.png", "image/png"),
    ("/icons/icon-512.png", "icons/icon-512.png", "image/png"),
    ("/icons/open-english.ico", "icons/open-english.ico", "image/x-icon"),
];

/// 静的ファイルの配信元ディレクトリ。
///
/// **正直な開示・実機テストで発覚した実バグ(2026-08-12)**: 以前は
/// 既定値をビルド時の`CARGO_MANIFEST_DIR`(コンパイルを実行した
/// マシン上のリポジトリパス)の親ディレクトリとしていたが、これは
/// GitHub Actions CIでビルドした配布用バイナリ(Windows installer/
/// Linux・macOS tarball)には全く無意味な値——CIランナー上の
/// `D:\a\open-english\open-english`や`/home/runner/work/...`が
/// バイナリに焼き込まれ、実際にインストールしたユーザーの環境には
/// そのパスは存在しないため、`index.html`等が見つからず**全ページが
/// 404になる**(Windows installer・Linux tarball実機インストールで
/// 実際に再現・確認した——ビルドはCI green、リリースも作れていたが、
/// 配布されたインストーラー自体が実際には機能しない状態だった)。
/// 実際のインストーラー/tarballのレイアウト(Windows Inno Setupの
/// `{app}`、Unix `install.sh`のインストール先)は、いずれも
/// 実行ファイルと`index.html`等を同じディレクトリへ**フラットに**
/// 配置する。このため既定値は「実行ファイルと同じディレクトリ」に
/// 変更し、そこに`index.html`が実在する場合のみ採用する。存在しない
/// 場合(開発機で`cargo run`する場合、`server/target/debug/`直下には
/// `index.html`が無い)は、開発時の便宜として`CARGO_MANIFEST_DIR`の
/// 親ディレクトリ(リポジトリルート)へフォールバックする。
/// `OPEN_ENGLISH_SERVER_ROOT`環境変数が設定されていれば常にそれを
/// 最優先する(Android版向け、2026-08-11追加——アプリの内部ストレージ
/// へ展開した静的ファイル群を指すパスを実行時に渡す必要があるため)。
fn repo_root() -> PathBuf {
    if let Ok(root) = std::env::var("OPEN_ENGLISH_SERVER_ROOT") {
        return PathBuf::from(root);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if exe_dir.join("index.html").exists() {
                return exe_dir.to_path_buf();
            }
        }
    }
    // このバイナリは`open-english/server/`配下でビルドされる前提
    // (Cargo.tomlのpath依存が`../../RPoem/...`である通り)。
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server/ crate must have a parent directory")
        .to_path_buf()
}

/// aruaru-llm(AI応答エンジン)をコマンド操作なしで自動起動する
/// (2026-08-19新設)。
///
/// **発端**: ユーザーがGitHub上で読んだ`README-INSTALLED.txt`には
/// 「aruaru-llmは別途手動でダウンロード・起動が必要」と書かれていたが、
/// 実際には`installer/windows/open-english.iss`の`installaruarullm`
/// タスク(既定オン)が`fetch-aruaru-llm.ps1`でaruaru-llm本体を
/// `{app}\aruaru-llm\`へ**取得済み**だった。矛盾の正体は
/// 「取得はするが起動はしない」——`fetch-aruaru-llm.ps1`はダウンロード・
/// 展開のみでプロセスは一切起動せず、インストーラーの`[Run]`セクションも
/// `open-english-server.exe`しか起動していなかったため、結局ユーザーは
/// 毎回aruaru-llm.exeを手動起動する必要があった(README-INSTALLED.txtの
/// 記述自体は「含まれていません」という部分こそ古いが、「コマンド操作が
/// 必要」という結論は皮肉にも正しかった)。
///
/// この関数は、サーバー起動時に`http://127.0.0.1:4600/healthz`へ
/// 到達できるか確認し、できなければ実行ファイルと同じディレクトリ配下の
/// `aruaru-llm\aruaru-llm.exe`(Windows)/`aruaru-llm/aruaru-llm`
/// (Linux/macOS、`installer/unix/fetch-aruaru-llm.sh`が同じ相対配置で
/// 取得する)を子プロセスとして起動する。**正直な開示**: バイナリが
/// 存在しない(「まとめてインストール」を選ばなかった、または未取得の)
/// 場合は何もしない——エラーにはせず、フロントエンド側の既存の接続状態
/// 表示(`checkHealth`)がその旨を正直にユーザーへ伝える設計に委ねる。
/// 起動した子プロセスはこのサーバープロセスの生存中にデタッチせず
/// 保持する必要は無い(`std::process::Command::spawn`後は`Child`を
/// drop——OS側でプロセス自体は起動したまま存続し続ける、Windows/Unix
/// いずれも同様)。
async fn maybe_launch_aruaru_llm() {
    let aruaru_llm_bind = std::env::var("ARUARU_LLM_BIND").unwrap_or_else(|_| "127.0.0.1:4600".to_string());
    let health_url = format!("http://{aruaru_llm_bind}/healthz");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(700))
        .build()
        .expect("building a minimal reqwest client cannot fail");
    if client.get(&health_url).send().await.is_ok() {
        println!("aruaru-llm already running at http://{aruaru_llm_bind}/ (skip auto-launch)");
        return;
    }

    let Ok(exe) = std::env::current_exe() else { return };
    let Some(exe_dir) = exe.parent() else { return };
    let binary_name = if cfg!(target_os = "windows") { "aruaru-llm.exe" } else { "aruaru-llm" };
    let candidate = exe_dir.join("aruaru-llm").join(binary_name);
    if !candidate.exists() {
        println!(
            "aruaru-llm binary not found at {} - skipping auto-launch (install it via the installer's \
             \"Also install aruaru-llm\" option, or start it manually)",
            candidate.display()
        );
        return;
    }

    match std::process::Command::new(&candidate)
        .env("ARUARU_LLM_BIND", &aruaru_llm_bind)
        .current_dir(&exe_dir.join("aruaru-llm"))
        .spawn()
    {
        Ok(child) => println!(
            "auto-launched aruaru-llm (pid {}) from {} on http://{aruaru_llm_bind}/",
            child.id(),
            candidate.display()
        ),
        Err(e) => println!("failed to auto-launch aruaru-llm from {}: {e}", candidate.display()),
    }
}

/// `self_update.rs`のロールバック用ヘルスチェックからも参照するため
/// `pub(crate)`にする(2026-08-19追加、自動ロールバック機能)。
pub(crate) fn bind_addr() -> SocketAddr {
    std::env::var("OPEN_ENGLISH_SERVER_BIND")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| "127.0.0.1:4601".parse().unwrap())
}

/// `/v1/db/*`のHTTPボディを、RPoemの`Json<T>`(内部は素の`serde_json`)
/// ではなく、Rust-JSON(`../../RS-JSON`)の`from_slice_strict`/
/// `to_vec_strict`経由で読み書きする(2026-08-18、ユーザー指摘「HTTP
/// ボディ処理は対象外とはどういう事?」への対応)。**訂正した理解**:
/// Rust-JSONの`from_slice_strict<T: DeserializeOwned>`/
/// `to_vec_strict<T: Serialize>`はRPC/wire format向けの型付き入出力を
/// 明示的に想定したAPIであり(クレート自身のdoc参照)、埋め込み/静的
/// JSONファイルのパースに限定される理由は無かった——過去のHANDOFFの
/// 「対象外」判断は撤回する。RFC 8259厳密モード(`parse_strict`と同じ
/// 検証)を経由するため、挙動はRPoemの`Json<T>`(`serde_json::from_slice`
/// 直接呼び出し)と等価——検証の通り道をRust-JSON経由に変えただけで、
/// 受理するJSONの範囲自体は変わらない(正直な開示)。
async fn read_rs_json_body<T: serde::de::DeserializeOwned>(req: Request) -> Result<T, Response> {
    use http_body_util::BodyExt;
    let bytes = match req.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return Err(rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "failed to read request body"}))),
    };
    rust_json::from_slice_strict::<T>(&bytes)
        .map_err(|e| rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": format!("invalid JSON body (Rust-JSON strict mode): {e}")})))
}

/// 単純な生存確認エンドポイント(2026-08-19新設、自動ロールバック機能の
/// ヘルスチェック先として`self_update.rs`が使う。既存の`aruaru-llm`側
/// `/healthz`と同じ命名に揃えた)。
async fn healthz() -> Response {
    rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true}))
}

fn rs_json_response(status: StatusCode, value: &impl serde::Serialize) -> Response {
    let body = rust_json::to_vec_strict(value).unwrap_or_else(|_| b"{}".to_vec());
    hyper::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(open_runo_poem_compat::hyper_compat::fixed_body(bytes::Bytes::from(body)))
        .expect("building a response from a fixed set of valid headers cannot fail")
}

#[derive(serde::Deserialize)]
struct AddMessageRequest {
    role: String,
    content: String,
}

async fn db_add_message(req: Request, db: Arc<Db>) -> Response {
    let body: AddMessageRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match db.add_message(&body.role, &body.content) {
        Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

async fn db_list_messages(db: Arc<Db>) -> Response {
    match db.list_messages(500) {
        Ok(msgs) => rs_json_response(StatusCode::OK, &msgs),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

async fn db_clear_messages(db: Arc<Db>) -> Response {
    match db.clear_messages() {
        Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

#[derive(serde::Deserialize)]
struct SetSettingRequest {
    key: String,
    value: String,
}

async fn db_set_setting(req: Request, db: Arc<Db>) -> Response {
    let body: SetSettingRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match db.set_setting(&body.key, &body.value) {
        Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

async fn db_get_settings(db: Arc<Db>) -> Response {
    match db.get_all_settings() {
        Ok(pairs) => {
            let map: std::collections::HashMap<String, String> = pairs.into_iter().collect();
            rs_json_response(StatusCode::OK, &map)
        }
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

/// **正直な開示**: `used_disk_bytes`/`total_disk_bytes`はこのコミット
/// 時点では未実装で常に`null`を返す(クロスプラットフォームのディスク
/// 空き容量取得はOS別実装が必要な別増分——Windows/Linux/macOS/Android
/// それぞれ異なるAPIが必要で、今回のスコープには含めていない)。
/// `db_file_size_bytes`(実際のDBファイルサイズ)・
/// `postgres_mirror_configured`(aruaru-db/PostgreSQLミラーが設定
/// 済みか)は実データを返す。フロントエンド側の円グラフ・保存先選択
/// UIは、この情報が出揃ってから次の増分で実装する。
async fn db_info(db: Arc<Db>) -> Response {
    rs_json_response(
        StatusCode::OK,
        &serde_json::json!({
            "db_path": db.path().display().to_string(),
            "db_file_size_bytes": db.file_size_bytes(),
            "postgres_mirror_configured": db.has_postgres_mirror(),
            "used_disk_bytes": serde_json::Value::Null,
            "total_disk_bytes": serde_json::Value::Null,
        }),
    )
}

/// 保存先変更(ユーザー指示「DATA保存先は、既存の保存先でもそれ以外でも
/// 選択可能にして」への対応、2026-08-18新設)。`new_path`にフルパスを
/// 渡す(例: 増設したマイクロSDのマウント先配下)。
#[derive(serde::Deserialize)]
struct RelocateRequest {
    new_path: String,
}

async fn db_relocate(req: Request, db: Arc<Db>) -> Response {
    let body: RelocateRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match db.relocate(PathBuf::from(&body.new_path)) {
        Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "new_path": db.path().display().to_string()})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

/// rsync同期先を選択して即時バックアップを実行する(ユーザー指示「同期先も
/// RSyncで選択可能にして」への対応、2026-08-18新設)。`destination`は
/// rsyncのCLIがそのまま受理する文字列(ローカルパス・`user@host:/path`の
/// いずれも可)。
#[derive(serde::Deserialize)]
struct RsyncBackupRequest {
    destination: String,
}

/// 「Let's install RSync! / RSyncをインストールしましょう！」の英日併記
/// メッセージ(ユーザー指示、2026-08-18新設)。`rsync`未導入で
/// バックアップに失敗した際、フロントエンドがこの文言+
/// `/v1/db/install-rsync`呼び出しボタンを表示できるよう、
/// `rsync_missing: true`と共に返す。
const INSTALL_RSYNC_PROMPT_EN: &str = "Let's install RSync! Click \"Install RSync\" to set it up automatically, then your backup will run right away.";
const INSTALL_RSYNC_PROMPT_JA: &str = "RSyncをインストールしましょう！「RSyncをインストール」を押すと自動でセットアップし、そのままバックアップを実行します。";

async fn db_rsync_backup(req: Request, db: Arc<Db>) -> Response {
    let body: RsyncBackupRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let destination = body.destination;
    match tokio::task::spawn_blocking(move || db.backup_via_rsync(&destination)).await {
        Ok(Ok(msg)) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "detail": msg})),
        Ok(Err(db::RsyncError::NotInstalled)) => rs_json_response(
            StatusCode::OK,
            &serde_json::json!({
                "ok": false,
                "rsync_missing": true,
                "message_en": INSTALL_RSYNC_PROMPT_EN,
                "message_ja": INSTALL_RSYNC_PROMPT_JA,
            }),
        ),
        Ok(Err(e)) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": format!("rsync task panicked: {e}")})),
    }
}

/// `rsync`を自動インストールし、`retry_destination`が指定されていれば
/// 成功直後にそのままバックアップまで実行する(ユーザー指示「簡単に
/// インストールして簡単に自動で移行する機能を搭載して」への対応、
/// 2026-08-18新設——「インストール」ボタン1回でインストール→
/// バックアップまで通しで完了する設計)。
#[derive(serde::Deserialize)]
struct InstallRsyncRequest {
    #[serde(default)]
    retry_destination: Option<String>,
}

async fn db_install_rsync(req: Request, db: Arc<Db>) -> Response {
    let body: InstallRsyncRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let install_result = tokio::task::spawn_blocking(Db::install_rsync).await;
    let install_msg = match install_result {
        Ok(Ok(msg)) => msg,
        Ok(Err(e)) => {
            return rs_json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &serde_json::json!({
                    "ok": false,
                    "error": e.to_string(),
                    "message_en": "Automatic install failed. Please install rsync manually for your OS (e.g. `winget install cwrsync.cwrsync` on Windows, `apt-get install rsync` on Linux, `brew install rsync` on macOS, `pkg install rsync` on Android/Termux).",
                    "message_ja": "自動インストールに失敗しました。お使いのOS向けに手動でrsyncをインストールしてください(例: Windowsは`winget install cwrsync.cwrsync`、Linuxは`apt-get install rsync`、macOSは`brew install rsync`、Android/Termuxは`pkg install rsync`)。",
                }),
            )
        }
        Err(e) => return rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": format!("install task panicked: {e}")})),
    };
    let Some(destination) = body.retry_destination else {
        return rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "detail": install_msg, "backup_ran": false}));
    };
    match tokio::task::spawn_blocking(move || db.backup_via_rsync(&destination)).await {
        Ok(Ok(backup_msg)) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "detail": install_msg, "backup_ran": true, "backup_detail": backup_msg})),
        Ok(Err(e)) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "detail": install_msg, "backup_ran": false, "backup_error": e.to_string()})),
        Err(e) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "detail": install_msg, "backup_ran": false, "backup_error": format!("rsync task panicked: {e}")})),
    }
}

/// SQLite会話履歴DB + aruaru-db/PostgreSQLミラー(設定されていれば)を
/// **1回の呼び出しで同時に**同じ宛先へrsyncバックアップする(ユーザー
/// 指示「RSyncで、open-englishのaruaru-dbとpostgresqlを他のデバイス
/// などにバックアップ同時を可能に、その設定方法も簡単にして」への
/// 対応、2026-08-19新設)。**設定を簡単にする狙い**: 利用者は宛先を
/// 1箇所入力するだけでよく、SQLite側は`<destination>`直下へ、
/// aruaru-db側は`db_rs::backup_postgres_via_pg_dump`が同じ宛先文字列を
/// そのまま`rsync`へ渡す(pg_dumpしたファイル1個の複製のため、
/// ディレクトリ宛先であれば両者は自動的に別ファイル名で共存する)。
/// aruaru-dbミラー未設定時は`postgres_backup: null`を返す(存在しない
/// 対象を「失敗」として扱わない)。データ一貫性への配慮は`db.rs`の
/// `backup_postgres_via_pg_dump`のdoc参照(稼働中データディレクトリの
/// 直接rsyncではなく`pg_dump`のトランザクション一貫スナップショットを
/// 経由する)。
async fn db_rsync_backup_all(req: Request, db: Arc<Db>) -> Response {
    let body: RsyncBackupRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let destination = body.destination;
    let sqlite_destination = destination.clone();
    let db_for_sqlite = Arc::clone(&db);
    let sqlite_result = tokio::task::spawn_blocking(move || db_for_sqlite.backup_via_rsync(&sqlite_destination)).await;
    let sqlite_json = match sqlite_result {
        Ok(Ok(msg)) => serde_json::json!({"ok": true, "detail": msg}),
        Ok(Err(db::RsyncError::NotInstalled)) => serde_json::json!({
            "ok": false, "rsync_missing": true,
            "message_en": INSTALL_RSYNC_PROMPT_EN, "message_ja": INSTALL_RSYNC_PROMPT_JA,
        }),
        Ok(Err(e)) => serde_json::json!({"ok": false, "error": e.to_string()}),
        Err(e) => serde_json::json!({"ok": false, "error": format!("rsync task panicked: {e}")}),
    };

    let postgres_destination = destination;
    let db_for_pg = Arc::clone(&db);
    let pg_result = tokio::task::spawn_blocking(move || db_for_pg.backup_postgres_via_pg_dump(&postgres_destination)).await;
    let postgres_json = match pg_result {
        Ok(None) => serde_json::Value::Null,
        Ok(Some(Ok(msg))) => serde_json::json!({"ok": true, "detail": msg}),
        Ok(Some(Err(db::RsyncError::NotInstalled))) => serde_json::json!({
            "ok": false, "rsync_or_pg_dump_missing": true,
            "message_en": "Let's install RSync and PostgreSQL client tools (pg_dump)! Please install rsync and the PostgreSQL client package (which provides pg_dump) for your OS, then try again.",
            "message_ja": "RSyncとPostgreSQLクライアントツール(pg_dump)をインストールしましょう！お使いのOS向けにrsyncとPostgreSQLクライアントパッケージ(pg_dumpを含む)をインストールしてから、もう一度お試しください。",
        }),
        Ok(Some(Err(e))) => serde_json::json!({"ok": false, "error": e.to_string()}),
        Err(e) => serde_json::json!({"ok": false, "error": format!("pg_dump/rsync task panicked: {e}")}),
    };

    rs_json_response(StatusCode::OK, &serde_json::json!({"sqlite_backup": sqlite_json, "postgres_backup": postgres_json}))
}

/// 旧形式データの取り込み(ユーザー指示「既存の古い物からDATABASE
/// システムに移動も簡単にする機能」への対応、2026-08-18新設。実際に
/// 存在する旧データ形式は無い〈`db.rs`の`import_legacy`doc参照〉ため、
/// 汎用的な取り込み口として実装)。
#[derive(serde::Deserialize)]
struct LegacyMessage {
    role: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct MigrateLegacyRequest {
    #[serde(default)]
    messages: Vec<LegacyMessage>,
    #[serde(default)]
    settings: std::collections::HashMap<String, String>,
}

async fn db_migrate_legacy(req: Request, db: Arc<Db>) -> Response {
    let body: MigrateLegacyRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let messages: Vec<(String, String)> = body.messages.into_iter().map(|m| (m.role, m.content)).collect();
    let settings: Vec<(String, String)> = body.settings.into_iter().collect();
    match db.import_legacy(&messages, &settings) {
        Ok((n_msg, n_set)) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "imported_messages": n_msg, "imported_settings": n_set})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": e.to_string()})),
    }
}

#[tokio::main]
async fn main() {
    let root = repo_root();
    let db_path = db::db_path(&root);
    let db = Arc::new(Db::open(db_path).expect("failed to open local SQLite DB (data/open-english.sqlite3)"));
    println!(
        "conversation DB: {} (aruaru-db/PostgreSQL mirror: {})",
        db.path().display(),
        if db.has_postgres_mirror() { "enabled via OPEN_ENGLISH_DATABASE_URL" } else { "disabled (SQLite only)" }
    );

    let mut app = Route::new();
    for (url_path, rel_file, content_type) in STATIC_FILES {
        let file_path = root.join(rel_file);
        app = app.at(url_path, get(static_file_handler(file_path, content_type)));
    }
    app = app.at("/healthz", get(handler_fn(move |_req, _p| async move { healthz().await })));

    // 会話履歴・設定の永続化API(2026-08-18新設、db.rsモジュールdoc参照)。
    {
        let db_for_add = Arc::clone(&db);
        let db_for_list = Arc::clone(&db);
        app = app.at(
            "/v1/db/history",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_add);
                async move { db_add_message(req, db).await }
            }))
            .get(handler_fn(move |_req, _p| {
                let db = Arc::clone(&db_for_list);
                async move { db_list_messages(db).await }
            })),
        );
        let db_for_clear = Arc::clone(&db);
        app = app.at(
            "/v1/db/history/clear",
            post(handler_fn(move |_req, _p| {
                let db = Arc::clone(&db_for_clear);
                async move { db_clear_messages(db).await }
            })),
        );
        let db_for_set = Arc::clone(&db);
        let db_for_get = Arc::clone(&db);
        app = app.at(
            "/v1/db/settings",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_set);
                async move { db_set_setting(req, db).await }
            }))
            .get(handler_fn(move |_req, _p| {
                let db = Arc::clone(&db_for_get);
                async move { db_get_settings(db).await }
            })),
        );
        let db_for_info = Arc::clone(&db);
        app = app.at(
            "/v1/db/info",
            get(handler_fn(move |_req, _p| {
                let db = Arc::clone(&db_for_info);
                async move { db_info(db).await }
            })),
        );
        let db_for_relocate = Arc::clone(&db);
        app = app.at(
            "/v1/db/storage-path",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_relocate);
                async move { db_relocate(req, db).await }
            })),
        );
        let db_for_rsync = Arc::clone(&db);
        app = app.at(
            "/v1/db/rsync-backup",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_rsync);
                async move { db_rsync_backup(req, db).await }
            })),
        );
        let db_for_rsync_all = Arc::clone(&db);
        app = app.at(
            "/v1/db/rsync-backup-all",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_rsync_all);
                async move { db_rsync_backup_all(req, db).await }
            })),
        );
        let db_for_migrate = Arc::clone(&db);
        app = app.at(
            "/v1/db/migrate-legacy",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_migrate);
                async move { db_migrate_legacy(req, db).await }
            })),
        );
        let db_for_install_rsync = Arc::clone(&db);
        app = app.at(
            "/v1/db/install-rsync",
            post(handler_fn(move |req, _p| {
                let db = Arc::clone(&db_for_install_rsync);
                async move { db_install_rsync(req, db).await }
            })),
        );
    }

    // aruaru-llmの自動起動(2026-08-19新設、上記maybe_launch_aruaru_llmの
    // doc参照)。サーバー本体の起動をブロックしないよう非同期タスクで
    // バックグラウンド実行する(ヘルスチェック自体に最大700msかかり
    // 得るため)。
    tokio::spawn(maybe_launch_aruaru_llm());

    // 起動時の自動メンテナンス/自動アップデート(2026-08-11追加、ユーザー
    // 指示「起動時の自動メンテナンスで自動UPDATEの自動バージョンアップ
    // 機能も搭載して」)。サーバーの起動(=フロントエンド側のメンテナンス
    // バナー表示中)をブロックしないよう、非同期タスクとしてバック
    // グラウンドで実行する。新バージョンが見つかった場合、この関数は
    // アンインストール/インストールを起動した上でプロセス自体を終了する
    // (`self_update.rs`のモジュールdoc参照)。
    tokio::spawn(self_update::check_and_apply_update());

    let addr = bind_addr();
    println!("open-english static server listening on http://{addr}/");
    println!("serving files from {}", root.display());

    let (bound_addr, handle) = Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .expect("failed to bind local server (is the port already in use?)");
    println!("bound to http://{bound_addr}/");
    handle.await.expect("server task panicked");
}
