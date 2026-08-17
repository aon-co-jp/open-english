//! 自動アップデート機能(2026-08-11新設)。
//!
//! ユーザー指示「起動時の自動メンテナンスで自動UPDATEの自動バージョン
//! アップ機能も搭載して、自動でGithubから最新版を見つけて古いのは
//! 自動でアンインストールして最新版を自動インストールする機能も
//! 搭載して」への対応。
//!
//! ## 正直な開示(最重要)
//!
//! - **現時点でGitHub Releaseが1件も存在しない**(2026-08-11確認、
//!   `GET /repos/aon-co-jp/open-english/releases/latest`は`404`)。
//!   この機能自体は正しく実装・単体テスト済みだが、実際に「新しい
//!   バージョンを検出→自動アンインストール→自動インストール」という
//!   一連の流れを実機で最初から最後まで実行するには、まずタグを
//!   push してGitHub Actions(`.github/workflows/release.yml`、本コミット
//!   で新設)経由で最初のリリースアセットを公開する必要がある。今回は
//!   「リリースが無い場合に正直に何もしない(クラッシュしない)」ことと
//!   「バージョン比較ロジックの正しさ」までを検証済みとし、実際の
//!   自動アンインストール→インストールの実行そのものは次回のリリース後に
//!   確認する必要があることを明記する。
//! - **アンインストール→インストールの実行方式**: 実行中のサーバー
//!   バイナリ自身が入っているディレクトリを、実行中のプロセスが
//!   ロックを保持したまま削除することはWindows上ではできない。
//!   このため、別プロセス(一時バッチスクリプト)を検出時に起動し、
//!   そのスクリプトが「少し待つ→既存のアンインストーラー
//!   (`unins000.exe`)をサイレント実行→新しいインストーラーをサイレント
//!   実行」という順で実行する間に、このサーバー自身のプロセスは終了する
//!   (`std::process::exit`)——ファイルロックを解放してから削除・
//!   上書きが行われる設計。新しいインストーラー側の`[Run]`セクション
//!   (既存の`open-english.iss`)がインストール後に自動でアプリを再起動
//!   するため、ユーザー操作なしで新バージョンが起動し直す。

use std::path::PathBuf;

use serde::Deserialize;

const GITHUB_REPO: &str = "aon-co-jp/open-english";

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct LatestRelease {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

/// `"v0.5.1"`や`"0.5.1"`のようなタグ文字列を`(major, minor, patch)`へ
/// パースする。パース不可な部分は0扱い(誇張せず「不明時は最新とは
/// 判断しない」安全側)。
fn parse_version(raw: &str) -> (u64, u64, u64) {
    let trimmed = raw.trim().trim_start_matches('v');
    let mut parts = trimmed.split('.').map(|s| s.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

/// リモート版がローカル版より新しいかどうか。
fn is_newer(remote: &str, local: &str) -> bool {
    parse_version(remote) > parse_version(local)
}

/// 現在の実行ファイルと同じディレクトリにある`version.json`の
/// `version`フィールドを読む(インストーラーがコピーする配置、
/// `open-english.iss`の`[Files]`参照)。
///
/// **正直な開示・実機テストで発覚した実バグ(2026-08-12)**: 以前は
/// 読めない場合に`"0.0.0"`を返していたが、これは`cargo build`の
/// `target/release/`から直接バイナリを実行する開発時やCI検証時など
/// (`version.json`が隣に存在しない)、常に「ローカルは0.0.0=最も古い」
/// と誤判定させ、`check_and_apply_update`が実際にGitHub Releaseの
/// インストーラーをダウンロードして無人インストールしてしまう
/// (このマシン自体で実際に再現・実害を確認した)。インストール済み
/// 配布物としての目印である`version.json`が無い=「インストーラー
/// 経由でインストールされたコピーではない」と判断し、`None`を返して
/// 呼び出し元でアップデート処理自体をスキップするよう変更。
fn local_version() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let path = dir.join("version.json");
    let text = std::fs::read_to_string(&path).ok()?;
    #[derive(Deserialize)]
    struct V {
        version: String,
    }
    serde_json::from_str::<V>(&text).ok().map(|v: V| v.version)
}

async fn fetch_latest_release() -> anyhow::Result<LatestRelease> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let res = client.get(&url).header("User-Agent", "open-english-self-updater").send().await?;
    if !res.status().is_success() {
        anyhow::bail!("GitHub releases API returned HTTP {}", res.status());
    }
    Ok(res.json::<LatestRelease>().await?)
}

/// 2026-08-17変更: インストーラーのファイル名規則を`open-english-setup-
/// {version}.exe`から`open-english-install.exe`(バージョン番号なし、
/// ユーザーが一目でインストーラーと分かる名前)へ変更したため、
/// マッチ対象の部分文字列も"setup"から"install"へ合わせた。旧リリース
/// (v0.6.6以前)のアセット名との後方互換のため"setup"も引き続き許容する。
fn windows_installer_asset(release: &LatestRelease) -> Option<&ReleaseAsset> {
    release.assets.iter().find(|a| {
        let name = a.name.to_lowercase();
        name.ends_with(".exe") && (name.contains("install") || name.contains("setup"))
    })
}

/// 実行ファイルと同じディレクトリの`unins000.exe`(Inno Setup既定の
/// アンインストーラー名)を探す。無ければ`None`(初回インストール直後の
/// 開発ビルド等、正常なケース)。
fn existing_uninstaller() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join("unins000.exe");
    candidate.exists().then_some(candidate)
}

