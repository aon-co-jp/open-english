# open-english

*English*: [README-English.md](README-English.md) ·
*Other languages*: [Deutsch](README-German.md) · [Italiano](README-Italian.md) ·
[Français](README-French.md) · [Русский](README-Russian.md) ·
[Українська](README-Ukrainian.md) · [עברית](README-Hebrew.md) ·
[فارسی](README-Persian.md) · [العربية](README-Arabic.md)

> 📌 **最新の更新(2026-08-19)**: Facebookしかアクセスできないスマホ
> 契約の利用者向けに、Facebookページ/Messengerで共有するリンク先
> `facebook.html`を新設しました。**正直な開示**: Facebookの
> 「Free Basics」等のゼロレーティングプログラムへの正式な提携・
> 登録は本プロジェクト単独ではできないため、「Facebook経由で完全
> 無料アクセス」自体は実現できていません——`facebook.html`は
> Facebookアプリ内蔵ブラウザから開けるリンク先として機能し、
> そこから既存のインストーラー(Windows/Linux/macOS/Android)への
> ダウンロード導線を案内するにとどまります。アプリ本体は変わらず
> 利用者端末上のローカルサーバー(`server/`)で動作します。詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-19 HANDOFF参照。
>
> *English*: Added `facebook.html`, an entry page meant to be shared as
> a link on a Facebook Page or in Messenger, for users whose mobile
> plan only allows Facebook access. Honest disclosure: true Facebook
> "Free Basics"-style zero-rated free access is not achievable without
> an official partnership with Meta, which this project does not have
> — `facebook.html` works as a normal page reachable from Facebook's
> in-app browser and points to the existing installers (Windows/Linux/
> macOS/Android); the app itself still runs on a local server on your
> own device (`server/`). See the 2026-08-19 HANDOFF entry in
> [CLAUDE.md](CLAUDE.md).

> 📌 **最新の更新(2026-08-11〜12、v0.6.0)**: Android/タブレットが
> PC/Linuxサーバー不要で単体動作するアプリになりました——AI応答
> エンジン(`aruaru-llm`)自体もAPKへ実際に同梱し、実機で両プロセスの
> 生存・`/healthz`・`/v1/chat`応答を確認済みです。あわせて英検1〜5級・
> TOEIC・TOEFL・JLPT N1〜N5・日本語検定1〜3級の資格試験対策コーナー
> (各10問、オリジナル問題)+採点後にAI講師との練習(JLPT/日本語検定は
> 「日本語教室」モードへ自動切替)へつなげる機能、「学びたい言語
> (英会話/日本語会話)」選択、Linux/macOS版インストーラー
> (`installer/unix/install.sh`)を追加しました。**正直な開示**:
> モデル重み(GPT-2系・埋め込みモデル)はAPKに同梱していないため、
> Android版でAI応答を使うには別途モデルを内部ストレージへ配置する
> 必要があります(自動ダウンロード機能は未実装)。詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-11(続き7〜10)HANDOFF参照。
>
> *English*: Android/tablet now runs fully standalone — no PC or Linux
> server required. The AI response engine (`aruaru-llm`) itself is now
> bundled into the APK; on-device verification confirmed both processes
> stay alive and respond to `/healthz`/`/v1/chat`. Also added: a
> certification exam-prep corner (Eiken 1-5, TOEIC, TOEFL, JLPT N1-N5,
> Nihongo Kentei 1-3, 10 original questions each) that hands missed
> questions to the AI trainer after scoring (auto-switching to a
> "Japanese classroom" mode for JLPT/Nihongo Kentei), a "which language
> to learn" selector, and Linux/macOS installers
> (`installer/unix/install.sh`). Honest disclosure: model weights
> (GPT-2 family, embedding model) are not bundled in the APK — using AI
> chat on Android still requires placing model files in internal storage
> manually (no auto-download yet). See the 2026-08-11 (continued 7-10)
> HANDOFF entries in [CLAUDE.md](CLAUDE.md).

