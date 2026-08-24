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

mod component_update;
mod db;
mod github_agent;
mod local_agent;
mod self_update;
mod vps_agent;

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
    ("/world-language-exams.json", "world-language-exams.json", "application/json; charset=utf-8"),
    ("/world-language-phrases.json", "world-language-phrases.json", "application/json; charset=utf-8"),
    ("/world-language-regions.json", "world-language-regions.json", "application/json; charset=utf-8"),
    ("/icons/icon-32.png", "icons/icon-32.png", "image/png"),
    ("/icons/icon-180.png", "icons/icon-180.png", "image/png"),
    ("/icons/icon-192.png", "icons/icon-192.png", "image/png"),
    ("/icons/icon-512.png", "icons/icon-512.png", "image/png"),
    ("/icons/open-english.ico", "icons/open-english.ico", "image/x-icon"),
];

/// 静的ファイルへの`HEAD`リクエスト用ハンドラ(2026-08-24新設)。
///
/// **背景・正直な開示**: 従来`STATIC_FILES`は`GET`のみ登録しており、
/// `curl -I`のようなHEADリクエストは(登録された`GET`ルートと同じ
/// パスであっても)`open_runo_router::hyper_compat::Router`が
/// メソッド不一致として`404`を返していた。多くのHTTPクライアント・
/// リバースプロキシ・ヘルスチェックツールは接続確認や更新確認に
/// `HEAD`を使うため、実用性への影響が小さくない既知のギャップ
/// だった(この節の直前のHANDOFF「静的ファイルサーバーがHEADメソッド
/// 未対応」参照)。RFC 9110 9.3.2の定義通り、`GET`と同じヘッダー
/// (`Content-Type`・`Content-Length`)を返しつつボディは送らない。
/// `open-runo-poem-compat`側に`MethodRouter::head`/`head()`が
/// 無かったため、本対応の一環として新設した(`RPoem/crates/
/// open-runo-poem-compat/src/lib.rs`参照、追加のみで既存APIは
/// 変更していない)。
fn static_file_head_handler(path: PathBuf, content_type: &'static str) -> open_runo_poem_compat::Handler {
    std::sync::Arc::new(move |_req, _params| {
        let path = path.clone();
        Box::pin(async move {
            match tokio::fs::metadata(&path).await {
                Ok(meta) => hyper::Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", content_type)
                    .header("content-length", meta.len().to_string())
                    .body(open_runo_poem_compat::fixed_body(bytes::Bytes::new()))
                    .expect("building a HEAD response from a fixed set of valid headers cannot fail"),
                Err(_) => open_runo_poem_compat::empty_status(StatusCode::NOT_FOUND),
            }
        })
    })
}

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

