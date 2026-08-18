//! 会話履歴・設定のローカルデータベース(SQLite、2026-08-18新設)。
//!
//! ユーザー指示「open-englishのDATABASEは…円グラフと何%中の何%
//! 使用中かを表示して保存先を選択可能にしたり、保存DATABASEを移動も
//! 可能にしたり外部のGoogleドライブやUSBスティックメモリーやVPSにも
//! RSyncなどを使ってバックアップも…PCとタブレットとスマホのDATABASEを
//! 融合、同期…簡単に自動バックアップや同期が取れる事をPRして」への
//! 対応の**第一段階**。ユーザーとの相談の結果、まず会話履歴・設定を
//! 本格的なローカルデータベース(SQLite)化するところから着手する方針を
//! 確認済み。
//!
//! ## 正直な開示・今回のスコープ(重要)
//!
//! 今回実装したのは以下のみ:
//! - 会話履歴(`messages`テーブル)・設定(`settings`テーブル)の
//!   SQLite永続化と、それを読み書きするHTTP API。
//! - DBファイルの保存先パスを`OPEN_ENGLISH_DB_PATH`環境変数で変更
//!   可能にする(将来の「保存先選択」機能が土台にできるよう、既定パス
//!   決定ロジックを`OPEN_ENGLISH_SERVER_ROOT`と同じパターンで実装)。
//! - `GET /v1/db/info`でDBファイルサイズ・保存先ディスクの空き容量・
//!   総容量(使用率%算出用の生データ)を返す——**ただし実際の円グラフ
//!   表示・保存先選択UI・DBファイルの移動機能・外部(Googleドライブ/
//!   USB/VPS)へのrsyncバックップ・複数端末間の同期(重複しないマージ)
//!   は、いずれもこのコミットではまだ実装していない**。これらは
//!   このSQLite化を土台として次の増分で着手する(規模の大きい別機能
//!   群のため、一度に実装すると検証が疎かになるのを避ける)。
//! - フロントエンド(`app.js`)側は、既存の会話ログ表示に加えてこの
//!   APIへ保存する配線を追加したが、**既存の`localStorage`使用箇所
//!   (バージョン管理関連のクリーンアップ等)は置き換えていない**——
//!   会話履歴の永続化先をSQLiteへ追加しただけで、localStorage自体を
//!   廃止したわけではない。
//!
//! ## aruaru-db(PostgreSQL DUAL DB)へのオプトインミラーリング(2026-08-18)
//!
//! ユーザー再確認「SQLiteではなく、aruaru-db+できればPostgreSQLの
//! DUAL DBの方が片側にトラブルがあっても片側から自動修復する機能で
//! 安全性が高い」を受けて追加。**正直な開示(重要)**: `aruaru-db`
//! 自身が`DUAL_DATABASE_URL`環境変数経由で2つのPostgreSQLインスタンス
//! 間の自己修復ミラーリング(`DualDatabaseMirror`)を実装している
//! (`aruaru-db`側の既存機能)——このリポジトリ側で新たにDUAL DB
//! ロジックを実装したわけではない。`open-english-server`は
//! `OPEN_ENGLISH_DATABASE_URL`環境変数が設定されていれば
//! (`aruaru-llm`の`geo_content.rs`と同じ`tokio-postgres`直結パターンで)
//! そのPostgreSQLエンドポイント(`aruaru-db`が管理する冗長化構成で
//! あることを推奨)へも書き込みをミラーする、**普通のPostgreSQL
//! クライアントとして振る舞う**設計。未設定・接続失敗時はSQLiteのみで
//! 引き続き動作し続ける(既存の「サービスを止めない」方針、書き込み
//! 失敗はログに警告を出すのみでリクエスト自体は失敗させない)。

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
    /// **2026-08-18変更**: 保存先パスを実行時に変更できるよう
    /// `Mutex`化(ユーザー指示「DATA保存先は、既存の保存先でもそれ以外
    /// でも選択可能にして」への対応、`relocate`メソッド参照)。
    path: Mutex<PathBuf>,
    /// `OPEN_ENGLISH_DATABASE_URL`が設定されていれば`Some`
    /// (aruaru-db/PostgreSQLへの追加ミラー先URL、接続自体はリクエスト
    /// ごとに行う——常時接続を維持する複雑さを避けた簡易実装)。
    postgres_url: Option<String>,
}

/// `OPEN_ENGLISH_DATABASE_URL`環境変数を読む(aruaru-db/PostgreSQLの
/// 接続文字列、例: `host=127.0.0.1 user=aruaru dbname=open_english`)。
fn postgres_url_from_env() -> Option<String> {
    std::env::var("OPEN_ENGLISH_DATABASE_URL").ok().filter(|s| !s.trim().is_empty())
}