> 📌 **最新の更新(2026-08-18)**: 会話履歴・設定の本格的なローカル
> データベース化に着手しました。**なぜSQLite単体ではないか**——SQLiteは
> 常時利用可能なローカルコピーとして必須の基盤にしつつ、`aruaru-db`
> (PostgreSQL)を設定すればそちらへも自動でミラー書き込みします。
> `aruaru-db`自身が持つ`DUAL_DATABASE_URL`(2つのPostgreSQLインスタンス
> 間の自己修復ミラーリング)機能と組み合わせることで、**片側のDBに
> 障害が起きてももう片側から自動修復し、データを守る**、SQLite単体より
> 安全性の高い構成になります。接続先未設定・接続失敗時はSQLiteのみで
> 動作を継続するため、可用性は損ないません。今回実装したのは会話履歴・
> 設定のSQLite永続化+`/v1/db/*`API+実HTTPでの動作確認まで
> (`server/src/db.rs`参照)——円グラフでの使用率表示・保存先選択
> (マイクロSD等)・外部rsyncバックアップ・複数端末間の同期は次の
> 増分で着手します。続けて保存先選択(`/v1/db/storage-path`)・rsync
> バックアップ(`/v1/db/rsync-backup`)・旧データ取り込み(`/v1/db/
> migrate-legacy`)も実装。rsyncが未導入の環境では「**RSyncを
> インストールしましょう！**」という案内が返り、`/v1/db/install-rsync`
> でOS別パッケージマネージャ(Windows: winget/choco、Linux: apt-get/dnf/
> pacman、macOS: brew、Android: pkg)経由の自動インストール+成功直後の
> 自動バックアップ実行までを1回の呼び出しで行えます。
>
> *English*: Started building a proper local database for conversation
> history/settings. **Why not SQLite alone** — SQLite remains the
> always-available local baseline, but when `aruaru-db` (PostgreSQL) is
> configured, writes are also mirrored there automatically. Combined with
> `aruaru-db`'s own `DUAL_DATABASE_URL` feature (self-healing mirroring
> between two PostgreSQL instances), **if one database instance fails,
> the other automatically repairs it and protects your data** — a safer
> setup than SQLite alone. If no mirror is configured or the connection
> fails, the app keeps working on SQLite only, so availability is never
> sacrificed. This increment implements SQLite persistence for messages/
> settings plus the `/v1/db/*` API, verified over real HTTP (see
> `server/src/db.rs`). Also added: storage-location picker
> (`/v1/db/storage-path`), rsync backup (`/v1/db/rsync-backup`), and a
> generic legacy-data import endpoint (`/v1/db/migrate-legacy`). If rsync
> isn't installed, the API replies with a bilingual **"Let's install
> RSync!"** prompt, and `/v1/db/install-rsync` auto-installs it via the
> right package manager for the OS (winget/choco on Windows, apt-get/dnf/
> pacman on Linux, brew on macOS, pkg on Android) and immediately retries
> the backup on success. Usage pie chart display and multi-device sync
> are still planned for the next increment.

> 📌 **旧更新(2026-08-11、続き3)**: 起動時に自動でGitHubの最新版を
> 確認し、新しいバージョンがあれば自動でアンインストール→自動で
> インストールする機能を追加(Windowsのみ、`server/src/self_update.rs`)。
> **正直な開示**: 現時点でGitHub Releaseがまだ1件も無いため、実際の
> 自動更新の一気通貫の動作確認はまだできていない(バージョン比較ロジック・
> 「リリース無し時に正直に継続する」動作は実機確認済み)。詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-11(続き5)HANDOFF参照。
>
> *English*: Added an automatic self-update feature (Windows only) that
> checks GitHub for the latest release at startup and, if newer,
> automatically uninstalls the old version and installs the new one.
> Honest disclosure: no GitHub Release exists yet, so the full
> uninstall→install flow hasn't been end-to-end verified yet (version-
> comparison logic and the "no release found, continue safely" path
> have been). See the 2026-08-11 (continued 5) HANDOFF entry in
> [CLAUDE.md](CLAUDE.md).

> 📌 **旧更新(2026-08-11、続き2)**: 就職・転職・観光の話題を検出し、
> aruaru.tokyo(AI駆動開発 CLAUDE CODE DESKTOP)・audiocafe.tokyo/aruaru
> (IT・建築系求人)・audiocafe.tokyo/aruaru-lady(女性向け求人)・
> nasa.tokyoへのリンクを日英併記で案内する機能を追加(通常チャット・
> 研修モード両方で動作)。実機でも実際にリンク表示を確認済み。詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-11(続き4)HANDOFF参照。
>
> *English*: Added detection for job-hunting/career-change/tourism
> topics that introduces aruaru.tokyo, audiocafe.tokyo/aruaru,
> audiocafe.tokyo/aruaru-lady, and nasa.tokyo in both English and
> Japanese (works in both normal chat and training mode). Verified
> live. See the 2026-08-11 (continued 4) HANDOFF entry in
> [CLAUDE.md](CLAUDE.md).