/// 実行基盤(CPU)の情報を返すエンドポイント(2026-08-22新設)。
///
/// `aruaru-llm`側に追加した実行基盤バッジ機能と同様に、このサーバーが
/// どのCPU命令セットを使える環境で動いているかをフロントエンドから
/// 確認できるようにする。検出はエコシステム共通ライブラリ
/// [`open_cpu`](https://github.com/aon-co-jp/open-cpu)へ委譲しており、
/// 内部で`OnceLock`キャッシュ済みのため毎リクエストのコストはほぼゼロ。
///
/// 【正直な開示】現時点では「検出結果を返すだけ」で、open-englishの
/// 処理そのものをこれらの命令で高速化しているわけではない。将来
/// 音声処理・行列演算等を最適化する際の判断材料として先に公開する。
async fn cpu_runtime() -> Response {
    let c = open_cpu::detect();
    let (vendor, family) = open_cpu::vendor_family();
    rs_json_response(
        StatusCode::OK,
        &serde_json::json!({
            "open_cpu_version": open_cpu::VERSION,
            "summary": c.summary(),
            "gf_impl": format!("{:?}", open_cpu::selected_impl()),
            // --- 2026-08-23 追加: 複数命令セットの「組み合わせ」情報 ---
            // 単独フラグの羅列だけでは「AVX-512F と BW が両方揃っているか」
            // のような実際のディスパッチ条件が分からないため、open-cpu が
            // 判定した組み合わせプロファイルと各カーネルの選択実装を返す。
            "isa_profile": c.isa_profile().name(),
            "isa_profile_raw": c.isa_profile_raw().name(),
            "float_impl": open_cpu::selected_float_impl().to_string(),
            "bit_impl": open_cpu::bit_impl_summary(),
            "avx512_opt_in": open_cpu::avx512_opt_in(),
            "cpu_vendor": format!("{:?}", vendor),
            "cpu_family": format!("{family:#x}"),
            // BMI2 ビットが立っていても Zen〜Zen 2 では pext/pdep が
            // マイクロコードで遅く、スカラーの方が速い(実測 7.1 倍差)。
            "fast_bmi2": c.fast_bmi2(),
            "detected_but_unused": c.detected_but_unused().to_names(),
            "combination_examples": {
                "avx2+fma3": c.supports_all(&[open_cpu::Feature::Avx2, open_cpu::Feature::Fma]),
                "avx512f+bw+vl": c.supports_all(&[
                    open_cpu::Feature::Avx512f,
                    open_cpu::Feature::Avx512bw,
                    open_cpu::Feature::Avx512vl
                ]),
                "avx512f+bw+vnni": c.supports_all(&[
                    open_cpu::Feature::Avx512f,
                    open_cpu::Feature::Avx512bw,
                    open_cpu::Feature::Avx512vnni
                ]),
                "ssse3+pclmulqdq": c.supports_all(&[
                    open_cpu::Feature::Ssse3,
                    open_cpu::Feature::Pclmulqdq
                ]),
                "gfni+avx2": c.supports_all(&[open_cpu::Feature::Gfni, open_cpu::Feature::Avx2])
            },
            "features": {
                "sse2": c.sse2,
                "ssse3": c.ssse3,
                "popcnt": c.popcnt,
                "aes": c.aes,
                "pclmulqdq": c.pclmulqdq,
                "bmi1": c.bmi1,
                "bmi2": c.bmi2,
                "fma": c.fma,
                "sha": c.sha,
                "avx2": c.avx2,
                "avx512f": c.avx512f,
                "avx512bw": c.avx512bw,
                "avx512vl": c.avx512vl,
                "avx_vnni": c.avx_vnni,
                "avx512vnni": c.avx512vnni,
                "gfni": c.gfni,
                "vpclmulqdq": c.vpclmulqdq
            },
            // 誇張しないための明示。
            "disclosure_ja": "このエンドポイントはCPU命令セットの検出結果を報告するだけで、open-english本体の処理(チャット応答・学習機能)はCPU集約的な演算を持たないため、現時点でSIMD高速化の適用先は無い。実際に高速化されているのは open-raid-z のGF(2^8)演算、open-cuda/aruaru-llm のCPU推論、open-cg-cad の断面積微分。",
            "disclosure_en": "This endpoint only reports detected CPU features. open-english itself has no CPU-bound hot loop, so no SIMD path is applied here yet; the actual accelerated consumers are open-raid-z (GF(2^8)), open-cuda/aruaru-llm (CPU inference) and open-cg-cad."
        }),
    )
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
    // `Db::rsync_available()`はブロッキングな子プロセス呼び出し
    // (`rsync --version`)のため、tokioワーカースレッドを塞がないよう
    // `spawn_blocking`へ逃がす(2026-08-24、これまでこの関数はどこからも
    // 呼ばれておらずdead_code警告が出ていた——`/v1/db/rsync-backup`は
    // 実際に`rsync`を起動してから初めて未インストールを検知する設計
    // だったため、事前にここで分かるようにして`GET /v1/db/info`一発で
    // 「バックアップを試す前にrsync が使えるかどうか」を確認できるようにした)。
    let rsync_available = tokio::task::spawn_blocking(Db::rsync_available).await.unwrap_or(false);
    rs_json_response(
        StatusCode::OK,
        &serde_json::json!({
            "db_path": db.path().display().to_string(),
            "db_file_size_bytes": db.file_size_bytes(),
            "postgres_mirror_configured": db.has_postgres_mirror(),
            "rsync_available": rsync_available,
            // 2026-08-24 DUAL同時書き込み対応で追加。`dual_mirror`が
            // trueなら`OPEN_ENGLISH_DATABASE_URL`と
            // `OPEN_ENGLISH_DATABASE_URL_SECONDARY`の両方が設定済みで、
            // 会話履歴が2つのDBへ同時に書き込まれる状態。
            // `mirror_targets`は表示名のみ(接続文字列はパスワードを
            // 含み得るため返さない)。
            "dual_mirror": db.is_dual_mirror(),
            "mirror_targets": db.mirror_labels(),
            // 2026-08-24 自己修復(未反映キュー)対応で追加。
            // `mirror_outbox_pending`は「ミラー書き込みに失敗して
            // 再送待ちの件数」、`mirror_outbox_given_up`は
            // 「規定回数リトライしても成功しなかった件数」
            // (黙って捨てず件数として見えるようにしてある)。
            "mirror_outbox_pending": db.outbox_counts().0,
            "mirror_outbox_given_up": db.outbox_counts().1,
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

/// クエリ文字列から1個のパラメータを取り出す(RPoem/hyperにクエリ
/// パーサが無いため、簡易な自前実装で足りる範囲〈`key=value`を`&`で
/// 区切り、`%XX`パーセントエンコーディングをデコード〉に留めた、
/// 2026-08-20新設)。
fn query_param(req: &Request, key: &str) -> Option<String> {
    let query = req.uri().query()?;
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next()?;
        let v = it.next().unwrap_or("");
        if k == key {
            return Some(percent_decode(v));
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
                out.push(bytes[i]);
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ============================================================
// AIプログラミング支援バックエンド(2026-08-20新設)
//
// ユーザー指示「open-englishに、AIプログラミング支援のためのバックエンド
// 機能として、VPSへの自動読み書き・GitHubへの自動読み書き・ローカル
// ドライブへの自動読み書き(サーバー常駐エージェント式)を追加」への
// 対応。セキュリティ設計の詳細は`vps_agent.rs`/`github_agent.rs`/
// `local_agent.rs`各モジュールのdoc参照(鍵・トークンはブラウザへ
// 送信しない、書き込み範囲は許可ディレクトリ/許可パスに限定、任意
// コマンド実行APIは公開しない、という3原則はいずれのモジュールでも
// 省略せず実装済み)。
// ============================================================

async fn agent_local_read(req: Request) -> Response {
    let Some(path) = query_param(&req, "path") else {
        return rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "missing 'path' query parameter"}));
    };
    match tokio::task::spawn_blocking(move || local_agent::read_file(&path)).await {
        Ok(Ok(content)) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "content": content})),
        Ok(Err(e)) => rs_json_response(StatusCode::FORBIDDEN, &serde_json::json!({"ok": false, "error": e.to_string()})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": format!("task panicked: {e}")})),
    }
}

#[derive(serde::Deserialize)]
struct LocalWriteRequest {
    path: String,
    content: String,
}

async fn agent_local_write(req: Request) -> Response {
    let body: LocalWriteRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match tokio::task::spawn_blocking(move || local_agent::write_file(&body.path, &body.content)).await {
        Ok(Ok(())) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true})),
        Ok(Err(e)) => rs_json_response(StatusCode::FORBIDDEN, &serde_json::json!({"ok": false, "error": e.to_string()})),
        Err(e) => rs_json_response(StatusCode::INTERNAL_SERVER_ERROR, &serde_json::json!({"error": format!("task panicked: {e}")})),
    }
}

