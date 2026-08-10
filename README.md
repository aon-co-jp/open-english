# open-english

> 📌 **最近の更新(2026-08-10、続き5)**: Google Custom Search JSON API
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