> 📌 **旧更新(2026-08-11、続き)**: 日本47都道府県・米国50州・主要
> 世界首都(観光名所・名物料理・お土産)のDBと連携し、自己紹介研修の
> 話題を動的化。富士山が話題になると安全上の注意(スキーウェア・
> ヘルメット着用、山小屋の事前予約推奨)・山小屋/登山バス/登山用品店
> 一覧・観光ツアーのオンライン予約検索を日英併記で案内する機能を追加。
> 年齢層(乳幼児〜シニア)・レベル(超初心者〜ネイティブ)・ビジネス
> 英会話追加選択のUIも追加。実際に`aruaru-llm`+配信サーバーを起動し
> ブラウザで検証済み(発見した3件の実バグも修正済み)。詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-11の各HANDOFF参照。
>
> *English*: Linked to a new geo/tourism database (all 47 Japanese
> prefectures, 50 US states, major world capitals with landmarks/food/
> souvenirs) to make the self-introduction training dynamic. When Mount
> Fuji comes up, the app now shows a bilingual safety advisory (wear
> ski gear + a helmet, reserve a mountain hut in advance) plus real hut/
> bus/gear-shop info and a tour-booking search. Added age-group/level/
> business-English selection UI. Verified live against a real running
> `aruaru-llm` + static server (found and fixed 3 real bugs in the
> process). See the 2026-08-11 HANDOFF entries in [CLAUDE.md](CLAUDE.md).

> 📌 **旧更新(2026-08-11)**: ブラウザから直接Google検索APIキー・
> 検索エンジンIDを保存できる設定パネルを追加(`POST /v1/settings/
> google-search`、メモリ上保持のみ)。Windowsインストーラー
> (`installer/windows/`)を実際にビルド・インストール・起動・
> アンインストールまで実機検証済み(Inno Setup、UAC不要)。
> 詳細は[CLAUDE.md](CLAUDE.md)の2026-08-11 HANDOFF参照。
>
> *English*: Added a settings panel for saving your Google Search API
> key/cx directly from the browser (`POST /v1/settings/google-search`,
> in-memory only). The Windows installer (`installer/windows/`) has
> now been actually built, installed, launched, and uninstalled on
> real hardware (Inno Setup, no admin rights required). See the
> 2026-08-11 HANDOFF entry in [CLAUDE.md](CLAUDE.md).

> 📌 **旧更新(2026-08-10、続き5)**: Google Custom Search JSON API
> によるブリッジ式検索補強(`POST /v1/generate-with-search`、ユーザー
> 自身のAPIキーが必要・未設定時は自動フォールバック)+「Google search
> boost」トグルをUIに追加。Android WebViewアプリ(`android/`、タブレット
> でも同一アプリで動作)・Windowsインストーラー(`installer/windows/`、
> Inno Setup)に着手(実機/実ビルド検証は一部次回持ち越し、詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-10(続き5)HANDOFF参照)。
>
> *English*: Added a bridge-style Google Custom Search JSON API
> integration (`POST /v1/generate-with-search`, requires your own API
> key, falls back automatically when unset) + a "Google search boost"
> UI toggle. Started an Android WebView app (`android/`, same app works
> on tablets) and a Windows installer (`installer/windows/`, Inno
> Setup) — some real-device/build verification is carried over to next
> time, see the 2026-08-10 (continued 5) HANDOFF entry in
> [CLAUDE.md](CLAUDE.md).