async fn agent_vps_read(req: Request) -> Response {
    let Some(path) = query_param(&req, "path") else {
        return rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "missing 'path' query parameter"}));
    };
    match vps_agent::read_file(&path).await {
        Ok(content) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "content": content})),
        Err(e) => rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

#[derive(serde::Deserialize)]
struct VpsWriteRequest {
    path: String,
    content: String,
}

async fn agent_vps_write(req: Request) -> Response {
    let body: VpsWriteRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match vps_agent::write_file(&body.path, &body.content).await {
        Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true})),
        Err(e) => rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

/// GitHubトークンはヘッダ(`x-github-token`)経由で都度受け取る(クエリ
/// 文字列やURLへは載せない——アクセスログ等への平文残留を避けるため)。
async fn agent_github_read(req: Request) -> Response {
    let token = req.headers().get("x-github-token").and_then(|v| v.to_str().ok()).map(str::to_string);
    let Some(owner) = query_param(&req, "owner") else {
        return rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "missing 'owner' query parameter"}));
    };
    let Some(repo) = query_param(&req, "repo") else {
        return rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "missing 'repo' query parameter"}));
    };
    let Some(path) = query_param(&req, "path") else {
        return rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"error": "missing 'path' query parameter"}));
    };
    let branch = query_param(&req, "ref");
    match github_agent::read_file(&owner, &repo, &path, branch.as_deref(), token.as_deref()).await {
        Ok(f) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "content": f.content, "sha": f.sha})),
        Err(e) => rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

