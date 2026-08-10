# open-english

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
   `cargo run --release`で起動(既定`http://localhost:4600`)。
2. このリポジトリの`index.html`をブラウザで開く(`aruaru-llm`と同一
   オリジンで配信する場合はそのURLを、直接`file://`で開く場合は
   ブラウザのローカルファイルfetch制限に注意——Chromeの場合
   `--allow-file-access-from-files`等が必要になる場合がある)。

## 次にすべきこと

1. ~~`aruaru-llm`側へのCORS対応~~ **完了(2026-08-10)**。
2. ~~GPT-2貪欲デコードの反復ループ~~ **根本解決済み(2026-08-10、
   繰り返しペナルティ実装)**。対話特化モデルへの入れ替え自体は
   進行中(下記「別セッションで進行中」参照)。
3. 音声合成(TTS)・リップシンクアニメーションの追加。
4. レベル別カリキュラム(文法・単語リスト等)の実装。
5. **(ユーザー指示、2026-08-10)** `open-directx`/`open-cuda`/
   `aruaru-llm`をブラウザ単体(WASM/WebGPU)でも動作させ、
   `RPoem`(GraphQL Federationプラットフォーム)とも連携させる構想。
   現在のPhase 0(ローカル常駐サーバー+localhost接続)とは別方向の
   大規模なアーキテクチャ変更(WASMコンパイル・WebGPU移植)を伴うため、
   MVP完成後に別途スコープを切って着手する。

## 別セッションで進行中(2026-08-10開始)

ユーザー指示により、以下の大規模タスクを別セッション(バックグラウンド
タスク`task_076ef43b`)で進行中:
1. GPT-2→対話ファインチューニング済みモデルへの差し替え検討+日英Web
   調査(英会話学習アプリのベストプラクティス調査含む)。
2. フロントエンド(`app.js`)をRust+RPoemパターン(`RPoem/apps/
   desktop-wasm`)へ移植。
3. `open-directx`/`open-cuda`/`aruaru-llm`/RPoemの組み合わせでGPT-2
   生成速度のボトルネック改善(実測ベース)。
4. 東芝SBM・DeepSeek系技術(Multi-Token Prediction・Speculative
   decoding等)の適用可否調査(こじつけ禁止、正直な判断を記録)。

進捗はそのセッション・各リポジトリのCLAUDE.md HANDOFFを確認すること。