> 📌 **最近の更新(2026-08-10、続き)**: (1) 既定モデルを`gpt2`(124M)から
> `distilgpt2`(82M)へ切替(約42%高速化、詳細は`aruaru-llm/CLAUDE.md`
> 参照)。(2) フロントエンドJSのRust/WASM移植は「性能上のメリットが無く
> `SpeechRecognition`は非標準APIで手書きFFIが必要」と判断し見送り、
> 代わりに**配信サーバー側をRust化**(新規`server/`、RPoem
> `open-runo-poem-compat`ベース、`python3 -m http.server`依存を解消)。
> (3) 日本語で話しかけてもハイブリッド(英日併記)応答を必ず返すよう
> 改善(`app.js`の`ensureHybridReply`——モデルが日本語を含む応答を
> 生成できなかった場合はフロントエンド側で日本語の一言を自動補完し、
> 「英日併記」という構造を保証する。機械翻訳の質を偽って主張はしない)。
> (4) バージョン管理機能(`version.json`にセマンティックバージョン追加+
> 画面下部に表示)と、旧バージョンの自動クリーンアップ
> (`auto-update.js`——新バージョン検出時にこのアプリ専用の
> localStorageを破棄しキャッシュ破棄付きで再読み込み。ネイティブ
> インストーラーではない静的Webアプリのため「旧ファイルの自動削除」は
> 安全性の観点から行わず、ブラウザ側の痕跡クリーンアップに限定)。
> 詳細は[CLAUDE.md](CLAUDE.md)の2026-08-10(続き3)HANDOFF参照。
>
> *English*: (1) Switched the default model from `gpt2` (124M) to
> `distilgpt2` (82M), ~42% faster (see `aruaru-llm/CLAUDE.md`).
> (2) Decided **against** porting the frontend JS to Rust/WASM (no
> performance benefit, and `SpeechRecognition` has no standard web-sys
> binding) — instead **ported the local file server to Rust**
> (new `server/` crate, built on RPoem's `open-runo-poem-compat`,
> removing the `python3 -m http.server` dependency). (3) Improved
> Japanese input handling so hybrid (English+Japanese) replies are
> always guaranteed (`app.js`'s `ensureHybridReply` — if the model's
> reply contains no Japanese, the frontend appends a short honest
> Japanese note itself; it does not fake machine-translation quality).
> (4) Added version management (`version.json` now has a semantic
> `version` field, shown in the footer) and automatic cleanup of old
> versions' browser-side traces (`auto-update.js` clears this app's own
> `localStorage` and does a cache-busting reload on update — since this
> is a static web app with no native installer, "uninstalling old
> versions" is scoped to browser-side leftovers only, not disk files).
> See the 2026-08-10 (continued 3) HANDOFF entry in
> [CLAUDE.md](CLAUDE.md) for details.

> 📌 **最近の更新(2026-08-10)**: CORS対応(`aruaru-llm`側に
> `.with_cors()`実装)、GPT-2貪欲デコードの反復ループ根本解決
> (`open-cuda`側`GptModel::generate_with_repetition_penalty`、既定
> `penalty=1.3`)、風天のトラさんキャラクターの見た目調整(カバン・
> わらじサンダル)+切替時ジングル+研修モード名乗り修正、実際の秋葉原
> メイドカフェ(@ほぉ～むカフェ)の接客技法を研修モードへ追加、日本文化
> ブーム(アニメ・漫画・アニソン・ゲーム・日本語学習者・御朱印・温泉旅館・
> 日本食)を日英でWeb調査し研修内容へ反映、Windows/Mac/Linux/Android/
> iPhone/iPad向けランチャーアイコン一式(`icons/`+`launchers/`+
> `manifest.json`)、自動更新機能(`auto-update.js`、`version.json`
> ポーリング)を追加。詳細は[CLAUDE.md](CLAUDE.md)の2026-08-10 HANDOFF
> 参照。
>
> *English*: Added CORS support (`.with_cors()` on the `aruaru-llm`
> side), fixed the root cause of GPT-2 greedy-decode's degenerate
> repetition loop (`open-cuda`'s `GptModel::generate_with_repetition_
> penalty`, default `penalty=1.3`), tweaked the Tora-san character's
> look (bigger light-brown bag, straw-sandal-style feet) + added a
> switch-in jingle + fixed his self-introduction, added a training step
> based on a real Akihabara maid cafe's (@ほぉ～むカフェ) actual
> customer-service technique, researched (in Japanese and English) and
> added a step covering the current overseas boom in Japanese culture
> (anime/manga, anime songs, games, Japanese-language learners, goshuin
> stamp collecting, onsen ryokan tourism, Japanese food), added launcher
> icons for Windows/Mac/Linux/Android/iPhone/iPad
> (`icons/`+`launchers/`+`manifest.json`), and added an auto-update
> mechanism (`auto-update.js` polling `version.json`). See the
> 2026-08-10 HANDOFF entry in [CLAUDE.md](CLAUDE.md) for details.

PC・タブレット・スマートフォンで動く英会話学習Webアプリ(Phase 0)。
「メイドカフェ・イングリッシュ」のような雰囲気で、超初心者から上級者まで
自由に対応する英会話トレーナーを、魔法少女メイドキャラクター
(`sample-maid`と同じ独自デザインの流れを汲む、アニメーション付き)が
担当する。

## アーキテクチャ(ユーザー指示、2026-08-10)

- **Linux(VPS)側**: 配布用のダウンロードサーバーのみ(このアプリ自体の
  実行環境ではない)。アプリ管理は
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web)が担う。
- **利用者の端末(PC/タブレット/スマホ)側**: このリポジトリの静的Web
  フロントエンド(HTML/CSS/JS、ブラウザで動く)+
  [`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm)のローカル常駐
  サーバー(ネイティブ実行ファイル、`open-directx`/`open-cuda`の推論基盤を
  内部で利用)を利用者自身の端末にダウンロード・実行してもらい、ブラウザは
  `http://localhost:4600`(aruaru-llmの既定ポート)へオンライン/オフライン
  問わずローカル接続する「ハイブリッド」構成。