#[derive(serde::Deserialize)]
struct GithubCommitRequest {
    owner: String,
    repo: String,
    path: String,
    content: String,
    message: String,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    sha: Option<String>,
    token: String,
}

async fn agent_github_commit(req: Request) -> Response {
    let body: GithubCommitRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match github_agent::commit_file(&body.owner, &body.repo, &body.path, &body.content, &body.message, body.branch.as_deref(), body.sha.as_deref(), &body.token).await {
        Ok((commit_sha, html_url)) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "commit_sha": commit_sha, "html_url": html_url})),
        Err(e) => rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

/// `GET /v1/updates/history`(2026-08-20新設): open-english本体+同梱
/// コンポーネント(aruaru-llm・aruaru-db)それぞれの現在バージョン+
/// 保持している旧バージョン一覧を返す。UI側の「🔄 Updates & Rollback」
/// パネルがこれを基にダウングレード先の選択肢を表示する。
async fn updates_history() -> Response {
    let mut list = vec![self_update::self_history_info()];
    list.extend(component_update::history_info_all());
    rs_json_response(StatusCode::OK, &serde_json::json!({"components": list}))
}

#[derive(serde::Deserialize)]
struct DowngradeRequest {
    component: String,
    version: String,
}

/// `POST /v1/updates/downgrade`(2026-08-20新設、ユーザー指示「バージョン
/// アップしたらBUGだった場合の為に、簡単にそのBUGのリポジトリだけ
/// ダウングレード出来るように」への対応、主な新規実装対象)。
/// `component`は`"self"`(open-english本体)・`"aruaru-llm"`・
/// `"aruaru-db"`のいずれか。**正直な開示**: `self`のダウングレードが
/// 成功する場合、内部で`std::process::exit`によりこのプロセス自体が
/// 終了する(既存の自己更新〈`self_update::check_and_apply_update`〉と
/// 同じ設計)ため、成功時のHTTPレスポンス自体はクライアントへ届かない
/// ことがある——UI側は数秒待ってから`/healthz`への再接続を試みる想定。
async fn updates_downgrade(req: Request) -> Response {
    let body: DowngradeRequest = match read_rs_json_body(req).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    if body.component == "self" || body.component == "open-english" {
        match self_update::downgrade_self(&body.version).await {
            Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true, "note": "restart in progress"})),
            Err(e) => rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"ok": false, "error": e.to_string()})),
        }
    } else {
        match component_update::downgrade_component(&body.component, &body.version).await {
            Ok(()) => rs_json_response(StatusCode::OK, &serde_json::json!({"ok": true})),
            Err(e) => rs_json_response(StatusCode::BAD_REQUEST, &serde_json::json!({"ok": false, "error": e.to_string()})),
        }
    }
}