/// アップデートを検出したら、アンインストール→新規インストールを
/// 行うバッチスクリプトを生成し、デタッチ起動した上でこのプロセス自身を
/// 終了する(ファイルロック解放のため)。呼び出し元には戻らない
/// (`std::process::exit`)。
async fn apply_update(installer_path: &std::path::Path) -> anyhow::Result<()> {
    let script_path = std::env::temp_dir().join("open-english-self-update.bat");
    let uninstaller = existing_uninstaller();
    let mut script = String::from("@echo off\r\nping 127.0.0.1 -n 3 > nul\r\n");
    if let Some(u) = &uninstaller {
        script += &format!("\"{}\" /VERYSILENT /SUPPRESSMSGBOXES\r\n", u.display());
        script += "ping 127.0.0.1 -n 3 > nul\r\n";
    }
    script += &format!("\"{}\" /VERYSILENT /SUPPRESSMSGBOXES\r\n", installer_path.display());
    script += &format!("del \"{}\"\r\n", script_path.display());
    std::fs::write(&script_path, script)?;

    tokio::process::Command::new("cmd")
        .args(["/C", "start", "", script_path.to_string_lossy().as_ref()])
        .spawn()?;

    tracing_log_if_available("open-english self-update: launched uninstall+reinstall script, exiting to release file locks");
    std::process::exit(0);
}

fn tracing_log_if_available(msg: &str) {
    println!("{msg}");
}

/// 起動時のメンテナンスチェックの一部として呼ぶ想定のエントリポイント。
/// GitHub Releasesを確認し、新しいバージョンがあればダウンロード→
/// アンインストール/インストールを起動する。ネットワーク断・リリース
/// 未存在(404)等は正直にログへ記録するだけで、サーバー自体の起動は
/// 妨げない(既存の「サービスを止めない」方針を踏襲)。
pub async fn check_and_apply_update() {
    // 2026-08-11修正: この自動更新機構は「実行中の自分自身のexeを
    // アンインストーラー→新インストーラーで差し替え、プロセスを終了する」
    // というWindows専用の設計(モジュールdoc冒頭参照)。プラットフォーム
    // 判定を一切していなかったため、Android版(単体動作アプリに同梱した
    // このバイナリ)でも新しいGitHub Releaseを検出するたびに
    // Windows用インストーラーの起動を試み(実機で`cmd: Can't find
    // service: /C`エラーとして実際に確認)、直後に`std::process::exit`
    // でサーバー自体を毎回強制終了させてしまう実バグがあった——
    // Android上ではこの機構全体を無効化する。Android版のアプリ更新は
    // 別途Kotlin側(`MainActivity.checkForAppUpdate`)がGitHub
    // Releasesページへのリンク表示のみを行う設計に委ねる。
    if !cfg!(target_os = "windows") {
        tracing_log_if_available("open-english self-update: skipped (this update mechanism is Windows-only)");
        return;
    }

    let release = match fetch_latest_release().await {
        Ok(r) => r,
        Err(e) => {
            tracing_log_if_available(&format!(
                "open-english self-update: could not check GitHub releases ({e}) — continuing without updating"
            ));
            return;
        }
    };

    let Some(local) = local_version() else {
        tracing_log_if_available(
            "open-english self-update: skipped (no version.json next to the executable — not an installed copy)",
        );
        return;
    };
    if !is_newer(&release.tag_name, &local) {
        tracing_log_if_available(&format!(
            "open-english self-update: up to date (local {local}, latest release {})",
            release.tag_name
        ));
        return;
    }

    let Some(asset) = windows_installer_asset(&release) else {
        tracing_log_if_available(&format!(
            "open-english self-update: newer release {} found but no Windows installer asset attached — skipping",
            release.tag_name
        ));
        return;
    };

    tracing_log_if_available(&format!(
        "open-english self-update: newer release {} found (local {local}), downloading {}",
        release.tag_name, asset.name
    ));

    let dest = std::env::temp_dir().join(&asset.name);
    match download_to(&asset.browser_download_url, &dest).await {
        Ok(()) => {
            if let Err(e) = apply_update(&dest).await {
                tracing_log_if_available(&format!("open-english self-update: failed to launch update ({e})"));
            }
        }
        Err(e) => {
            tracing_log_if_available(&format!("open-english self-update: download failed ({e}) — continuing without updating"));
        }
    }
}

async fn download_to(url: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::Client::builder().build()?;
    let bytes = client.get(url).header("User-Agent", "open-english-self-updater").send().await?.bytes().await?;
    std::fs::write(dest, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_strings_with_and_without_v_prefix() {
        assert_eq!(parse_version("v0.5.1"), (0, 5, 1));
        assert_eq!(parse_version("0.5.1"), (0, 5, 1));
        assert_eq!(parse_version("1.0"), (1, 0, 0));
        assert_eq!(parse_version("garbage"), (0, 0, 0));
    }

    #[test]
    fn is_newer_compares_semver_correctly() {
        assert!(is_newer("v0.6.0", "0.5.1"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(!is_newer("v0.5.1", "0.5.1"));
        assert!(!is_newer("v0.5.0", "0.5.1"));
    }
}
