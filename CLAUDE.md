# 開発方針＆開発環境ルール（open-english-pc）

開発方針・開発環境ルールの**正本**は
[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) の `CLAUDE.md`
（確認不要の自動継続／リミット解除後の自動再開・白画面バグ等を見逃さない
検証徹底・参照資料一覧など、全リポジトリ共通ルールはそちらを参照）。

## このリポジトリの役割

[`open-english`](https://github.com/aon-co-jp/open-english) の**クライアント**
（デスクトップ／タブレット／モバイル）を、本体（共有デモ環境＋配信サーバー
`open-english-server`）から分離して集約するモノレポ。

- `pc/` … Windows / Linux / macOS デスクトップ版
- `tablet/` … タブレット版
- `mobile/` … モバイル版

## 移設状況（HANDOFF）

- **2026-09-10 リポジトリ新設**: `aon-co-jp/open-english-pc` を作成（public、
  デフォルトブランチ `main`）。モノレポ骨組み（`pc/` `tablet/` `mobile/` +
  各 README）を配置。
- **2026-09-10 Phase 2 完了**: `git filter-repo` で `open-english` から
  クライアント資産を**履歴保持**で移設し、`--allow-unrelated-histories` で
  骨組みへマージ（203 コミット）。
  - `web/` … 共有 Web クライアント（`index.html` `style.css` `app.js`
    `auto-update.js` `version.json` `manifest.json` `exam-prep-questions.json`
    `sw.js` `facebook.html` `qr-confirm.html` `vault.html` `icons/`）
  - `mobile/android/` … 本体 `android/`
- **Phase 2b（未）**: `open-english` 本体は現在も同じファイルのコピーを保持して
  デモ配信中。本体の `.github/workflows/release.yml`（Package/cp）・
  `installer/windows/open-english.iss`（Source パス）・`server/src/main.rs`
  （静的 root 解決）・`android/` 参照を、この `web/` を参照する形へ切り替え、
  取得方法（submodule / CI clone）を決めてから本体のコピーを削除する。
- **Phase 3（未）**: 「起動してメンテナンス表示中は 30 秒間隔」で
  `self_update` ＋ `component_update`（**関連リポジトリ全て**のバージョン
  チェックと自動アップグレード）を回す拡張。正本ロジックは
  `open-english/server/src/self_update.rs` ／ `component_update.rs`。
  通常時の定期チェックは 30 分（`open-english` 側で 2026-09-09 に 6h→30m 済み）。
- **2026-09-14 Androidキーボード自動スクロールの実機根本修正(v0.8.7)**:
  ユーザー報告「縦スマホでキーボード表示より上に自動スクロールしない」を
  OPPO Reno11A実機（ADB接続）で繰り返し検証し、3件の実バグを特定・修正した。
  1. **根本原因**: `WindowInsetsCompat.Type.ime()` + `View.setPadding()`
     でWebViewへキーボード高さ分のbottom paddingを適用する方式は、padding
     自体は正しく適用されていたが、Chromium系WebViewが`onSizeChanged`を
     受け取らずJS側`window.innerHeight`/`visualViewport`が一切更新されない
     ことを実機ログ(`currentPadding=897`と適用は確認できるのに画面上は
     無反応)で特定した。`WebView.layoutParams.height`を直接書き換える方式
     (`getWindowVisibleDisplayFrame`でキーボード高さを算出)に切り替え、
     実際のビュー寸法変更として確実に`onSizeChanged`を発火させることで
     解決。ADB実機テストで入力欄がキーボード上に正しく自動スクロールされ、
     文字入力も可能なことを確認済み。
  2. ネイティブ`CLOSE`/`OPEN SETUP`ボタンが画面右上で重なって表示される
     バグ(`closeSetupBtn`がローカル変数でサーバー起動成功時の自動非表示
     処理から参照できず、非表示にし忘れていた)を修正。tablet flavor側の
     レイアウトにも同じオーバーレイボタンを追加。
  3. `assets/webroot/`(APK同梱の静的アセット)が`web/`(正本)から手動`cp`
     でしか同期されておらず、同期忘れで旧コンテンツが実機に配信され
     続ける実害を繰り返し引き起こしていた問題を、Gradle `preBuild`依存の
     自動コピータスク(`app/build.gradle.kts`の`syncWebrootFromWeb`)で解消。
  詳細は`mobile/android/app/src/main/java/tokyo/runo/openenglish/MainActivity.kt`
  のコメント参照。

## GitHub organization

https://github.com/aon-co-jp