/// `GET /v1/world-languages`(2026-08-22新設、ユーザー指示「世界中の言語
/// でも擬似模擬試験を受けられるように」への対応)。`world-language-exams.json`
/// を読み、**問題本文を含まない**言語一覧のサマリ(言語コード・現地語表記・
/// 英語名・日本語名・RTLか・収録問題数・収録レベル)だけを返す。
///
/// **設計意図**: フロントエンドの「追加する言語を選ぶ」UI(メンテナンス
/// 中の言語追加パネル)は一覧だけあれば描画でき、全問題(数十KB)を毎回
/// 転送する必要が無い。実際の出題時のみ`/world-language-exams.json`
/// (静的配信)を取得する二段構えにしている。
///
/// **正直な開示**: 収録問題はこのアプリ用に書き下ろしたオリジナル問題で、
/// 実在の資格試験(DELE・DELF・Goethe-Zertifikat・HSK・TOPIK等)の
/// 過去問ではなく、それらの試験とは一切無関係。言語ごとの収録数は不均一
/// (3〜6問)で、レベル表記もCEFR風の目安に過ぎない。この不均一さは
/// レスポンスの`question_count`にそのまま出るため、UI側で正直に表示できる。
async fn world_languages() -> Response {
    let path = repo_root().join("world-language-exams.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => {
            return rs_json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &serde_json::json!({"error": format!("failed to read world-language-exams.json: {e}")}),
            )
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return rs_json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &serde_json::json!({"error": format!("world-language-exams.json is not valid JSON: {e}")}),
            )
        }
    };
    let empty = vec![];
    let langs = parsed.get("languages").and_then(|v| v.as_array()).unwrap_or(&empty);
    let summary: Vec<serde_json::Value> = langs
        .iter()
        .map(|lang| {
            let questions = lang.get("questions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut levels: Vec<String> = questions
                .iter()
                .filter_map(|q| q.get("level").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect();
            levels.sort();
            levels.dedup();
            serde_json::json!({
                "code": lang.get("code").cloned().unwrap_or(serde_json::Value::Null),
                "endonym": lang.get("endonym").cloned().unwrap_or(serde_json::Value::Null),
                "en": lang.get("en").cloned().unwrap_or(serde_json::Value::Null),
                "ja": lang.get("ja").cloned().unwrap_or(serde_json::Value::Null),
                "rtl": lang.get("rtl").cloned().unwrap_or(serde_json::Value::Bool(false)),
                "authored": lang.get("authored").cloned().unwrap_or(serde_json::Value::Bool(false)),
                "question_count": questions.len(),
                "levels": levels,
            })
        })
        .collect();
    rs_json_response(
        StatusCode::OK,
        &serde_json::json!({
            "count": summary.len(),
            "disclosure_en": "Original practice questions written for this app. Not past questions from, and not affiliated with, any official language certification exam. Levels are loose CEFR-style approximations only.",
            "disclosure_ja": "本アプリ用に書き下ろしたオリジナル練習問題です。実在の語学資格試験の過去問ではなく、いかなる公式試験とも無関係です。レベル表記はCEFR風の大まかな目安に過ぎません。",
            "languages": summary,
        }),
    )
}

/// `GET /v1/region-info?lang=<code>`(2026-08-22新設、ユーザー指示「選択した
/// 言語〈特に一番上の言語〉の地域に関する情報を集めて、その言語での話題に
/// ついていけるように」への対応)。`world-language-regions.json`から
/// 該当言語1件ぶんの静的な基本情報(国旗・国名・首都・主要都市・観光名所・
/// 名物・著名人・代表的な企業とその概要・その他)を返す。
///
/// **正直な開示**: ここで返すのは**リアルタイムの情報ではない**——本アプリ用に
/// 記述した一般的な百科事典的知識であり、インターネットから取得していないため
/// 古くなり得る。最新ニュースの見出しは別エンドポイント(`/v1/region-news`)が
/// 担当する。
async fn region_info(req: Request) -> Response {
    let lang = query_param(&req, "lang").unwrap_or_default();
    let path = repo_root().join("world-language-regions.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => {
            return rs_json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &serde_json::json!({"error": format!("failed to read world-language-regions.json: {e}")}),
            )
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return rs_json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &serde_json::json!({"error": format!("world-language-regions.json is not valid JSON: {e}")}),
            )
        }
    };
    let empty = vec![];
    let found = parsed
        .get("languages")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty)
        .iter()
        .find(|l| l.get("code").and_then(|c| c.as_str()) == Some(lang.as_str()));
    match found {
        Some(entry) => rs_json_response(
            StatusCode::OK,
            &serde_json::json!({
                "ok": true,
                "region": entry,
                "disclosure_en": "Static background knowledge written for this app, not fetched from the internet and not real-time. Details may be out of date.",
                "disclosure_ja": "本アプリ用に記述した静的な基礎知識です。インターネットから取得したリアルタイム情報ではなく、内容が古くなっている場合があります。",
            }),
        ),
        None => rs_json_response(
            StatusCode::OK,
            &serde_json::json!({
                "ok": false,
                "error_en": format!("No background data has been written for language '{lang}' yet."),
                "error_ja": format!("言語コード「{lang}」の基礎データはまだ作成されていません。"),
            }),
        ),
    }
}