## 現在のスコープ(Phase 0)・正直な開示

- **AI応答の品質について**: `aruaru-llm`の`/v1/generate`はGPT-2(124M〜
  1.5B、英語中心・対話特化のファインチューニング無し)による自己回帰的
  テキスト生成であり、`aruaru-llm`自身のCLAUDE.mdに「応答品質は不安定」
  と明記されている。本アプリのAI応答にも同じ注意書きを画面上に表示する
  ——「流暢な会話ができる」という誇張はしない。
- **CORSについて**: `aruaru-llm`のHTTPサーバーにはCORSヘッダの設定が
  無いため、このフロントエンドを`aruaru-llm`とは別オリジン
  (別ポート/別ホスト)で配信すると、ブラウザから直接`fetch`できない
  (ブロックされる)。Phase 0では、利用者が`file://`または
  `aruaru-llm`と同一オリジンでこのフロントエンドを開く運用を前提とする
  ——恒久対応(`aruaru-llm`側へのCORS対応追加)は別リポジトリの変更を
  伴うため、ユーザー確認の上で別途対応する。
- **レベル別対応**: 超初心者〜上級者のレベル選択UIはこのPhase 0でも
  実装しているが、実際にレベルに応じて応答の難易度を変える機能は
  プロンプトへの簡単な指示文の付加のみ(GPT-2側で確実にレベルを
  守った応答をする保証は無い、正直な開示)。
- **アニメーション**: メイドキャラクターはCSSアニメーション(口の
  開閉ループ)で「喋っている」演出をするプレースホルダー。実際の
  音声合成(TTS)・リップシンクは未実装(次回以降のロードマップ)。

## 必要なインストーラー一覧(2026-08-17新設)

open-englishを動かすには、以下2つのソフトをダウンロード・インストール
する必要があります(ソースからのビルドが不要な、ワンタップに近い方法)。

