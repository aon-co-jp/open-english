# 設計思想＆開発方針＆開発環境ルール(open-english)

作業ドライブは`F:\runo`。この節は
[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を
正本とし、各プロジェクトへコピーして同期する方針に準じる。GitHub
リポジトリ: [aon-co-jp/open-english](https://github.com/aon-co-jp/open-english)。

**開発開始日: 2026-08-10。**

## このプロジェクトの役割

PC・タブレット・スマートフォンで動く英会話学習Webアプリ。「メイドカフェ
・イングリッシュ」のような雰囲気で、超初心者から上級者まで対応する
英会話トレーナーを、魔法少女メイドキャラクター(独自デザイン、
アニメーション付き)が担当する。AI応答は
[`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm)が担う。

## アーキテクチャ(ユーザー指示、2026-08-10)

- **Linux(VPS)側**: 配布用のダウンロードサーバーのみ。アプリ管理は
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web)。
- **利用者端末側**: このリポジトリの静的Webフロントエンド+
  `aruaru-llm`のローカル常駐サーバー(ネイティブ実行ファイル、
  `open-directx`/`open-cuda`の推論基盤を内部利用)を利用者自身が
  ダウンロード・実行し、ブラウザは`http://localhost:4600`へオンライン/
  オフライン問わずローカル接続するハイブリッド構成。

## 正直な開示・既知の制約(2026-08-10時点)

- `aruaru-llm`の`/v1/generate`はGPT-2(英語中心、対話特化のファイン
  チューニング無し)——応答品質・レベル遵守は保証されない。
- `aruaru-llm`のHTTPサーバーにCORSヘッダ設定が無いため、このフロント
  エンドを別オリジンで配信すると`fetch`がブロックされる(Phase 0では
  同一オリジン配信または`file://`での利用を前提、恒久対応は
  `aruaru-llm`側の変更が必要でユーザー確認の上で別途着手)。
- 音声合成(TTS)・リップシンクは未実装(現在はCSSアニメーションの
  口の開閉ループのみ)。

## 将来構想(ユーザー指示、2026-08-10、未着手)

`open-directx`/`open-cuda`/`aruaru-llm`をブラウザ単体(WASM/WebGPU)でも
動作させ、`RPoem`(GraphQL Federationプラットフォーム)とも連携させる
構想がある。現在のPhase 0(ローカル常駐サーバー+localhost接続)とは
別方向の大規模なアーキテクチャ変更(WASMコンパイル・WebGPU移植)を
伴うため、MVP完成後に各リポジトリ側で「ブラウザ対応済みか」を調査した
上で、別途スコープを切って着手する。

## HANDOFF

- **2026-08-10(続き) CORS修正・反復ループ根本解決・トラさん調整・
  実メイドカフェ技法反映・日本文化ブーム調査・ランチャーアイコン・
  自動更新機能**:
  1. **CORS対応**: `aruaru-llm`側`src/main.rs`の`Route`チェーン末尾に
     `.with_cors()`(`open-runo-poem-compat`既存機能)を追加。実HTTPで
     preflight・GET・POST全てに`access-control-allow-origin`ヘッダが
     付くことを確認済み(詳細は`aruaru-llm/CLAUDE.md`参照)。
  2. **GPT-2反復ループの根本解決**: `open-cuda`側
     `GptModel::generate_with_repetition_penalty`(CTRL方式)を新設し、
     `aruaru-llm`側`/v1/generate`が既定`penalty=1.3`で呼ぶよう配線
     (`ARUARU_LLM_REPETITION_PENALTY`環境変数で調整可)。実GPT-2重みで
     反復ループ解消を実証済み(詳細は`open-cuda/CLAUDE.md`・
     `aruaru-llm/CLAUDE.md`参照)。
  3. **トラさんキャラクター調整**(`index.html`のSVG): カバンを拡大+
     薄茶色化、靴をわらじ風サンダルへ変更。切替時に短いオリジナル
     ジングル(実在曲の再現ではない、Web Audio APIで手書き合成)を追加。
     研修モード中に切替した場合、表示済みの挨拶メッセージを新キャラの
     台詞へ差し替えるバグ修正(`replaceLastMessage`)。
  4. **実メイドカフェ接客技法を研修モードへ反映**: ユーザー提供の参考
     記事(秋葉原@ほぉ～むカフェの実際の接客スタイル)から、「完璧な
     文法は不要、キーワード+笑顔で会話成立」という技法を短い引用・
     要約に留めて練習問題へ翻案(記事本文の丸ごと転載はしていない)。
     挨拶に「Welcome home, master! / おかえりなさい、ご主人様!」を
     追加。ページ内に出典クレジット(「秋葉原メイドカフェ〈@ほぉ～む
     カフェ〉の接客技法・英会話研修のブログを参考にさせて頂きました。」)
     を明記。
  5. **日本文化ブームの日英Web調査を研修内容へ反映**: WebSearchで
     日本語・英語両方で調査(アニメ・漫画の世界的成長〈鬼滅の刃・進撃の
     巨人等〉、アニソンライブ〈Animelo Summer Live〉、日本のゲーム市場
     海外成長、世界の日本語学習者約379万人、御朱印集めブーム、温泉旅館・
     神社仏閣巡りの観光ブーム、日本食ブーム)。誇張しないよう、検索で
     確認できた範囲の事実(概数・傾向)のみを研修ステップへ追加。
  6. **話速・発話の流暢さ改善**: ユーザー指摘「メイドの喋りが途切れ
     途切れ」の原因を特定——発話テキストが"English / 日本語"混在形式
     のまま単一言語の`utter.lang`で読まれており、TTSエンジンが混在
     文字列を無理に読んで不自然な途切れが発生していた。`extractSpeechText`
     関数で行ごとに"/"分割し、選択言語に一致する側だけを抽出して
     読点でつなぐよう修正。あわせてメイド版の話速をさらに低速化
     (`rate: 0.92→0.82`)。
  7. **全プラットフォーム向けランチャーアイコン**: `icons/`(手書きPNG
     エンコーダで生成、外部画像ツール非依存)+`manifest.json`(PWA)+
     `launchers/`(Windows `.lnk`作成PowerShellスクリプト、Linux
     `.desktop`、macOS `.app`ビルドスクリプト、Android/iPhone/iPad向け
     「ホーム画面に追加」手順書)を新設。Windows用ショートカット作成は
     実際に実行し`.lnk`生成を確認済み。
  8. **自動更新機能**: `auto-update.js`(`version.json`を5秒間隔で
     ポーリングし、`buildId`が変化したら`location.reload()`)を追加。
     **正直な開示**: `file://`で直接開いた場合、一部ブラウザは
     ローカルファイルへの`fetch()`をセキュリティ上の理由でブロック
     することがある(その場合は静かに無効化されるのみで、ページの
     他機能には影響しない)。確実に動かすには`python3 -m http.server`
     等のローカルHTTPサーバー経由で開くこと。コード変更を反映させるには
     `version.json`の`buildId`を手動で更新する必要がある(自動ビルド
     パイプラインは無い)。
  - 次にすべきこと: (1) `version.json`の`buildId`更新を今後の変更時に
    忘れないこと(自動化されていない手動運用)、(2) 別セッション
    (`task_076ef43b`)で進行中の、モデル差し替え・フロントエンドRust
    移植・生成速度改善・SBM/DeepSeek調査の結果を確認し、完了後にこの
    CLAUDE.md・README.mdへ反映すること。

- **2026-08-10(続き3) 4項目タスク(速度改善→モデル差し替え→フロントエンド
  Rust移植→SBM/DeepSeek調査)着手、うち3番目を「RPoemサーバー側Rust化」に
  スコープ変更して実装**:
  1. **項目1(速度改善)・項目2(モデル差し替え)は`aruaru-llm`側で対応**
     (詳細は`aruaru-llm/CLAUDE.md`同日HANDOFF参照)。distilgpt2(82M)へ
     既定モデルを切替し約42%高速化(8.37秒→4.83秒/24トークン)。
  2. **項目3(フロントエンドRust移植)の調査結果**: `app.js`(533行)は
     DOM操作・fetch・Web Speech API(`SpeechSynthesis`+非標準
     `webkitSpeechRecognition`)制御が中心で計算負荷の高い処理が無く、
     Rust/WASM化に性能上のメリットが無いと判断(`SpeechRecognition`は
     web-sysに標準バインディングが無く手書きFFIが必要という追加コストも
     ある)。ユーザー確認の上、**フロントエンドJS自体の移植は見送り**、
     代わりに「配信サーバー側のRust化」へスコープを変更した。
  3. **新規`server/`crate**(`open-english-server`、RPoem
     `open-runo-poem-compat`をpath依存): `python3 -m http.server`への
     依存(2026-08-10(続き)HANDOFFに記載の既知の制約——`file://`直接
     オープン時に一部ブラウザが`fetch()`をブロックし`auto-update.js`の
     ポーリングが無効化される問題への回避策として案内していたコマンド)
     を解消。既存の`open_runo_poem_compat::hyper_compat::
     static_file_handler`(新規実装なし、既存関数の再利用)で
     `index.html`/`style.css`/`app.js`/`auto-update.js`/`version.json`/
     `manifest.json`/アイコン一式をディスクから配信する。既定
     `http://127.0.0.1:4601/`(`aruaru-llm`の既定`:4600`とは別ポート、
     `OPEN_ENGLISH_SERVER_BIND`環境変数で上書き可)。
  4. **実機検証**: `cargo build --release`成功。実際にバイナリを起動し
     `curl`で`/`(200・index.html本文確認)・`/version.json`(200・
     `buildId`確認)・`/app.js`(200・`content-type: application/
     javascript`・26935バイト)・`/style.css`(200・6363バイト)を実HTTPで
     確認済み。
  5. **正直な開示**: (1) ファイルはディスクから都度読み込む設計
     (`tokio::fs::read`、既存`static_file_handler`の実装通り)——
     埋め込み(`include_bytes!`)ではないため、配布時はこの`server/`
     バイナリと静的ファイル群を同じ相対位置に置く必要がある
     (`CARGO_MANIFEST_DIR`の親ディレクトリを実行時に解決)。
     (2) HEADメソッドは未対応(GETのみ登録、`curl -I`は405を返す——
     ブラウザの通常のページロード〈GET〉には影響しない)。
     (3) JSONを扱う処理は無い(静的ファイルをバイト列のまま配信するのみ)
     ため、ユーザー指示のあった「JSONはRS-JSON(`rust-json`クレート)へ
     切替」は本サーバーには該当箇所なし。
  6. **項目4(東芝SBM/DeepSeek調査)は未着手**(次回セッションへ持ち越し)。
  - 次にすべきこと: (1) `README.md`/`README-English.md`の「確実に動かす
    には`python3 -m http.server`」という案内を、この新サーバーの案内
    (`cargo run --release`、`server/`ディレクトリ)へ更新する、
    (2) `launchers/`(Windows `.lnk`等)がこの新サーバーを起動する形に
    更新するか検討、(3) 項目4(東芝SBM/DeepSeek技術組み込み構想)への着手。

- **2026-08-10(続き4) 日本語入力時のハイブリッド応答保証+バージョン管理
  機能+旧バージョンのブラウザ側クリーンアップを実装(ユーザー指示
  「日本語でしゃべっても英語と日本語で返事して」「バージョン管理する
  機能も搭載して古いのは自動アンインストールして」への対応)**:
  1. **ハイブリッド応答の構造的保証**(`app.js`): `containsJapanese
     (text)`(ひらがな/カタカナ/漢字のUnicodeプロパティエスケープ判定)・
     `ensureHybridReply(completion, userText)`を新設。ハイブリッドモード
     選択時、モデルの応答に日本語が一切含まれなければ、フロントエンド側で
     定型の短い日本語の一言(またはノート)を自動的に追記し、「英日併記」
     という構造を必ず保証する——機械翻訳の質を偽って主張はしない
     (正直な開示)。あわせて、ユーザー発話が日本語の場合はその事実を
     プロンプトへ明示する一文を追加(`langInstruction`の動的拡張)。
  2. **実ブラウザ+実aruaru-llm(distilgpt2)で検証**: 日本語入力
     「こんにちは、元気ですか?」→モデルが偶然漢字混じりの文字列を
     生成したケース(構造的にはハイブリッド成立、意味内容は保証外と
     正直に開示)、英語入力「How are you today?」→モデルが英語のみで
     応答したケースでは実際に`ensureHybridReply`のフォールバック日本語
     ノートが自動追記されることを確認した(誇張なし、実際のブラウザ
     操作+実HTTP経由)。
  3. **バージョン管理**(`version.json`): `buildId`(自動更新トリガー用、
     既存)に加え`version`(セマンティックバージョン、新設)フィールドを
     追加。`index.html`にフッター(`#app-version-label`)を新設し、
     `app.js`の`showAppVersion`が`fetch("version.json")`経由で表示する
     (実ブラウザで`v0.2.0`表示を確認済み)。
  4. **旧バージョンのブラウザ側クリーンアップ**(`auto-update.js`):
     **正直な開示・スコープ**——open-englishはネイティブインストーラーを
     持たない静的Webアプリのため、「旧バージョンの自動アンインストール」を
     ディスク上のファイル削除として実装するのは(他ファイルの誤削除
     リスクがあり)危険と判断し見送った。代わりに、Webアプリとして安全に
     実現できる範囲——新バージョン検出時にこのアプリ専用の名前空間
     (`openEnglish.`接頭辞)を持つ`localStorage`キーを削除し
     (`clearOwnLocalStorage`、無関係なブラウザデータには一切触れない)、
     `?v=<buildId>`クエリ付きの再読み込み(`reloadBustingCache`、
     ブラウザのHTTPキャッシュに残る旧JS/CSSではなく新アセットを確実に
     取得させる)を行う、という「旧バージョンの痕跡クリーンアップ」を
     実装した。
  5. **JSON処理について(ユーザー指示「JSONを使っていればRS-JSONに
     切替」への回答)**: `app.js`はブラウザ標準の`JSON.parse`/
     `JSON.stringify`(`fetch().json()`・`JSON.stringify(body)`)を
     使用しており、Rustクレートである`RS-JSON`(`rust-json`)は
     ブラウザJS環境からは利用できない(別言語・別ランタイム)。
     今回新設した`server/`(Rust)クレートは静的ファイルをバイト列の
     まま配信するのみでJSON処理を一切行わないため、該当箇所は無し
     (正直な開示、対応不要と判断)。
  6. **検証**: 実際に`server/`を再起動せず(`static_file_handler`が
     リクエストごとにディスクから読み直す設計のため)、編集した
     `app.js`/`index.html`/`style.css`/`version.json`/`auto-update.js`が
     即座にブラウザへ反映されることを確認した。
  - 次にすべきこと: (1) `README.md`/`README-English.md`/`PORTING.md`の
    今回分の反映(このコミットで実施済み)、(2) git push・
    バージョンタグ(`v0.2.0`)でのGitHub Release作成、(3) 東芝SBM/
    DeepSeek技術組み込み構想の調査(未着手のまま持ち越し)。

- **2026-08-10 リポジトリ着手**: `index.html`/`style.css`/`app.js`による
  静的フロントエンド(Phase 0)を新規作成。`aruaru-llm`の実際のHTTP API
  (`/v1/generate`・`/v1/chat`・`/healthz`、既定ポート4600)をソースから
  確認した上で`/v1/generate`へ接続する形で実装(推測でエンドポイントを
  決めていない)。レベル選択・チャットログ・接続状態表示・CSSアニメー
  ションの喋るキャラクター(独自デザイン)を実装。GitHubへのpushは未実施。
  - 次にすべきこと: (1) GitHubリポジトリ作成・push、(2) 実際に
    `aruaru-llm`を起動して実機で動作確認(同一オリジン配信でのCORS回避
    含む)、(3) `open-directx`/`open-cuda`/`aruaru-llm`のブラウザ対応
    (WASM/WebGPU)調査、(4) `RPoem`との連携方法の検討。