/// `GET /v1/region-news?lang=<code>`(2026-08-22新設)。**実際にインターネットへ
/// 接続して**、その言語向けのGoogleニュースRSS(公開フィード)から最新の見出しを
/// 取得して返す。
///
/// **設計判断**: 専用のニュースAPI(有償・APIキー必須のものが多い)へ依存させず、
/// 認証不要の公開RSSを使う。取得するのは**見出しと公開日時とリンクのみ**で、
/// 記事本文は取得・再配布しない(著作権への配慮——本文を読みたい利用者は
/// 元記事のリンクを開く)。
///
/// **正直な開示**: オフライン環境・RSSの仕様変更・地域による到達性の違いに
/// より取得に失敗することがある。その場合はエラーを黙って握りつぶさず、
/// `ok:false`と理由を返す(UI側もその旨を利用者へ正直に表示する)。
async fn region_news(req: Request) -> Response {
    let lang = query_param(&req, "lang").unwrap_or_default();
    // 言語コード -> Googleニュースの hl/gl/ceid パラメータ。未知の言語は
    // 英語版のフィードへフォールバックする(黙って空を返さない)。
    let (hl, gl) = match lang.as_str() {
        "ja" => ("ja", "JP"),
        "es" => ("es", "ES"),
        "fr" => ("fr", "FR"),
        "de" => ("de", "DE"),
        "it" => ("it", "IT"),
        "pt" => ("pt-PT", "PT"),
        "nl" => ("nl", "NL"),
        "sv" => ("sv", "SE"),
        "no" => ("no", "NO"),
        "da" => ("da", "DK"),
        "fi" => ("fi", "FI"),
        "pl" => ("pl", "PL"),
        "cs" => ("cs", "CZ"),
        "hu" => ("hu", "HU"),
        "ro" => ("ro", "RO"),
        "ru" => ("ru", "RU"),
        "uk" => ("uk", "UA"),
        "el" => ("el", "GR"),
        "tr" => ("tr", "TR"),
        "ar" => ("ar", "EG"),
        "he" => ("he", "IL"),
        "fa" => ("fa", "IR"),
        "hi" => ("hi", "IN"),
        "bn" => ("bn", "BD"),
        "id" => ("id", "ID"),
        "ms" => ("ms", "MY"),
        "vi" => ("vi", "VN"),
        "th" => ("th", "TH"),
        "tl" => ("en", "PH"),
        "zh" => ("zh-CN", "CN"),
        "zh-Hant" => ("zh-TW", "TW"),
        "ko" => ("ko", "KR"),
        "ta" => ("ta", "IN"),
        "te" => ("te", "IN"),
        "ur" => ("ur", "PK"),
        "mr" => ("mr", "IN"),
        "pa" => ("pa", "IN"),
        "sw" => ("sw", "KE"),
        "rm" => ("de", "CH"),
        _ => ("en", "US"),
    };
    let url = format!("https://news.google.com/rss?hl={hl}&gl={gl}&ceid={gl}:{hl}");
    let client = match reqwest::Client::builder()
        .user_agent("open-english/1.0 (topic briefing)")
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return rs_json_response(
                StatusCode::OK,
                &serde_json::json!({"ok": false, "error": format!("failed to build HTTP client: {e}")}),
            )
        }
    };
    let body = match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                return rs_json_response(
                    StatusCode::OK,
                    &serde_json::json!({"ok": false, "source": url, "error": format!("failed to read the feed body: {e}")}),
                )
            }
        },
        Ok(resp) => {
            return rs_json_response(
                StatusCode::OK,
                &serde_json::json!({"ok": false, "source": url, "error": format!("the news feed returned HTTP {}", resp.status())}),
            )
        }
        Err(e) => {
            return rs_json_response(
                StatusCode::OK,
                &serde_json::json!({"ok": false, "source": url, "error": format!("could not reach the news feed (offline?): {e}")}),
            )
        }
    };
    let items = extract_rss_titles(&body, 8);
    rs_json_response(
        StatusCode::OK,
        &serde_json::json!({
            "ok": true,
            "lang": lang,
            "source": url,
            "headlines": items,
            "disclosure_en": "Headlines only, fetched live from a public Google News RSS feed. Article text is not copied; open the original link to read a story.",
            "disclosure_ja": "公開されているGoogleニュースRSSからその都度取得した見出しのみです。記事本文は取得・転載していません(本文は元記事のリンクからお読みください)。",
        }),
    )
}