| # | 何か | Windows | Linux | Android/タブレット |
|---|---|---|---|---|
| 1 | **open-english本体**(このリポジトリ、静的フロントエンド+配信サーバー) | [open-english-install.exe](https://github.com/aon-co-jp/open-english/releases/latest/download/open-english-install.exe) | [tar.gz](https://github.com/aon-co-jp/open-english/releases/latest) | [APK](https://github.com/aon-co-jp/open-english/releases/latest)(アセット一覧から`.apk`を選択) |
| 2 | **aruaru-llm**(AI応答エンジン、必須——無いとチャット機能が動きません) | [aruaru-llm-windows-x86_64.zip](https://github.com/aon-co-jp/aruaru-llm/releases/latest/download/aruaru-llm-windows-x86_64.zip) | [tar.gz](https://github.com/aon-co-jp/aruaru-llm/releases/latest) | Android版は同梱済み(open-englishのAPK内に含まれる、別途インストール不要) |

**正直な開示**: 上記の表の「latest」リンクはGitHub Releasesの最新版を指す
自動追従リンクです(タグを固定した特定バージョンが欲しい場合は
[Releasesページ](https://github.com/aon-co-jp/open-english/releases)から
個別に選んでください)。macOS向けの`aruaru-llm`ビルド済み配布は現時点で
まだ無く(`open-english`側はLinux/macOS両対応のtar.gzがありますが、
`aruaru-llm`はLinux/Windowsのみ)、macOSで動かす場合は`aruaru-llm`を
ソースからビルドする必要があります。

Windows版はインストール後、起動時に自動アップデート機能
(`server/src/self_update.rs`)がGitHub Releasesを確認し、新しいバージョンが
あれば自動でアンインストール→インストールします(ユーザー操作不要)。

## 実行方法

1. [`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm)を
   `cargo run --release`で起動(既定`http://localhost:4600`、既定モデルは
   `distilgpt2`)。
2. `server/`ディレクトリで`cargo run --release`を実行し、このリポジトリの
   静的フロントエンドを`http://127.0.0.1:4601/`で配信する(RPoemベース、
   `python3 -m http.server`は不要になった——`OPEN_ENGLISH_SERVER_BIND`
   環境変数でポート変更可)。
3. ブラウザで`http://127.0.0.1:4601/`を開く。`file://`で直接開くことも
   可能だが、一部ブラウザは`fetch()`をブロックし自動更新機能が無効化
   されるため、上記手順2のサーバー経由を推奨する。

## 資格試験対策コーナー(2026-08-11)

英検1〜5級・TOEIC・TOEFLのレベル別擬似模擬試験機能(「📝 Exam Prep /
資格対策」ボタン)。**すべてオリジナルの練習問題**(実際の過去問は
著作権保護対象のため未使用)。採点後、間違えた問題を「トレーナーと
練習する」ボタンでチャットへ引き継ぎ、AI講師との会話練習につなげられる。

## Android版(単体動作、2026-08-11更新、記述の古さを2026-08-17に修正)

`android/`は、**PC/LinuxのWEBサーバー無しでスマホ単体で動作する**
Androidアプリ。`open-english-server`(静的フロントエンド配信)に加え、
AI応答エンジン`aruaru-llm`本体も`libaruarullm.so`としてAPKへ同梱済み
(2026-08-11)——両方とも端末上の`127.0.0.1`限定で自己完結して起動する
設計で、**同じWi-Fi内のPCへ接続する必要は無い**(このREADMEに以前
あった「PC上のIPアドレスを入力して接続」という説明は、aruaru-llm同梱前
〈2026-08-11当日の早い時点〉の古い状態を指しており、実態と乖離した
まま更新漏れしていたものです。訂正します)。

**正直な開示(このAndroid版固有の制約)**: 実際の応答生成に必要な
モデル重み(GPT-2系〈数百MB〉・multilingual-e5-small)はAPKには同梱
していません(サイズ・ライセンスの都合、PC版と同じ制約)。現時点では
ユーザー自身が端末の内部ストレージへモデルファイルを手動配置する
必要があり、自動ダウンロード機能はまだ実装していません——「サーバーが
端末内で起動すること」「静的UIが表示されること」までは単体で完結
しますが、実用的な応答品質を得るにはこのモデル配置手順が別途必要です。
詳細・実機検証結果は`CLAUDE.md`のHANDOFFを参照してください。

## 次にすべきこと

1. ~~`aruaru-llm`側へのCORS対応~~ **完了(2026-08-10)**。
2. ~~GPT-2貪欲デコードの反復ループ~~ **根本解決済み(2026-08-10、
   繰り返しペナルティ実装)**。
3. ~~既定モデルの高速化~~ **完了(2026-08-10、distilgpt2切替、約42%
   高速化)**。
4. ~~日本語入力時のハイブリッド応答保証~~ **完了(2026-08-10)**。
5. ~~配信サーバーのRust化~~ **完了(2026-08-10、`server/`crate)**。
   フロントエンドJS自体のRust/WASM移植は性能上のメリットが無いと判断し
   見送り(調査結果は`CLAUDE.md`参照)。
6. 音声合成(TTS)・リップシンクアニメーションの追加。
7. レベル別カリキュラム(文法・単語リスト等)の実装。
8. **(ユーザー指示、2026-08-10)** `open-directx`/`open-cuda`/
   `aruaru-llm`をブラウザ単体(WASM/WebGPU)でも動作させ、
   `RPoem`(GraphQL Federationプラットフォーム)とも連携させる構想。
   現在のPhase 0(ローカル常駐サーバー+localhost接続)とは別方向の
   大規模なアーキテクチャ変更(WASMコンパイル・WebGPU移植)を伴うため、
   MVP完成後に別途スコープを切って着手する。
9. 東芝SBM・DeepSeek系技術の適用可否調査(未着手)。