/// aruaru-db/PostgreSQLへメッセージ1件をベストエフォートでミラーする。
/// 接続・書き込み失敗は警告ログのみ(呼び出し元の成否には影響しない
/// ——SQLite側が常に正の情報源であり続ける設計)。
async fn mirror_message_to_postgres(url: &str, role: &str, content: &str) {
    let (client, connection) = match tokio_postgres::connect(url, tokio_postgres::NoTls).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("open-english: aruaru-db mirror connect failed (continuing with SQLite only): {e}");
            return;
        }
    };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("open-english: aruaru-db mirror connection error: {e}");
        }
    });
    if let Err(e) = client
        .execute(
            "CREATE TABLE IF NOT EXISTS open_english_messages (
                id BIGSERIAL PRIMARY KEY,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
            &[],
        )
        .await
    {
        eprintln!("open-english: aruaru-db mirror schema migration failed: {e}");
        return;
    }
    if let Err(e) = client.execute("INSERT INTO open_english_messages (role, content) VALUES ($1, $2)", &[&role, &content]).await {
        eprintln!("open-english: aruaru-db mirror insert failed: {e}");
    }
}

/// DBファイルの保存先。`OPEN_ENGLISH_DB_PATH`環境変数が最優先(将来の
/// 「保存先選択(内部ストレージ/microSD等)」機能が、この環境変数を
/// 動的に切り替える形で実装できるようにする土台)。未設定時は、
/// `OPEN_ENGLISH_SERVER_ROOT`(静的ファイル配信元、`main.rs`参照)と
/// 同じディレクトリ配下の`data/open-english.sqlite3`とする——インス
/// トール先ディレクトリに書き込み権限があることが既に前提の構成
/// (静的ファイルもそこから読むため)なのでDBファイルもそこに置く。
pub fn db_path(server_root: &std::path::Path) -> PathBuf {
    if let Ok(p) = std::env::var("OPEN_ENGLISH_DB_PATH") {
        return PathBuf::from(p);
    }
    server_root.join("data").join("open-english.sqlite3")
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("failed to create DB directory {parent:?}"))?;
        }
        let conn = Connection::open(&path).with_context(|| format!("failed to open SQLite DB at {path:?}"))?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at_unix INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )
        .context("failed to run schema migration")?;
        Ok(Self { conn: Mutex::new(conn), path: Mutex::new(path), postgres_url: postgres_url_from_env() })
    }

    pub fn has_postgres_mirror(&self) -> bool {
        self.postgres_url.is_some()
    }

    pub fn path(&self) -> PathBuf {
        self.path.lock().unwrap().clone()
    }

    /// 保存先を`new_path`へ変更する(ユーザー指示「DATA保存先は、既存の
    /// 保存先でもそれ以外でも選択可能にして」への対応、2026-08-18新設)。
    /// 現在のSQLite接続を閉じ、既存DBファイルを新しい場所へコピーした
    /// 上で新しい場所を開き直す——コピー完了まで元ファイルは削除しない
    /// (コピー失敗時に元データを失わないため)。
    pub fn relocate(&self, new_path: PathBuf) -> Result<()> {
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("failed to create DB directory {parent:?}"))?;
        }
        let mut conn_guard = self.conn.lock().unwrap();
        let mut path_guard = self.path.lock().unwrap();
        // 保留中のSQLiteページキャッシュをディスクへ確実に反映してから
        // ファイルをコピーする(WALモード等の未フラッシュデータの
        // 取りこぼしを防ぐ)。
        conn_guard.execute("PRAGMA wal_checkpoint(FULL)", []).ok();
        std::fs::copy(&*path_guard, &new_path).with_context(|| format!("failed to copy DB file from {:?} to {new_path:?}", *path_guard))?;
        let new_conn = Connection::open(&new_path).with_context(|| format!("failed to open relocated DB at {new_path:?}"))?;
        let old_path = path_guard.clone();
        *conn_guard = new_conn;
        *path_guard = new_path;
        drop(conn_guard);
        drop(path_guard);
        // 元ファイルは安全のため自動削除しない(ユーザー自身の判断で
        // 削除できるよう、パスをログへ出すのみに留める——誤って
        // データを失わせないための意図的な保守的挙動)。
        eprintln!("open-english: DB relocated, old file left in place at {old_path:?} (delete manually once verified)");
        Ok(())
    }

    /// `rsync`を子プロセスとして起動し、現在のDBファイルを
    /// `destination`(ローカルパス・`user@host:/path`のいずれも可、
    /// rsyncのCLI仕様通り)へ複製する(ユーザー指示「外部のGoogle
    /// ドライブやUSBスティックメモリーやVPSにもRsyncなどを使って
    /// バックアップも簡単に取れる」+「同期先もRSyncで選択可能に」への
    /// 対応、2026-08-18新設)。**正直な開示**: `rsync`本体はこのアプリに
    /// 同梱していない——利用者の環境(Linux/macOS/VPS/Android Termux等)に
    /// 既にインストールされている`rsync`をそのまま呼び出すのみ。
    /// Windows開発機のようにPATH上に`rsync`が無い環境では、その旨を
    /// 正直にエラーとして返す(黙って失敗にしない)。
    pub fn backup_via_rsync(&self, destination: &str) -> Result<String> {
        let path = self.path();
        let output = std::process::Command::new("rsync").arg("-a").arg(&path).arg(destination).output().with_context(|| {
            "failed to launch `rsync` (is it installed and on PATH? On Windows dev machines it typically isn't — this is expected there; use a Linux/macOS/VPS/Android target for real rsync backups)".to_string()
        })?;
        if output.status.success() {
            Ok(format!("rsync backup of {path:?} to {destination} succeeded"))
        } else {
            anyhow::bail!("rsync exited with {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
        }
    }

    /// 旧形式のエクスポート(メッセージ配列+設定連想配列)を新しい
    /// DATABASEシステムへ取り込む(ユーザー指示「既存の古い物から
    /// DATABASEシステムに移動も簡単にする機能」への対応、2026-08-18
    /// 新設)。**正直な開示**: このコミット時点でopen-english自体には
    /// 実際に永続化された「古いデータ」は存在しない(`app.js`は
    /// これまで会話履歴をlocalStorageへ保存していなかったことを実際に
    /// 確認済み——バージョン管理用の一部キーのみ)。そのため本関数は
    /// 実在する具体的な旧データ形式への対応ではなく、**将来どのような
    /// 旧形式のエクスポートが持ち込まれても受け入れられる汎用的な
    /// 取り込み口**として実装した(役割ID・本文の配列、キー/値設定の
    /// 連想配列という最小公倍数的な形)。戻り値は取り込んだ件数。
    pub fn import_legacy(&self, messages: &[(String, String)], settings: &[(String, String)]) -> Result<(usize, usize)> {
        for (role, content) in messages {
            self.add_message(role, content)?;
        }
        for (key, value) in settings {
            self.set_setting(key, value)?;
        }
        Ok((messages.len(), settings.len()))
    }

    /// SQLiteへ同期的に保存し、`OPEN_ENGLISH_DATABASE_URL`が設定されて
    /// いればaruaru-db/PostgreSQLへも非同期でベストエフォートミラー
    /// する(呼び出し元のエラーには影響しない、上記モジュールdoc参照)。
    pub fn add_message(&self, role: &str, content: &str) -> Result<()> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        self.conn
            .lock()
            .unwrap()
            .execute("INSERT INTO messages (role, content, created_at_unix) VALUES (?1, ?2, ?3)", rusqlite::params![role, content, now as i64])
            .context("failed to insert message")?;
        if let Some(url) = self.postgres_url.clone() {
            let role = role.to_string();
            let content = content.to_string();
            tokio::spawn(async move { mirror_message_to_postgres(&url, &role, &content).await });
        }
        Ok(())
    }

    pub fn list_messages(&self, limit: i64) -> Result<Vec<StoredMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, role, content, created_at_unix FROM messages ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(StoredMessage { id: row.get(0)?, role: row.get(1)?, content: row.get(2)?, created_at_unix: row.get(3)? })
        })?;
        let mut out: Vec<StoredMessage> = rows.collect::<rusqlite::Result<_>>()?;
        out.reverse(); // 古い順に戻す(表示側は時系列順を期待するため)。
        Ok(out)
    }

    pub fn clear_messages(&self) -> Result<()> {
        self.conn.lock().unwrap().execute("DELETE FROM messages", []).context("failed to clear messages")?;
        Ok(())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn
            .lock()
            .unwrap()
            .execute("INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value", rusqlite::params![key, value])
            .context("failed to upsert setting")?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// DBファイルの容量・保存先ディスクの空き容量を返す(将来の円グラフ
    /// 表示機能向けの生データ、2026-08-18時点ではこの数値をそのまま
    /// JSONで返すのみで、円グラフ描画自体は未実装)。
    pub fn file_size_bytes(&self) -> u64 {
        std::fs::metadata(self.path()).map(|m| m.len()).unwrap_or(0)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredMessage {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at_unix: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_list_messages_round_trips_in_chronological_order() {
        let dir = std::env::temp_dir().join(format!("open-english-db-test-{}", std::process::id()));
        let db = Db::open(dir.join("test.sqlite3")).unwrap();
        db.add_message("user", "hello").unwrap();
        db.add_message("trainer", "hi there").unwrap();
        let msgs = db.list_messages(10).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "hello");
        assert_eq!(msgs[1].content, "hi there");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clear_messages_removes_all_rows() {
        let dir = std::env::temp_dir().join(format!("open-english-db-test-clear-{}", std::process::id()));
        let db = Db::open(dir.join("test.sqlite3")).unwrap();
        db.add_message("user", "hello").unwrap();
        db.clear_messages().unwrap();
        assert_eq!(db.list_messages(10).unwrap().len(), 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn set_setting_upserts_by_key() {
        let dir = std::env::temp_dir().join(format!("open-english-db-test-settings-{}", std::process::id()));
        let db = Db::open(dir.join("test.sqlite3")).unwrap();
        db.set_setting("level", "beginner").unwrap();
        db.set_setting("level", "intermediate").unwrap();
        let all = db.get_all_settings().unwrap();
        assert_eq!(all, vec![("level".to_string(), "intermediate".to_string())]);
        std::fs::remove_dir_all(&dir).ok();
    }
}