/// RSS(XML)から`<item>`の`<title>`を最大`limit`件抜き出す最小のパーサ。
/// XMLパーサのクレートを追加せずに済ませるための割り切った実装で、
/// `<title>`の入れ子や属性は扱わない(Googleニュースの単純なRSSに限定)。
/// **正直な開示**: 汎用のRSSパーサではないため、フィードの構造が変われば
/// 見出しが0件になることがある(その場合もエラーにはせず空配列を返す)。
fn extract_rss_titles(xml: &str, limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    // チャンネル自身の<title>を拾わないよう、最初の<item>以降だけを見る。
    let body = match xml.find("<item>") {
        Some(i) => &xml[i..],
        None => return out,
    };
    for chunk in body.split("<item>").skip(0) {
        if out.len() >= limit {
            break;
        }
        let Some(start) = chunk.find("<title>") else { continue };
        let rest = &chunk[start + "<title>".len()..];
        let Some(end) = rest.find("</title>") else { continue };
        let title = decode_xml_entities(rest[..end].trim());
        if !title.is_empty() {
            out.push(title);
        }
    }
    out
}

fn decode_xml_entities(s: &str) -> String {
    let s = s
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>")
        .to_string();
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

#[tokio::main]
async fn main() {
    let root = repo_root();
    let db_path = db::db_path(&root);
    let db = Arc::new(Db::open(db_path).expect("failed to open local SQLite DB (data/open-english.sqlite3)"));
    println!(
        "conversation DB: {} (mirror: {})",
        db.path().display(),
        if db.is_dual_mirror() {
            format!("DUAL simultaneous write to [{}]", db.mirror_labels().join(", "))
        } else if db.has_postgres_mirror() {
            format!("single target [{}] (set OPEN_ENGLISH_DATABASE_URL_SECONDARY for DUAL)", db.mirror_labels().join(", "))
        } else {
            "disabled (SQLite only)".to_string()
        }
    );

    // 自己修復(未反映キューの再送)バックグラウンドタスク
    // (2026-08-24新設、`db.rs`モジュールdoc「自己修復」節参照)。
    // 起動時に1回試し、その後は一定間隔で繰り返す。ミラー先が
    // 未設定なら何もしない(タスク自体を起動しない)。
    if db.has_postgres_mirror() {
        let retry_secs: u64 = std::env::var("OPEN_ENGLISH_MIRROR_RETRY_SECS").ok().and_then(|v| v.parse().ok()).filter(|n| *n > 0).unwrap_or(60);
        let retry_path = db.path();
        let retry_mirrors = db.mirrors();
        println!("DB mirror self-repair: retry queue enabled (every {retry_secs}s)");
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(retry_secs));
            loop {
                // 1回目の`tick()`は即座に返る(=起動時の初回実行)。
                ticker.tick().await;
                db::outbox_retry_once(retry_path.clone(), retry_mirrors.clone()).await;
            }
        });
    }

    let mut app = Route::new();
    for (url_path, rel_file, content_type) in STATIC_FILES {
        let file_path = root.join(rel_file);
        app = app.at(
            url_path,
            get(static_file_handler(file_path.clone(), content_type)).head(static_file_head_handler(file_path, content_type)),
        );
    }
    app = app.at("/healthz", get(handler_fn(move |_req, _p| async move { healthz().await })));
    // `/health`はopen-web-server/open-easy-web側の「分身の術」テナント
    // 登録パターン(他リポジトリのCLAUDE.md HANDOFF多数参照)が汎用的に
    // 期待するヘルスチェック命名に形状を揃えるための別名(2026-08-24新設)。
    // 中身は`/healthz`と同一(`healthz()`をそのまま呼ぶだけ、新規ロジック
    // なし)——open-english自体をopen-web-server経由で公開する際に
    // エンドポイント名の食い違いで弾かれないようにする、実装コストの
    // 小さい歩み寄り。既存の`/healthz`は後方互換のためそのまま維持する。
    app = app.at("/health", get(handler_fn(move |_req, _p| async move { healthz().await })));
    // 実行基盤(CPU命令セット)情報(2026-08-22新設、cpu_runtime()のdoc参照)。
    app = app.at("/v1/cpu-runtime", get(handler_fn(move |_req, _p| async move { cpu_runtime().await })));
    // 多言語擬似模擬試験の対応言語一覧(2026-08-22新設、world_languages()のdoc参照)。
    app = app.at("/v1/world-languages", get(handler_fn(move |_req, _p| async move { world_languages().await })));
    // 話題ブリーフィング(2026-08-22新設): 静的な地域情報+公開RSSからの実ニュース見出し。
    app = app.at("/v1/region-info", get(handler_fn(move |req, _p| async move { region_info(req).await })));
    app = app.at("/v1/region-news", get(handler_fn(move |req, _p| async move { region_news(req).await })));

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
        app = app.at("/v1/updates/history", get(handler_fn(move |_req, _p| async move { updates_history().await })));
        app = app.at("/v1/updates/downgrade", post(handler_fn(move |req, _p| async move { updates_downgrade(req).await })));
    }

    // AIプログラミング支援バックエンド(2026-08-20新設、上記
    // "AIプログラミング支援バックエンド"セクションのdoc参照)。
    {
        app = app.at("/v1/agent/local/read", get(handler_fn(move |req, _p| async move { agent_local_read(req).await })));
        app = app.at("/v1/agent/local/write", post(handler_fn(move |req, _p| async move { agent_local_write(req).await })));
        app = app.at("/v1/agent/vps/read", get(handler_fn(move |req, _p| async move { agent_vps_read(req).await })));
        app = app.at("/v1/agent/vps/write", post(handler_fn(move |req, _p| async move { agent_vps_write(req).await })));
        app = app.at("/v1/agent/github/read", get(handler_fn(move |req, _p| async move { agent_github_read(req).await })));
        app = app.at("/v1/agent/github/commit", post(handler_fn(move |req, _p| async move { agent_github_commit(req).await })));
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

    // 同梱コンポーネント(aruaru-llm・任意のaruaru-db)の自動アップデート
    // (2026-08-19新設、`component_update.rs`のモジュールdoc参照、ユーザー
    // 指示「自動UPDATE機能は、全ての関連リポジトリを自動アップデートする
    // 機能搭載として」への対応)。open-english本体の自己更新
    // (`self_update::check_and_apply_update`)とは独立のタスクとして
    // バックグラウンド実行する。
    tokio::spawn(component_update::check_and_apply_all());

    // 定期的な自動アップデートチェック(2026-08-20新設、ユーザー指示
    // 「メンテナンスのタイミングで自動バージョンアップの自動アップデート
    // 機能も確実に」への対応)。従来は起動時のみのチェックだったため、
    // 長時間起動しっぱなしのユーザーには新バージョンがいつまでも
    // 反映されない可能性があった。GitHub REST APIの未認証レート制限
    // (1時間あたり60リクエスト)に配慮し、過度に頻繁にはせず6時間間隔
    // とした(本体+aruaru-llm+aruaru-dbで1回あたり最大3リクエスト、
    // 24時間でも12リクエスト程度に収まる現実的な値)。
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 60 * 60));
        interval.tick().await; // 1回目のtickは即時発火するため消費するだけ(起動時チェックは上で既に実施済み)
        loop {
            interval.tick().await;
            println!("open-english periodic maintenance: running scheduled update check (every 6h)");
            self_update::check_and_apply_update().await;
            component_update::check_and_apply_all().await;
        }
    });

    let addr = bind_addr();
    // 実行基盤(CPU命令セット)を起動時に1行ログへ出す。どのSIMD経路が
    // 選ばれたかを実機で後から確認できるようにするため(open-cpu導入、2026-08-22)。
    println!("{}", open_cpu::runtime_summary());
    println!("open-english static server listening on http://{addr}/");
    println!("serving files from {}", root.display());

    let (bound_addr, handle) = Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .expect("failed to bind local server (is the port already in use?)");
    println!("bound to http://{bound_addr}/");
    handle.await.expect("server task panicked");
}
