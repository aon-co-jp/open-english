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
use open_runo_poem_compat::{get, Route, Server, TcpListener};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

mod self_update;

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

#[tokio::main]
async fn main() {
    let root = repo_root();
    let mut app = Route::new();
    for (url_path, rel_file, content_type) in STATIC_FILES {
        let file_path = root.join(rel_file);
        app = app.at(url_path, get(static_file_handler(file_path, content_type)));
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
