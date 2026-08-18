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
    ("/style.css", "style.css", "text/css; charset=utf-8"),
    ("/app.js", "app.js", "application/javascript; charset=utf-8"),
    ("/auto-update.js", "auto-update.js", "application/javascript; charset=utf-8"),
    ("/version.json", "version.json", "application/json; charset=utf-8"),
    ("/manifest.json", "manifest.json", "application/manifest+json; charset=utf-8"),
    ("/exam-prep-questions.json", "exam-prep-questions.json", "application/json; charset=utf-8"),
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

fn bind_addr() -> SocketAddr {
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
            "db_path": db.path.display().to_string(),
            "db_file_size_bytes": db.file_size_bytes(),
            "postgres_mirror_configured": db.has_postgres_mirror(),
            "used_disk_bytes": serde_json::Value::Null,
            "total_disk_bytes": serde_json::Value::Null,
        }),
    )
}

#[tokio::main]
async fn main() {
    let root = repo_root();
    let db_path = db::db_path(&root);
    let db = Arc::new(Db::open(db_path).expect("failed to open local SQLite DB (data/open-english.sqlite3)"));
    println!(
        "conversation DB: {} (aruaru-db/PostgreSQL mirror: {})",
        db.path.display(),
        if db.has_postgres_mirror() { "enabled via OPEN_ENGLISH_DATABASE_URL" } else { "disabled (SQLite only)" }
    );

    let mut app = Route::new();
    for (url_path, rel_file, content_type) in STATIC_FILES {
        let file_path = root.join(rel_file);
        app = app.at(url_path, get(static_file_handler(file_path, content_type)));
    }

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
    }

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
