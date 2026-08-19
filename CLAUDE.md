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

- **2026-08-19(続き11) aruaru-db/PostgreSQLも同時rsyncバックアップ可能に
  (ユーザー指示「RSyncで、open-englishのaruaru-dbとpostgresqlを他の
  デバイスなどにバックアップ同時を可能に、その設定方法も簡単にして」
  への対応)**:
  1. **事前調査**: `aruaru-db`(`F:\runo\aruaru-db`)は独自のストレージ
     エンジン(fjall LSM行ストア + Prolly Tree + WAL、`README.md`
     アーキテクチャ図参照)であり、稼働中のデータディレクトリを
     `rsync`で直接ファイルコピーすると、書き込み中のファイルを
     中途半端な状態で複製する一貫性リスクがあると判断した(PostgreSQL
     本体のデータディレクトリを止めずにそのままrsyncするのが危険なのと
     同じ理由)。また`open-english`は`aruaru-db`のデータディレクトリへ
     直接アクセスできる位置関係にあるとは限らない
     (`OPEN_ENGLISH_DATABASE_URL`はネットワーク越しの接続文字列で
     あり得る)。`aruaru-db`自身が持つ`aruaru-backup`クレート
     (Parquetフル/増分/スナップショットバックアップ)はin-process
     ライブラリで`QueryEngine`インスタンスを直接要求する設計であり、
     アーキテクチャ方針(`open-english`は`aruaru-db`に直接依存せず
     普通のPostgreSQLクライアントとして振る舞う)上、このクレートへ
     直接依存させることはしなかった。
  2. **採用した方式**: 標準の`pg_dump`(PostgreSQLワイヤプロトコル
     経由、単一トランザクションのスナップショットとして一貫性のある
     ダンプを取得する標準クライアントツール)でSQLダンプファイルへ
     書き出した上で、そのダンプファイル1個だけを`rsync`で複製する。
     `server/src/db.rs`に`Db::backup_postgres_via_pg_dump(destination)`
     を新設(`OPEN_ENGLISH_DATABASE_URL`未設定時は`None`を返し
     「対象なし」として扱う)。
  3. **新規`POST /v1/db/rsync-backup-all`**(`server/src/main.rs`):
     宛先を1つ受け取り、既存のSQLite会話DBバックアップと
     `pg_dump`+rsyncバックアップを同時に実行し、両方の結果を
     `{sqlite_backup, postgres_backup}`として返す。**設定を簡単に
     する狙い**の実現方法: 利用者は宛先を1箇所入力するだけでよい
     (既存の`/v1/db/rsync-backup`とは別エンドポイントとして追加、
     既存機能は変更していない)。
  4. **UI(`index.html`/`app.js`/`style.css`)**: 既存の「💾 Data &
     Model Storage」パネル内に「Backup conversation DB + aruaru-db/
     PostgreSQL together / 会話履歴DBとaruaru-db/PostgreSQLを同時に
     バックアップ」節を新設し、宛先1つの入力欄+「🔄 Backup both now /
     両方を今すぐバックアップ」ボタンを追加。`pg_dump`経由である旨・
     データ一貫性への配慮を日英併記で明記した。`.setup-note`に
     `white-space: pre-line`を追加(結果メッセージを2行〈SQLite側/
     aruaru-db側〉で見やすく表示するため)。
  5. **正直な開示・検証範囲の限界(重要)**: (a) この開発機には
     `cargo`/`rustc`がPATH上に無く(既存HANDOFFに記載済みの既知の
     制約)、新設したRustコード(`db.rs`/`main.rs`)は**実際に
     コンパイルして検証できていない**——手作業でのコードレビュー
     (既存の`backup_via_rsync`/`db_rsync_backup`と同型のエラー
     処理・型シグネチャに揃えた)にとどまる。(b) UI側
     (`index.html`/`app.js`/`style.css`)は、既存のビルド済み
     旧バイナリ(新エンドポイント未搭載)を実際に起動し、Claude
     Browserで実機確認した: パネルの新セクションが正しく描画される
     こと、宛先未入力→エラーメッセージ表示、宛先入力→ボタン押下で
     `fetch`が実際に発火し(新エンドポイントが無いため404、期待通り)
     例外を投げずに日英併記の失敗メッセージへフォールバックすること、
     コンソールに新規のJSエラーが出ないこと、を確認した。(c) この
     開発環境に到達可能な`aruaru-db`インスタンス・`pg_dump`バイナリが
     無いため、「`aruaru-db`のpgwire実装が実際に`pg_dump`の要求する
     クエリに対応しているか」自体も未検証(モジュールdocに明記済み)
     ——対応していない場合は`pg_dump`のエラーをそのまま利用者へ
     返す設計にしている(黙って成功したことにしない)。
  - 次にすべきこと: (1) `cargo`が使える環境で`cargo build --release`
    ・`cargo test`を実施し、新規コードの実コンパイル確認、(2) 実際の
    `aruaru-db`インスタンス+`OPEN_ENGLISH_DATABASE_URL`+`pg_dump`が
    揃った環境での`/v1/db/rsync-backup-all`のE2E検証(`pg_dump`が
    `aruaru-db`のpgwireに対して実際に成功するかの確認を含む)、
    (3) rsync実在環境での実ファイル複製確認(既存`/v1/db/rsync-backup`
    と同じ既知の未検証事項の延長)。

- **2026-08-19(続き10) 「使わなくなったスマホもフル動員しましょう」
  日英併記バナーを追加(ユーザー指示への対応、詳細な検出実装は
  `aruaru-llm`側、本リポジトリはUI表示のみ)**:
  1. **`index.html`**: `#free-tier-banner`直下・`.topbar`直上に
     `#phone-accel-banner`を新設。内容は日本語「使わなくなったスマホも
     フル動員しましょう！USBで接続すると、そのスマホのCPU・GPU・NPUを
     モデル圧縮の計算に活用できます」+英語併記、末尾に「実験的機能」
     である旨と現状の実装範囲(PC側NPU自動検出+USB接続台数検出のみ、
     スマホ側の実計算処理は未実装)を明記し誇張しない表現にした。
  2. **バックエンド連携は無し(正直な開示)**: このバナーは静的テキスト
     表示のみで、`aruaru-llm`側新設の`GET /v1/background-fold/status`
     (`accelerators`フィールド、NPU/USB端末検出結果を含む)を`fetch`
     して動的表示する配線は今回行っていない——文言の「実験的機能」注記
     で現状の実装範囲を誇張なく伝えることを優先し、UIの複雑化は
     見送った(次回、実際の検出結果をリアルタイム表示したくなった場合の
     増分として残す)。
  3. **検証**: Claude Browserで`file:///F:/runo/open-english/index.html`
     を開き、`document.getElementById('phone-accel-banner')`が存在し
     (`display:block`)、日英併記の期待テキストが実際に描画されている
     ことをJS経由で確認した。この開発機には`aruaru-llm`実行ファイルが
     無いため、既存のチャット機能自体の実サーバー統合検証(白画面・
     コンソールエラーの有無)は今回のスコープでは実施していない
     (単純な静的HTML追記のみのため影響範囲は限定的と判断)。
  - 次にすべきこと: (1) `GET /v1/background-fold/status`の
    `accelerators`フィールドを実際にfetchしてバナー内に検出結果
    (NPU有無・USB接続台数)を動的表示する、(2) 実際にNPU搭載機・
    Android実機が用意でき次第、`aruaru-llm`側の検出ロジックと
    合わせてE2E検証。

- **2026-08-19(続き9) モデル重み保存先の表示+バックアップ/復元UI(日英併記)を実装
  (ユーザー指示「Windows/Android両方でモデル重みの保存先明示・可能なら同期
  バックアップを、日英併記UIで」への対応)**:
  1. **事前調査**: モデル取得自体(Windows: `app.js`の`installAndSwitchModel`
     経由でaruaru-llmの`/v1/models/install`、Android: `MainActivity.kt`の
     `downloadModelsAndRestartAruaruLlm`)は既に他セッションで実装済みと判明
     (2026-08-17)。また会話履歴・設定DBの保存先変更・rsyncバックアップAPI
     (`/v1/db/storage-path`・`/v1/db/rsync-backup`・`/v1/db/install-rsync`)も
     2026-08-18に実装済みだったが、`app.js`側から未接続のまま残っていた
     (HANDOFF記載の既知のギャップ)。今回はこのギャップを埋めることに
     スコープを絞った。
  2. **`index.html`/`app.js`/`style.css`**: 新規「💾 Data & Model Storage /
     データ・モデルの保存先」パネルを追加し、既存の`/v1/db/info`・
     `/v1/db/storage-path`・`/v1/db/rsync-backup`・`/v1/db/install-rsync`
     を初めてUIから呼び出せるようにした。**正直な開示**: このパネルが
     直接バックアップ・移動できるのは会話履歴・設定のSQLite DBのみ——
     `aruaru-llm`本体のモデル重み自体は別リポジトリ`aruaru-llm`の
     `/v1/models/*`が管理しており対象外である旨をパネル冒頭に日英併記で
     明記した。
  3. **Android(`MainActivity.kt`/`activity_main.xml`/`strings.xml`)**:
     モデル重みの内部ストレージ保存先(`filesDir/aruaru-llm-models`)を
     常設のバイリンガルTextViewで表示するようにした(従来はダウンロード中
     の一時的なステータス表示にしか現れなかった)。また「💾 モデルを外部
     ストレージへバックアップ / Back up model to external storage」・
     「📂 復元 / Restore」ボタンを新設し、`getExternalFilesDir`(権限不要、
     ファイルマネージャから参照可能なアプリ専用領域)へモデル一式を
     再帰コピーする簡易バックアップを実装した。**正直な開示**: クラウドへ
     直接アップロードする機能ではない——利用者が復元先フォルダを
     Googleドライブ等へ自分でアップロードする運用を想定した現実的な設計。
  4. **実機検証**: Rustソース側の変更は無い(既存バイナリ
     `server/target/release/open-english-server.exe`をそのまま利用、この
     開発機に`cargo`が無い制約は従来通り)。ポート4711で起動しClaude
     Browserで実際に(a) パネルを開き`/v1/db/info`の実データ(DBパス・
     ファイルサイズ・aruaru-dbミラー無効)が表示されること、(b)
     バックアップ先未入力状態で「今すぐバックアップ」を押すと`rsync`未検出
     を検出し、「Let's install RSync! / RSyncをインストールしましょう！」+
     「⬇ Install RSync」ボタンの表示切替が正しく動くことを確認した。
     **未検証**: Android側(`backupModelsBtn`/`restoreModelsBtn`)は
     この開発機にAndroid実機/エミュレータが無いためコードレビューレベルの
     確認のみで、実機ビルド・実行はできていない——次回Android実機/
     エミュレータで確認する必要がある。またrsyncが実在する環境での
     「実際にファイルがコピーされる」E2Eも引き続き未検証(既知の制約の
     まま)。
  - 次にすべきこと: (1) Android実機/エミュレータでのバックアップ/復元
    ボタンのE2E検証、(2) rsyncが実在する環境での`/v1/db/rsync-backup`の
    実ファイルコピーE2E検証、(3) モデル重み自体(会話DBではなく)の
    Windows版rsyncバックアップが必要か、次回ユーザーへ要否確認。

- **2026-08-19(続き8) 1日の利用回数制限に到達した際の日英併記メッセージを
  実装(ユーザー指示「検索や質問などで1日の利用回数制限を超えた場合に、
  有料版切替の案内+他プロバイダの無料枠案内を日英併記で表示して」への
  対応)**:
  1. **事前調査結果**: `app.js`・`server/src/main.rs`・`db.rs`のいずれにも
     「1日100回まで」という自前の利用回数カウンタは実装されていなかった
     (`index.html`内の「1日100件まで無料」という記述はGoogle Custom
     Search JSON API自体の無料枠説明であり、open-english自身の制限では
     ない、という正直な事実確認)。そのためまず`app.js`側に
     `localStorage`ベースの簡易日次カウンタ(`DAILY_USAGE_LIMIT_KEY =
     "openEnglish.dailyUsage"`、上限`DAILY_USAGE_LIMIT = 100`、日付が
     変わると自動リセット)を新規実装した。**正直な開示**: これは
     クライアント側のみのカウンタであり`localStorage`消去や別端末利用で
     回避可能——サーバー側での強制ではなく、利用者への通知目的の
     簡易的な仕組み。
  2. **`isDailyLimitExceeded()`/`recordDailyUsage()`**: チャット送信
     フォーム(`formEl`の`submit`ハンドラ)冒頭でチェックし、上限到達時は
     `askTrainer`等のAPI呼び出しを一切行わずに`dailyLimitExceededMessage()`
     の結果を`appendMessage("system", ...)`で表示するのみに留める
     (通常モード・メイドカフェ研修モードの両方に適用)。
  3. **`dailyLimitExceededMessage()`**: 要望1(「本日の無料利用枠を
     超えました。有料版に切り替えますか？」/ "You've exceeded today's
     free usage limit. Would you like to switch to a paid plan?"、実際の
     決済・アップグレード導線は実装していない旨も併記)+要望2(他社
     無料枠の案内)を日英併記で構築。要望2は既存の
     `provider-free-tiers.json`を`fetch`して動的に組み立てる設計
     (ハードコードなし、既存の無料枠バナーとの一貫性を維持)。
  4. **テスト用フック**: `window.OPEN_ENGLISH_DAILY_LIMIT_OVERRIDE`
     (既定`null`)で上限を一時的に下げられるようにし、実機検証を
     容易にした(本番運用では未設定のまま`DAILY_USAGE_LIMIT`が使われる)。
  5. **実機検証**: 他エージェントの並行作業で元々起動していた
     `open-english-server.exe`(ポート4601)がこのセッション中に停止して
     いたため、既存バイナリを`OPEN_ENGLISH_SERVER_BIND=127.0.0.1:4699`で
     別ポート起動し(他エージェントの作業に干渉しないよう配慮)、Claude
     Browserで実際に(a) `OPEN_ENGLISH_DAILY_LIMIT_OVERRIDE=1`+
     使用済みカウント`1`を`localStorage`へ設定した状態でチャット送信→
     `.msg.system`として日英併記の上限到達メッセージ(有料版切替の案内+
     5プロバイダ全ての無料枠情報)が正しく表示されることを確認、
     (b) カウントを`0`にリセットした状態では`isDailyLimitExceeded()`が
     `false`を返し通常のチャット送信フロー(`askTrainer`呼び出し)へ
     進むことを確認した。検証後、テスト用に起動したサーバー
     プロセス(ポート4699)は`taskkill`で終了済み。
  6. **変更範囲**: `app.js`のみ(106行追加、既存の`askTrainer`本体・
     `provider-free-tiers.json`・`style.css`の`.msg`系クラスは変更
     せず流用、他エージェントが同時に編集中だった`installer/windows/`・
     `server/src/main.rs`・`fetch-aruaru-db.ps1`には一切触れていない)。
  - 次にすべきこと: (1) サーバー側(`server/`)でも同種のレート制限を
    課すか検討(現状はクライアント側のみで回避可能な簡易実装)、
    (2) 「有料版に切り替えますか？」に対する実際の課金・アップグレード
    導線の実装(今回は意図的にスコープ外、ユーザー確認の上で着手)、
    (3) `DAILY_USAGE_LIMIT`(100)の値が実際の運用ニーズに合っているか
    ユーザーへ確認。

- **2026-08-19(続き7) Windows版のコマンド操作ゼロ化(aruaru-llm自動起動)+
  README-INSTALLED.txtの矛盾を解消(ユーザーがGitHub上で読んだ
  `README-INSTALLED.txt`「aruaru-llmは別途手動でダウンロード・起動が
  必要」という記述への「含めて下さい。コマンド操作を不要にして下さい」
  指摘への対応)**:
  1. **矛盾の正体を特定**: `installer/windows/open-english.iss`の
     `installaruarullm`タスク(既定オン)は`fetch-aruaru-llm.ps1`で
     aruaru-llm本体を`{app}\aruaru-llm\`へ**取得**していたが、
     **起動する処理がどこにも無かった**(`[Run]`セクションは
     `open-english-server.exe`しか起動しない、`fetch-aruaru-llm.ps1`も
     ダウンロード・展開のみ)。つまり「取得はするが起動しない」が
     矛盾の実体——`README-INSTALLED.txt`の「含まれていません」という
     記述自体は古かったが、「コマンド操作が必要」という結論は
     皮肉にも正しかった。
  2. **`server/src/main.rs`に`maybe_launch_aruaru_llm()`を新設**:
     サーバー起動時、`http://127.0.0.1:4600/healthz`(`ARUARU_LLM_BIND`
     で上書き可)へ到達できなければ、実行ファイルと同じディレクトリの
     `aruaru-llm\aruaru-llm.exe`(Windows)/`aruaru-llm/aruaru-llm`
     (Linux/macOS、`fetch-aruaru-llm.sh`と同じ相対配置)を子プロセスと
     して自動起動する。インストール時の一度きりではなく、
     **ショートカットを実行するたび毎回**この判定が走るため、
     「インストール直後だけ起動している」という見せかけを避けている。
     バイナリが存在しない場合(「まとめてインストール」を外した、
     または未取得)はエラーにせずログのみで正直にスキップする(既存の
     フロントエンド側`checkHealth`が接続状態を正直に表示する設計に
     委ねる)。
  3. **`README-INSTALLED.txt`を全面的に書き直し**(日英併記):
     「aruaru-llmも一緒にインストール」を選べばコマンド操作ゼロで
     チャットまで到達できる旨、外した場合の手動手順、モデル重み別途
     取得の制約、Windows向けアセットが見つからない場合の制約を正直に
     明記。**Android版の記述も併せて訂正**——調査の結果、2026-08-11の
     アーキテクチャ変更で既にAndroid版は単体動作版(サーバー・
     aruaru-llmともに端末内蔵)になっており、PCのIPアドレス入力は
     既に不要になっていた(`android/app/src/main/res/layout/
     activity_main.xml`のコメント、`app.js`の`location.hostname`
     自動補完コメント参照)。つまりユーザー要望2番(IPアドレス入力不要
     化)は**Android用ネイティブアプリについては既に他セッションで
     実現済み**で、今回は古いまま放置されていた案内文の訂正のみで
     足りると判明した——実装漏れではなく、ドキュメントが実態に
     追従していなかったことが原因。PCブラウザ経由でアクセスする
     場合(Androidアプリではなく)も`app.js`の`apiBaseEl`が
     `location.hostname`から自動補完する既存実装がある旨を参考情報
     として追記した。
  4. **実機E2E検証(型チェック・ビルド成功だけで完了と報告しない
     方針の徹底)**: (a) `cargo build --release`成功。(b)
     `server/target/release/aruaru-llm/aruaru-llm.exe`が存在しない
     状態で起動→ログに「aruaru-llm binary not found ... skipping
     auto-launch」と正直に出ることを確認。(c) 依存無しの最小スタブ
     exe(`ARUARU_LLM_BIND`で待受・`/healthz`に200を返すのみ)を同じ
     パスへ配置して再起動→ログに「auto-launched aruaru-llm (pid …)」
     と出て実際に`http://127.0.0.1:4600/healthz`へ到達できることを
     確認。(d) **本番と同じ経路での一気通貫検証**: `ISCC.exe`で
     実際にインストーラーをビルドし、`/VERYSILENT /DIR=<一時
     フォルダ>`で完全無人インストール→**コマンド操作を一切行わずに**
     `open-english-server.exe`(PID確認済み)・`aruaru-llm.exe`(PID
     確認済み)の両プロセスが自動的に立ち上がり、`http://
     127.0.0.1:4601/healthz`(`{"ok":true}`)・`http://
     127.0.0.1:4600/healthz`(200)の両方に実HTTPで到達できることを
     確認した。検証後、両プロセスを終了し`unins000.exe /VERYSILENT`
     でアンインストール・一時フォルダの削除も確認済み。
  5. **他エージェントとの並行作業への配慮**: 着手前に`git log`/
     `git status`を確認したところ、`README-English.md`等の翻訳
     ファイル群・`android/oe_rot1.png`が他エージェントの未コミット
     作業として存在していたため一切触れていない。また作業中に
     `README-INSTALLED.txt`・`open-english.iss`へ別エージェントが
     aruaru-db(任意コンポーネント)対応を並行して加えていたことが
     判明したため、`README-INSTALLED.txt`の書き直しはその内容
     (aruaru-dbの節)を保持したまま自分の変更を統合した。`app.js`にも
     別エージェントによる「1日の利用回数制限」機能の未コミット変更が
     あったため、このコミットには含めていない(自分が触っていない
     ファイルとして除外)。
  6. **正直な開示・今回のスコープ外**: (a) Android側の「IPアドレス
     入力不要化」は上記3番の通り既に実現済みと判明したため、新規の
     UDPブロードキャスト/mDNS/サブネットスキャン方式の実装は行って
     いない——もし将来的にPCサーバーへブラウザ経由で接続する
     ケースでの自動検出(現状はホスト名からの単純補完のみ)を
     強化したい場合は、別途スコープを切って上記いずれかの方式を
     検討すること。(b) Linux/macOS版(`installer/unix/`)は
     `fetch-aruaru-llm.sh`が既にaruaru-llm本体を取得する設計だが、
     今回の自動起動ロジック(`maybe_launch_aruaru_llm`)はWindows/
     Linux/macOS共通コード(`cfg!(target_os = "windows")`でファイル名
     のみ分岐)として実装したため理論上は動作するはずだが、この
     開発機がWindowsのため**Linux/macOS実機でのE2E検証はまだ
     行っていない**——次回、該当環境で確認する必要がある。
  - 次にすべきこと: (1) Linux/macOS実機での`maybe_launch_aruaru_llm`の
    E2E検証、(2) PCブラウザ経由アクセス時の自動検出強化(UDP
    ブロードキャスト等)が必要かどうかの要否確認、(3) 他エージェントの
    `app.js`(1日利用回数制限)・翻訳ファイル群のコミットは各担当
    セッションに委ねる。

- **2026-08-19(続き6) Windowsインストーラーへaruaru-db(任意)の同梱オプションを
  追加(ユーザー指示「aruaru-dbも同梱して」への対応、直前の
  aruaru-llm完全同梱作業とは別スレッドで並行実施)**:
  1. **aruaru-dbの位置づけを`aruaru-db/CLAUDE.md`・`README.md`から
     確認**: `aruaru-db`は「PostgreSQLワイヤプロトコル互換のPure Rust
     実装データベース」で、外部にPostgreSQL本体を別途インストールする
     必要はない(pgwireを自前実装、`aruaru-server`という単体バイナリ
     として動作)。open-english側は`server/src/db.rs`が既に
     SQLite(ローカルファイル)で会話履歴・設定を完結させており、
     aruaru-dbは`OPEN_ENGLISH_DATABASE_URL`環境変数を設定した場合にのみ
     有効になる**ベストエフォートの追加ミラー先**(未設定ならSQLite単体で
     問題なく動作)。この位置づけを誤認せず、「aruaru-db同梱=DB機能の
     必須要件化」ではなく「任意のオプション機能の同梱」として実装した。
  2. **同梱の実現可能性**: `aruaru-db`の`.github/workflows/release.yml`
     (`softprops/action-gh-release`)がWindows/Linux向けのビルド済み
     単体バイナリを`aruaru-server-windows-x86_64.zip`等としてGitHub
     Releasesへ既に添付していることを確認(`aruaru-llm`と全く同じ
     配布パターン)。フルPostgreSQLサーバー自体の同梱が必要なケース
     ではなかった。
  3. **実装**: `installer/windows/fetch-aruaru-db.ps1`を新設
     (`fetch-aruaru-llm.ps1`と同じ設計、GitHub Releases APIから
     Windows向けzipを取得・展開、失敗してもインストーラー全体は止めない)。
     `open-english.iss`へ`[Files]`(`fetch-aruaru-db.ps1`)・`[Tasks]`
     (`installaruarudb`、**既定は未チェック**——aruaru-llmと異なり
     open-english本体の動作に必須ではないため)・`[Run]`
     (該当タスク選択時のみ`fetch-aruaru-db.ps1`を実行)を追加。
     取得したaruaru-serverは自動起動しない(ユーザー自身が起動し
     `OPEN_ENGLISH_DATABASE_URL`を設定する必要がある、手順は
     `README-INSTALLED.txt`に日英併記で追記済み)。Tauri管理GUI・
     Raft分散クラスタ構成・バックアップ運用等は同梱対象外(単体
     サーバーバイナリのみ)であることも明記した。
  4. **実装できなかった点・正直な開示**: この開発機に`ISCC.exe`
     (Inno Setup Compiler)が無く、`.iss`ファイル自体の実コンパイル・
     生成されたインストーラーでの実インストール検証はできていない
     (PowerShellスクリプト単体の構文チェックのみ実施、エラー無し)。
     次回、Inno Setupが利用可能な環境で実際に`ISCC.exe
     open-english.iss`を実行し、「aruaru-dbも一緒にインストール」
     チェック時に実際に`aruaru-server.exe`がダウンロード・配置される
     ことのE2E検証が必要。
  5. **他エージェントとの並行作業への配慮**: 着手前に`git status`を確認
     したところ、別エージェントが`server/src/main.rs`(aruaru-llm自動
     起動機能)・翻訳系ファイル群を並行編集中だったため、それらには
     一切触れず、`installer/windows/open-english.iss`(このセッション
     以前は未編集だった)への追記のみに限定した。`README-INSTALLED.txt`
     は他エージェントも同時に大幅更新していたため、このセッションでは
     aruaru-db向けの段落追記のみを行い(他エージェント側の変更を
     上書き・競合させないよう確認)、コミットは`open-english.iss`と
     新設`fetch-aruaru-db.ps1`の2ファイルに限定し、他エージェントの
     変更が残る`README-INSTALLED.txt`・`server/src/main.rs`は今回
     コミット対象から外した(他エージェントのコミット完了後に別途
     整合を確認すること)。
  - 次にすべきこと: (1) Inno Setup実機での`.iss`コンパイル+
    「aruaru-dbも一緒にインストール」チェック時の実インストールE2E検証、
    (2) 他エージェントの`README-INSTALLED.txt`変更が確定コミットされた
    後、このセッションで追記したaruaru-db段落が正しく残っているか確認、
    (3) Linux/macOS版インストーラー(`installer/unix/install.sh`)にも
    同様の`--with-aruaru-db`オプションを追加するかの検討(今回は
    Windows版のみ)。

- **2026-08-19(続き5) 無料枠バナーへClaude(Anthropic)を追加(ユーザー指示
  「最初から有料でも良ければClaudeも選択可能にしておいて」への対応)**:
  1. **`provider-free-tiers.json`にClaudeエントリを追加**: `id: "claude"`。
     他社(DeepSeek/ChatGPT/Gemini)と違い恒常的な無料枠が基本的に無い点を
     誇張せず正直に記載——「新規登録時のごく少額・期限付きクレジットが
     付与される場合があるが金額・有無は保証しない、継続利用はクレカ
     登録+従量課金が前提」という趣旨を日英併記。出典は
     `https://www.anthropic.com/pricing`。
  2. **`index.html`/`app.js`側の変更は不要と判明**: 直前のセッションで
     実装済みのバナーは`provider-free-tiers.json`の`providers`配列を
     `fetch`して動的に描画する設計(プロバイダ名のハードコードなし)
     だったため、JSON側にエントリを追加するだけでClaudeが一覧へ
     自動的に反映される。
  3. **実機検証**: 既に別セッションが起動していた
     `open-english-server.exe`(`http://127.0.0.1:4601/`)に対し実際に
     `curl`で`GET /provider-free-tiers.json`を叩き、200 OKで応答本文に
     `"id": "claude"`が含まれることを確認した。**正直な開示**: この
     セッションのサンドボックス制約により、Browserツールから
     `http://127.0.0.1:4601/`への`navigate`が拒否され、実ブラウザでの
     バナー内Claude行の目視確認は今回できていない——次回、通常の
     開発機ブラウザで`💰 AI/Search free tiers`バナーを開き、Claudeの
     行が正しく表示されることを目視確認する必要がある。またこの開発機
     には`cargo`がPATH上に無く`cargo build`自体は実行できなかった
     (既存のビルド済みバイナリで検証、ソース側`server/`の変更は今回
     無いためビルド不要と判断)。
  - 次にすべきこと: (1) 実ブラウザでバナーを開きClaude行の表示を目視
    確認、(2) Claude(Anthropic)以外にも有料前提で追加したいプロバイダが
    無いか、次回ユーザーへ確認。

- **2026-08-19(続き4) AI/検索プロバイダの無料枠情報バナーを追加(ユーザー
  指示「それらの無料枠については、メンテナンス時に情報を得て、
  open-englishの上の方に表示して」への対応、DeepSeek/ChatGPT/Gemini
  自体をプロバイダ選択肢に加える件は今回のスコープ外)**:
  1. **正直な設計判断**: 各AIプロバイダの無料枠は変更されやすく、
     機械可読なAPIで残り回数をリアルタイムに正確取得できる仕組みは
     一般に存在しない(各社とも無料枠情報をAPIとして公式公開している
     わけではない)。自動スクレイピングで常時最新を保証すると称する
     実装は過大な約束になるため避け、代わりに「開発者が定期メンテナンス
     時に手動更新する設定ファイル+それを表示するUI」という現実的な
     設計にした。
  2. **新規`provider-free-tiers.json`**(リポジトリ直下): Google検索
     (Custom Search JSON API)・DeepSeek・ChatGPT(OpenAI API)・Google
     Gemini・Claude(Anthropic、参考として追加)の5件、`last_updated`
     フィールド付き。2026-08-19時点でWeb調査した内容を初期値として
     埋めた(いずれも「researched」——調べられず「要確認」とすべき項目は
     今回は無かった):
     - Google Custom Search JSON API: 1日100クエリ無料(追加$5/1000件、
       上限1万件/日)。**新規顧客への提供は終了しており、既存利用者も
       2027-01-01までに代替手段への移行が必要**という重要な現状も明記。
     - DeepSeek API: 新規登録時に500万トークンを1回限り無料付与
       (30日間有効)。常時無料モデルは無く、消費後は従量課金
       (2026-08-16よりピーク/オフピーク別料金制へ変更)。
     - ChatGPT(OpenAI API): 自動無料クレジットは2025年半ばに廃止済み。
       現在は新規アカウントに$5分のクレジット(3か月で失効)、クレジット
       無しならGPT-3.5 Turbo限定・3RPMのみ。データ共有オプトイン時は
       GPT-5系で最大1日100万トークンの無料枠あり。
     - Google Gemini API: モデル別(Gemini 2.5 Pro/Flash/Flash-Lite)に
       5〜15RPM・1日100〜1000件、共通25万TPM。2025年12月にクォータが
       引き下げられた経緯あり、無料枠利用時はプロンプトがGoogle側の
       改善に使われ得る点も明記。
  3. **`index.html`**: 既存の`#maintenance-banner`(固定60秒で自動的に
     隠れる)とは別に、`#free-tier-banner`(常設・折りたたみ式トグル
     ボタン)を新設。既存バナーの直下に配置。
  4. **`app.js`**: `showProviderFreeTiers()`(IIFE)を新設し、
     `provider-free-tiers.json`を`fetch`して各プロバイダ名・無料枠内容・
     最終更新日を日英併記でリスト表示。外部/JSON由来テキストのため
     `innerHTML`は使わず`textContent`で組み立てる(既存の`referralsSuffix`
     等と同じXSS回避方針)。読み込み失敗時も他機能に影響させない
     (`catch`で握りつぶすのみ、既存の可用性優先方針を踏襲)。
     UI内にも「※この情報は手動更新運用であり自動取得ではない、
     最新情報は各社公式サイトで」という日英併記の注記を明記。
  5. **`style.css`**: `.free-tier-banner`/`.free-tier-toggle`/
     `.free-tier-body`等を新設(既存の`.maintenance-banner`と衝突しない
     独立クラス)。
  6. **`server/src/main.rs`**: `STATIC_FILES`へ`/provider-free-tiers.json`
     を追加(既存`static_file_handler`の再利用のみ、新規ロジック無し)。
  7. **実機検証**: `cargo build --release`成功。実際にサーバーを起動し
     `Invoke-WebRequest`で`/provider-free-tiers.json`が実HTTP経由で
     200・全プロバイダ情報を返すことを確認。さらにブラウザ(Claude
     Browser)で実際にページを開き、トグルボタンクリックで
     `#free-tier-body`の表示/非表示が切り替わること、
     `#free-tier-list`に5件・`#free-tier-last-updated`に
     `2026-08-19`が正しく反映されることをJS経由で確認した。
  8. **他エージェントとの並行作業への配慮**: 同時に他エージェントが
     `README.md`/`server/src/main.rs`/`server/src/self_update.rs`を
     編集中だったため、着手前に`git status`/`git diff`で状態を確認し、
     `main.rs`への変更は`STATIC_FILES`への1行追加のみに限定、
     `README.md`・`self_update.rs`には一切触れていない。
  - 次にすべきこと: (1) DeepSeek・ChatGPT・Geminiを実際の検索/AI応答
    プロバイダとして選択できるようにする本体機能(ユーザーの別アイデア、
    今回のスコープ外)、(2) `provider-free-tiers.json`の定期的な手動更新
    (自動化されていない運用である旨を忘れないこと)、(3)
    `README.md`/`README-English.md`への反映(今回は`CLAUDE.md`のみ、
    索引的な性質が薄いため見送ったが、次回気になる場合は追記)。

- **2026-08-19(続き3) macOS対応の自己アップデート機能+自動ロールバック
  (ダウングレード)機能を追加(ユーザー指示「MAC版にも対応、失敗した時
  ダウングレードする機能も搭載」への対応)**:
  1. **macOS対応**: `server/src/self_update.rs`の`apply_update_linux`を
     `apply_update_unix`へリネームし、`cfg!(any(target_os = "linux",
     target_os = "macos"))`を前提とする共通コードへ一般化(Linux/macOSは
     Unix系ファイルシステムの仕組み上、実行中バイナリのその場置き換えが
     同じ手順で可能なため、重複実装を作らず1つの関数を共有)。アセット
     探索関数も`linux_tarball_asset`(macOSを明示的に除外していた)から
     `unix_tarball_asset`へ改名し、実行中のOSに応じて`"linux"`/`"macos"`
     を含むtarball名を振り分けるよう変更。`check_and_apply_update`冒頭の
     プラットフォームガードも`Windows/Linux`→`Windows/Linux/macOS`へ
     拡張(Android/iOSは引き続き対象外、理由はモジュールdoc・過去の
     HANDOFF参照)。`installer/unix/install.sh`は元々`uname -s`で
     Linux/macOS両対応済み(`.desktop`ランチャー作成のみLinux限定に
     分岐)だったため変更不要と確認した。
  2. **自動ロールバック(ダウングレード)機能**: Windows/Linux/macOSの
     いずれも、新バージョン適用前に現在のバイナリ(Linux/macOSは
     `open-english-server.bak`、Windowsはインストールディレクトリ全体を
     一時フォルダへ)を退避するようにした。新バージョン起動後、
     `server/src/main.rs`に新設した`/healthz`エンドポイント(既存の
     `aruaru-llm`側`/healthz`と同じ命名に統一)へ`HEALTH_CHECK_SECS`
     (12秒)の猶予以内に到達できるか、または新プロセスが直後に終了して
     いないかを確認し、失敗すれば新プロセスを終了させて退避しておいた
     バイナリ/ディレクトリを復元し、旧バージョンで動作を継続する。
     複雑な状態機械は組んでおらず、「起動→N秒待つ→ヘルスチェック→
     失敗ならkill+restore+restart」という単純な線形フローのみ
     (ユーザー指示「シンプルに」を踏まえた設計)。
     - **Linux/macOS**: この関数を呼ぶプロセス自身(=旧バージョン)が
       まだ実ポートで稼働し続けているため、新バイナリはまず実ポート+1の
       一時プローブポート(`OPEN_ENGLISH_SERVER_BIND`環境変数で上書き)で
       起動してヘルスチェックし、通れば改めて実ポートで正式起動して
       から旧プロセス自身が終了、失敗すればプローブを`kill`して
       バイナリを復元するだけで良い(旧プロセスを一度も止めていない
       ため、復旧のための追加の再起動処理が不要という設計上の利点)。
     - **Windows**: 既存の「デタッチしたバッチスクリプトがアン
       インストール→インストールを行う間に自分自身は終了する」という
       設計上、ヘルスチェック・ロールバック判断もこのバッチスクリプト
       側へPowerShellの`Invoke-WebRequest`呼び出しとして追記した
       (失敗時は`taskkill`でアプリを止め、`xcopy`でバックアップ
       ディレクトリを復元して再起動)。
  3. **Android/iOSでのロールバックについて(正直な開示)**: OSの制約上
     APKの完全サイレント自動インストール自体ができない(既存の
     モジュールdocに記載済みの制約と同一理由)ため、「失敗時に自動で
     旧APKへダウングレードする」機能も実現不可能。本ロールバック機能は
     ネイティブサーバーバイナリを自己置換するWindows/Linux/macOSに
     限定されるスコープであり、Android/iOSアプリ自体の更新は引き続き
     ユーザー自身のタップによる手動インストールのままである旨を
     `self_update.rs`のモジュールdocへ明記した(この制約はコード上の
     不備ではなくAndroid/iOSのプラットフォーム制約そのもの)。
  4. **検証**: `cargo build`(devプロファイル)成功、`cargo test`
     (self_update::2件+db::3件)全green。実際にサーバーを起動し
     `Invoke-WebRequest http://127.0.0.1:4601/healthz`が実HTTP経由で
     `200`・`{"ok":true}`を返すことを確認した。**正直な開示・未検証
     事項**: この開発機はWindowsのため、(a) macOS実機での「新バージョン
     検出→ダウンロード→自己置換→再起動」の一連のE2E検証、(b) Linux/
     macOS双方でのロールバック機構(プローブ起動→ヘルスチェック失敗→
     復元)の実機E2E検証、(c) Windows側のバッチスクリプトによる
     ヘルスチェック→ロールバックの実機E2E検証(Inno Setupインストーラー
     が実際に動く環境+意図的に壊れたビルドを使った検証が必要)、
     の3点はいずれも次回、該当プラットフォーム/リリース環境で実施する
     必要がある——今回はコンパイル成功・単体テスト・`/healthz`単体の
     実HTTP確認までにとどまる。
  5. **`README.md`更新**: 自動アップデート機能の説明を「Windowsのみ」→
     「Windows/Linux/macOS対応、失敗時は自動ロールバック」へ更新し、
     Android/iPhone/iPadは引き続き対象外である旨を明記した。他言語版
     README(`README-English.md`等)は今回未更新(次回、日本語版との
     内容乖離が気になる場合に追従させる)。
  - 次にすべきこと: (1) 上記4番(a)〜(c)の実機E2E検証、(2) 他言語版
    READMEへの反映、(3) Windows側ロールバックの`taskkill`は実行
    ファイル名の完全一致に依存するため、複数バージョンが同時に
    存在する状況での誤爆が無いか実機で確認すること。

- **2026-08-19(続き) 自動アップデート機能をWindows限定からWindows+Linux
  対応へ拡張(ユーザー指示「(旧更新2026-08-11の)Windowsのみ、の所を
  Android・iPhone・Linuxでも使えるようにしてほしい」への対応)**:
  1. **プラットフォームごとの正直な評価(実装前の技術調査)**:
     - **Linux**: 実行中のバイナリ自身を別バイナリで置き換えることは
       Windowsと異なり技術的に可能(Linuxでは実行中の実行ファイルを
       上書き・rename しても、既に起動中のプロセスは古いinodeを掴んだ
       まま動作を続けられ、ファイルロックの制約が無い)。**実装した**。
     - **Android**: OS自体がAPKの完全サイレント自動インストールを
       許可しない(`REQUEST_INSTALL_PACKAGES`+ユーザーの明示的な確認
       タップが必須)。調査の結果、`MainActivity.kt`の
       `checkForAppUpdate`/`downloadAndInstallApk`が2026-08-17時点で
       既に「新バージョン検出→タップでAPKダウンロード→Android標準
       インストーラー画面を開く(最終確認はユーザー自身)」という、
       OS制約内で可能な最大限の自動化まで実装済みだった——**今回の
       スコープでは追加の実装は不要**(既存実装を確認し現状維持)。
     - **iPhone/iOS**: このリポジトリに`ios/`相当のネイティブアプリは
       存在しない(2026-08-19確認)。iOSアプリ自体が実在しないため
       自己更新機構の実装対象が無く、「完全自動インストール」と
       偽ることを避けるため実装しなかった。Webアプリとして使う場合は
       既存の`auto-update.js`(`version.json`ポーリング→
       `location.reload()`)がSafari等でも同様に動作する。
  2. **`server/src/self_update.rs`変更**: 冒頭ガードを
     `cfg!(target_os = "windows")`単独から`windows || linux`へ拡張。
     `linux_tarball_asset`(CI`.github/workflows/release.yml`の
     `build-unix-installer`が生成する`open-english-linux-x86_64.tar.gz`
     命名を検出)・`apply_update_linux`(tarball展開→
     `installer/unix/install.sh`と同じフラット構成で現在の実行
     ディレクトリへ上書きコピー→新バイナリを子プロセスとして起動→
     自身終了)を新設。Windows版と異なりLinux版にはアンインストーラー
     手順が無い(root権限不要のユーザー空間コピーインストールの
     ため単純な上書きで十分)。
  3. **検証**: `cargo build --release`成功、`cargo test --release`
     (self_update::以下2件)green。実際にサーバーを起動し
     `Invoke-WebRequest`で`/version.json`への到達を確認、ログにも
     従来通りの安全な「開発ビルドのため更新チェックをスキップ」
     メッセージが出ることを確認した。**正直な開示**: この開発機は
     Windowsのため、Linux版の「新バージョン検出→実際にダウンロード→
     自己置換→再起動」という一連の流れの実機E2E検証はできていない
     (コンパイル成功・単体テストの確認までにとどまる)——次回、
     実際のLinux環境(VPS等)で確認する必要がある。
  - 次にすべきこと: (1) 実際のLinux環境(conoha VPS等)でLinux版
    自動更新のE2E検証(新バージョンをリリースし、旧バージョンが
    実際に自己置換されることを確認)、(2) 将来iOSネイティブアプリを
    新設する場合は、Apple審査ポリシー上サードパーティによる
    バイナリ自己差し替えが原則禁止されている制約を踏まえた別方式
    (App Store経由の通常アップデートのみ)を前提に設計すること。

- **2026-08-19 Facebook経由の入口ページを新設(ユーザー指示「外国で
  スマホ契約がFacebookしかアクセスできない人でも使えるように、
  Facebookにアクセスしながら使用する仕様に変更、かつ自分の端末へ
  インストーラー付きアプリとしてダウンロードして自らサーバーを立てて
  利用してもらう仕様」への対応)**:
  1. **正直な開示(最重要、実装前の技術調査結果)**: Facebookの
     「Free Basics」等のゼロレーティングプログラムは、Meta社との公式な
     提携・ホワイトリスト登録があるサイトのみが対象であり、
     このプロジェクト側だけで一方的にopen-englishをその対象へ登録する
     ことはできない。そのためユーザー要望のうち「Facebookのみの契約でも
     完全無料でこのアプリにアクセスできるようにする」という部分は、
     現状の技術・権限の範囲では実現不可能——実装したと偽らず、この
     制約を`facebook.html`内・本HANDOFFの両方に日英併記で明記した。
  2. **新規`facebook.html`**: Facebookページの投稿やMessengerの
     チャットで共有するリンク先として使うことを想定した入口ページ。
     Facebookアプリ内蔵ブラウザから開ける通常のWebページとして実装
     (Facebook Instant Games SDK等の組み込みは不要・対象外——単なる
     リンク遷移で足りるため)。内容は(a)なぜこのページが存在するかの
     説明、(b)上記1番の正直な開示、(c)Facebookはあくまで入口であり
     アプリ本体はFacebook内では動作せず利用者端末のローカルサーバー
     (`server/`crate)で動くという説明、(d)既存README記載の
     Windows/Linux・macOS/Androidインストーラーへのダウンロード表
     (README.mdの表と同じリンクを再利用、新規インストーラー機構は
     作っていない——既存の`installer/windows`・`installer/unix`・
     `android/`をそのまま案内)。
  3. **配線**: `server/src/main.rs`の`STATIC_FILES`へ`/facebook.html`を
     追加(既存`static_file_handler`をそのまま再利用、新規ロジック無し)。
     `index.html`のフッターへ「Access via Facebook / Facebook経由で
     アクセス」という1行リンクのみ追加(既存の他エージェントによる
     モデル配置UI作業との衝突を避けるため、`index.html`/`app.js`への
     変更はこの1行に限定し大規模書き換えはしていない)。`style.css`に
     `.fb-entry`/`.download-table`を新設(既存`.setup-panel`を静的
     ページ内で使うための軽微な上書きのみ)。
  4. **実機検証**: `cargo build --release`成功。実際にバイナリを起動し
     `Invoke-WebRequest`で`/facebook.html`(200・5520バイト)・
     `/version.json`(200・`buildId`反映)を実HTTP経由で確認した。
     **正直な開示**: Facebookアプリ実機(実際のFacebookアプリ内蔵
     ブラウザ)での検証はこのセッションでは実施できていない(Facebook
     アカウント・実機アプリへのアクセスが無いため)——コードレベルの
     実装・通常ブラウザでの表示確認までにとどまる。
  5. **未実装として明確にスコープ外にした項目**: (a) Facebook
     Instant Games SDK/カスタムタブ/Messenger拡張等によるFacebook
     プラットフォーム内での実行(通常のWebページリンクで足りると
     判断し、SDK組み込みの複雑さ・審査要件を追加する理由が無いため
     見送り)、(b) Messengerチャットボットによる自動リンク配布
     (Facebook側のBot API・ページ管理者権限・アクセストークン発行が
     必要で、この環境では実施できない——ユーザー自身がFacebookページ/
     Messenger側で今回作った`facebook.html`のURLを手動投稿する運用を
     想定)。
  - 次にすべきこと: (1) 実際のFacebookページを用意し、そこから
    `facebook.html`のURLを投稿・共有した上で、実際のFacebookアプリ
    内蔵ブラウザから開けることを確認する(ユーザー側のFacebook
    アカウントが必要)、(2) 対象国で実際に使われているゼロレーティング
    プランの詳細(Free Basics以外の類似プログラムの有無)を調査し、
    もし技術的に参加可能な枠組みが見つかれば別途スコープを切って
    再検討する、(3) `version.json`の`buildId`更新運用を継続。

- **2026-08-18(続き2) RSyncインストール促進+自動インストール+成功後の
  自動バックアップを実装(ユーザー指示「同期バックアップを機能させる
  には、RSyncをインストールしましょう！を英語と日本語で表示して簡単に
  インストールして簡単に自動で移行する機能を搭載して」への対応)**:
  1. **`db.rs`に`RsyncError`列挙型を新設**(`NotInstalled`/`Other`)。
     `backup_via_rsync`が`std::io::ErrorKind::NotFound`を判別し
     `NotInstalled`を返すことで、`main.rs`側が「rsyncが無い」ケースを
     明確に分岐できるようにした。
  2. **`POST /v1/db/rsync-backup`**: rsync未検出時、`ok:false`+
     `rsync_missing:true`と共に`message_en`("Let's install RSync! ...")・
     `message_ja`("RSyncをインストールしましょう！...")を返すよう変更。
  3. **`POST /v1/db/install-rsync`新設**(`Db::install_rsync`):
     OS別パッケージマネージャを順に試す(Windows: winget→choco、
     Linux: apt-get→dnf→pacman、macOS: brew、Android/Termux: pkg)。
     `retry_destination`を渡せば、インストール成功直後にそのまま
     `backup_via_rsync`まで実行する(ユーザー指示「簡単に自動で移行
     する機能」への対応、インストール+バックアップをボタン1回で
     完結させる設計)。**正直な開示**: このアプリ自身はrsyncの
     インストーラーを同梱・ダウンロードしない——各OS標準/準標準の
     パッケージマネージャを子プロセスとして呼ぶのみ。該当する
     パッケージマネージャが1つも無い環境では、その旨と手動インストール
     手順(日英併記)を返す。
  4. **実機検証**: `cargo build --release`成功、`cargo test --release`
     (db::以下3件)全green。実サーバー起動+`Invoke-RestMethod`で
     (a) rsync未検出時に`rsync-backup`が日英併記メッセージを正しく
     返すこと、(b) この開発機にはwinget/chocoともに存在しないため
     `install-rsync`が「両方とも見つからなかった」旨を正直に返すこと、
     の両方を確認した。**正直な開示**: 「実際にパッケージマネージャが
     存在する環境でインストールが成功し、続けてバックアップが実際に
     走る」という一気通貫のE2E検証は、この開発機の制約上まだ実施
     できていない——次回、Windows(winget/choco導入済み環境)または
     Linux/macOS/VPS/Android実機で確認する必要がある。
  - 次にすべきこと: (1) 上記4番のE2E検証(実際にパッケージマネージャが
    存在する環境で)、(2) `app.js`フロントエンド側のUI実装(「💾
    データベースバックアップ」モーダル、保存先選択・rsync宛先入力・
    「RSyncをインストール」ボタン)——現状はバックエンドAPIの実装・
    実HTTP検証のみでUIからは未接続、(3) 円グラフ使用率表示・複数端末
    同期は引き続き次の増分。

- **2026-08-18(続き) 保存先選択・rsync同期・旧データ取り込みを実装、実HTTP
  で3エンドポインドとも検証済み(ユーザー指示「DATAやDATABASE保存先は、
  既存の保存先でもそれ以外でも選択可能にして、同期先もRSyncで選択可能に
  して」+「未着手は着手して」への対応)**:
  1. **`POST /v1/db/storage-path`**(`db.rs`の`Db::relocate`): 保存先パスを
     実行時に変更。現在のSQLite接続をPRAGMA wal_checkpointでフラッシュ→
     ファイルを新しい場所へコピー→新しい場所で開き直す。元ファイルは
     安全のため自動削除しない(誤ってデータを失わせないための意図的な
     保守的挙動、ログに手動削除を促す旨を出力)。`Db.path`を
     `Mutex<PathBuf>`化し、複数リクエストからの同時アクセスでも一貫した
     パスを返せるようにした。
  2. **`POST /v1/db/rsync-backup`**(`Db::backup_via_rsync`): `rsync`を
     子プロセスとして起動し、DBファイルをローカルパス/`user@host:/path`
     いずれの宛先へもバックアップできる。**正直な開示**: `rsync`本体は
     同梱しない(利用者環境に既存のものを呼ぶのみ)。このWindows開発機に
     は`rsync`が無いため、実際にコマンドが「見つからない」エラーを
     正しく返すことまでを実HTTPで確認した(黙って失敗にしない設計の
     検証——Linux/macOS/VPS/Android実機での「実際にファイルがコピー
     される」E2E検証は次回、該当環境で実施する必要がある)。
  3. **`POST /v1/db/migrate-legacy`**(`Db::import_legacy`): 旧形式データ
     (メッセージ配列+設定連想配列)を取り込む。**正直な開示・調査結果**:
     `app.js`を実際に確認したところ、これまで会話履歴を`localStorage`
     等へ永続化していた実装は存在しなかった(バージョン管理用の一部
     キーのみ)——つまり移行元となる具体的な「既存の古いデータ」は
     open-english自体には実在しない。そのため本エンドポイントは
     特定の旧形式への対応ではなく、**どんな旧形式のエクスポートが
     今後持ち込まれても受け入れられる汎用的な取り込み口**として実装
     した。実HTTPでメッセージ2件+設定1件の取り込み→一覧取得での反映を
     確認済み。
  4. **aruaru-llm側の調査結果**: `aruaru-llm`のソース(`tenants.rs`・
     `security.rs`等)を確認したが、こちらにも会話履歴・設定をファイル/
     DBへ永続化している既存コードは無かった(Google Search APIキーは
     既存方針通りメモリ上保持のみ)。**つまりaruaru-llm側にも移行元と
     なる具体的な「古いデータ」は実在しない**——「aruaru-llmのバージョン
     アップとして…移行機能」という指示は、この調査結果を踏まえ
     aruaru-llm側`CLAUDE.md`の同日HANDOFFへ記録済み(詳細は同ファイル
     参照、バージョン番号は次のリリースでopen-englishと足並みを揃える
     形で更新する)。
  5. **検証**: `cargo build --release`成功、`cargo test --release`
     (db::以下3件)全green。実サーバー起動+`Invoke-RestMethod`で
     migrate-legacy(取り込み件数の反映確認)・rsync-backup(未インストール
     時の正直なエラー確認)・storage-path(移動後もデータが引き続き
     読めることを確認)の3エンドポイントすべてを実HTTP経由で検証。
  - 次にすべきこと: (1) Linux/macOS/VPS/Android実機での`rsync-backup`の
    「実際にファイルがコピーされる」E2E検証(rsyncが実在する環境が
    必要)、(2) `app.js`フロントエンド側からこれら3エンドポイントを
    呼ぶUI(保存先選択ドロップダウン、rsync宛先入力欄、円グラフ使用率
    表示)の実装——現状はバックエンドAPIの実装・実HTTP検証のみ、
    (3) PC/タブレット/スマホ間の重複しない同期(現状のrsync-backupは
    単方向コピーのみ、双方向マージは別実装が必要)。

- **2026-08-18 会話履歴・設定のローカルDB化(第一段階)を実装・実機HTTP検証
  完了(ユーザー指示「SQLiteではなく、aruaru-db+PostgreSQLのDUAL DBの方が
  片側にトラブルがあっても片側から自動修復する機能で安全性が高い事を
  日本語と英語でPRして」への対応)**:
  1. **`server/src/db.rs`新設**: SQLite(`rusqlite`、`bundled`)を常時
     利用可能なローカル基盤とし、`OPEN_ENGLISH_DATABASE_URL`環境変数が
     設定されていれば`aruaru-db`/PostgreSQLへも書き込みをベストエフォート
     でミラーする(`aruaru-llm`の`geo_content.rs`と同じ`tokio-postgres`
     直結パターン、未設定・接続失敗時はSQLiteのみで継続——可用性優先)。
     `aruaru-db`自身の`DUAL_DATABASE_URL`自己修復ミラーリングとの組合せに
     よる安全性の説明を日英でREADME.md/README-English.mdへ追記済み。
  2. **`main.rs`に`/v1/db/history`(GET/POST)・`/v1/db/history/clear`
     (POST)・`/v1/db/settings`(GET/POST)・`/v1/db/info`(GET)を新設**。
  3. **実機検証**: `cargo test --release`でdb.rsの単体テスト3件全green。
     さらに実際にサーバーを起動し`Invoke-RestMethod`で5エンドポイント
     すべてを実HTTP経由で確認(メッセージ追加→一覧取得→設定保存→
     設定一覧取得→DB情報取得、いずれも期待通りのJSONを返すことを確認)。
  4. **【訂正・2026-08-18追記】RS-JSONを`/v1/db/*`のHTTPボディへ実際に
     適用した**: 直後にユーザーから「RS-JSONは埋め込み/静的JSONファイル
     向け?HTTPボディ処理は対象外とはどういう事?」と指摘を受け、過去の
     方針(2026-08-10続き3・続き4、「RS-JSONはHTTPボディ処理には適用
     しない」)を再検証した結果、**過度に狭い解釈だった**と判断し撤回。
     `RS-JSON`(`rust-json`クレート)の`full`モジュールには
     `from_slice_strict<T: DeserializeOwned>`/`to_vec_strict<T:
     Serialize>`というRPC/wire format向けの型付き入出力APIが最初から
     用意されており(クレート自身のdoc「storage records, RPC/replication
     wire formats, backup manifests, ...」に明記)、HTTPボディを除外する
     技術的理由は無かった。`server/src/main.rs`に`read_rs_json_body`/
     `rs_json_response`ヘルパーを新設し、RPoemの`Json<T>`(内部は素の
     `serde_json`)経由だったリクエスト/レスポンス双方を`rust_json::
     from_slice_strict`/`to_vec_strict`経由へ置き換えた。**正直な
     開示**: RFC 8259厳密モード(`serde_json::from_str`委譲)を通すため、
     受理するJSONの範囲自体はRPoemの`Json<T>`と同じ(JSON5風の寛容拡張は
     このエンドポイントでは意図的に無効——リクエスト元は自分自身の
     フロントエンドJSであり寛容さは不要なため)。実HTTPで(1)正常系の
     往復、(2)JSON5風(クォート無しキー+トレイリングカンマ)入力が
     `BadRequest`で正しく拒否されること、の両方を確認済み。
     `aruaru-db`/`open-raid-z`等でも同様のパターンが適用可能かは
     このコミットのスコープ外(次回検討)。
  5. **未着手のまま次回へ持ち越し**: (a)
     「aruaru-llmのバージョンアップとして、既存の古い物からDATABASE
     システムに移動も簡単にする機能を搭載して」——aruaru-llmと
     open-englishは別リポジトリのため、具体的にaruaru-llm側の何を
     バージョンアップし、何を「既存の古い物」(open-englishの
     localStorage起点のデータ)として移行するのか、次回開始時に
     ユーザーへ具体的なスコープを確認してから着手する。(b) 円グラフ
     使用率表示(「何％使用／何％中」表示、日英)・保存先選択(内部
     ストレージ/microSD)・DBファイル移動・外部rsyncバックアップ
     (Googleドライブ/USB/VPS)・PC/タブレット/スマホ間の重複しない
     同期・`open-easy-web`/`open-web-server`のインストール導線——
     いずれも今回のDB基盤を土台に次の増分で着手する。(c) `app.js`
     フロントエンド側を`/v1/db/*`APIへ実際に配線する作業(現状は
     バックエンドAPIの実装・実HTTP検証のみで、UIからの呼び出しは
     まだ無い)。
  - 次にすべきこと: 上記5番(a)〜(c)を、優先度・依存関係を踏まえて
    次回セッション冒頭でユーザーに確認の上、順に着手する。

- **2026-08-12 シャットダウン前チェックポイント(v0.6.0リリースCI対応、
  中断)**: `v0.6.0`タグのリリースCIがWindows向けビルドで
  `dxc(DirectX Shader Compiler)未検出`エラーで失敗していた件、
  根本原因は`RS-SmartTCP`が依存する`zfs_accel_hlsl`の既定feature
  (`gpu`)がWindows CIランナーに無い`dxc`を要求すること。`open-raid-z`
  自身のCIが同じ理由でWindows向けも`--no-default-features`を選んでいる
  既存方針に揃え、`RS-SmartTCP/Cargo.toml`を「OS問わず常に
  `default-features = false`」へ修正し、`cargo test --lib`で82件全pass
  確認の上コミット・push済み(`16caba7`)。**このリポジトリ
  (`open-english`)自体には未コミットの変更は無い**(`git status`で
  確認済み、作業ツリーはクリーン)。
  - 次にすべきこと(次回セッション再開時): (1) `RS-SmartTCP`修正が
    反映された状態で`open-english`の`v0.6.0`リリースCI
    (`gh run list --workflow=release.yml`で確認、直近の失敗run ID
    `31536706862`)を再実行し、Windows向けビルドが今回こそ成功するか
    確認(既存タグ`v0.6.0`は変更不要——CIはRS-SmartTCPを`main`branchから
    都度cloneするため再pushは不要、`gh workflow run`または
    `gh run rerun`での再トリガーのみで良い)。(2) 全3プラットフォーム
    成功後、GitHub Releaseに3種のインストーラー/tarballが実際に添付
    されているか`gh release view v0.6.0`で確認。(3) Android実機での
    アップデート検知確認・Windows`self_update.rs`のE2E確認(いずれも
    まだ実施していない、詳細は本ファイルの過去のHANDOFF参照)。

- **2026-08-12 v0.6.0チェックポイント: README/README-English.mdに
  最新バナー追記、`version.json`/Android`versionName`をバンプ
  (ユーザー指示「完成したらリリースして。READMEとCLAUDEとPORTINGを
  日本語と英語で編集してコミットして」への対応、うちドキュメント編集・
  コミットまでを実施——実際のタグpush・GitHub Release作成は
  ユーザー指示「コミットしたら停止して」により次回へ持ち越し)**:
  1. **`version.json`**: `0.5.3`→`0.6.0`(buildId:
     `2026-08-12-android-standalone-exam-prep-unix-installer`)。
  2. **`android/app/build.gradle.kts`**: `versionName`を`0.6.0`へ、
     `versionCode`を`1`→`2`へ同期(既存の運用ルール「versionCode/
     versionNameは`version.json`の`version`と手動で同期させること」
     に従った)。
  3. **README.md/README-English.md**: 直近(2026-08-11〜12)の主要機能
     (Android単体動作化+aruaru-llm実同梱、資格試験対策コーナー、
     学びたい言語選択、Linux/macOS版インストーラー)を要約した新しい
     バナーを日英両方の先頭に追加。
  - 次にすべきこと: (1) 実際に`git tag v0.6.0`→pushし、
    `.github/workflows/release.yml`(Windows/Linux/macOS
    インストーラー3種)を実行させ、GitHub Releaseとして公開する、
    (2) リリース後、Android実機で`checkForAppUpdate()`が新バージョンを
    正しく検出することの確認、(3) Windowsの`self_update.rs`が実際に
    この新リリースを検出→アンインストール→インストールする一気通貫の
    実機検証(現時点でまだ一度も実施していない、`v0.5.2`リリース時から
    持ち越しの既知の未検証事項)。

- **2026-08-11(続き10) Linux/macOS版インストーラー(`install.sh`)+CI
  ジョブを新設、資格試験対策の問題数を増量(ユーザー指示「Windows/
  Linux/macOS版インストーラーの横展開、モデル重み自動配置、各試験の
  問題数拡充」への対応、うち1番目・3番目を実施)**:
  1. **`installer/unix/install.sh`+`fetch-aruaru-llm.sh`(新設)**:
     Windows版(`installer/windows/`、Inno Setup)の「まとめてインス
     トール」設計をLinux/macOS共通のroot権限不要インストールスクリプト
     として移植。`--with-aruaru-llm`フラグでaruaru-llm本体(実行
     ファイルのみ)も取得可能(Windows版の`fetch-aruaru-llm.ps1`と
     同じ設計、GitHub Releases APIから対応OSのアセットを検索)。
     Linuxのみ`.desktop`ランチャーも作成(macOSにはこの仕組みが無い
     ため対象外)。**実機検証**: 実際に`server/target/release/`の
     バイナリ+静的アセットを使い、`sh install.sh`を実行→インストール
     先に正しく配置→ランチャー経由で起動→`curl`で`http://
     127.0.0.1:4601/`(200・index.html本文)・`/version.json`(200)への
     到達を確認した(型チェックのみで完了と報告しない方針の徹底)。
  2. **`.github/workflows/release.yml`拡張**: `build-unix-installer`
     ジョブ(`ubuntu-latest`/`macos-latest`のマトリクス、既存の
     `build-windows-installer`と同じsibling path依存checkoutパターン)
     を新設し、Linux(`x86_64`)・macOS(`aarch64`)向けtarball
     (バイナリ+静的アセット+`install.sh`+`fetch-aruaru-llm.sh`)を
     ビルド・GitHub Releaseへ添付するようにした。**正直な開示**:
     `aruaru-llm`自体はまだmacOS向けのCIビルドを持たない
     (`aruaru-llm/.github/workflows/release.yml`確認済み、Linux/
     Windowsのみ)——`fetch-aruaru-llm.sh`のmacOS(Darwin)分岐は
     現時点では該当アセットが見つからず「見つからない」旨を正直に
     案内するのみに留まる。
  3. **資格試験対策の問題数増量**: 英検3級・TOEIC・JLPT N5をそれぞれ
     5問→10問(既存5問に5問追加)。他の区分(英検1級/2級/準2級/4級/
     5級・TOEFL・JLPT N1-N4・日本語検定1-3級)は今回未着手のまま
     (時間の都合で優先度の高い3区分のみ、次回以降拡充)。
  4. **検証**: `node --check app.js`構文確認、Android実機へAPK再
     インストール・再起動し、両プロセス(`libopenenglishserver.so`/
     `libaruarullm.so`)が引き続き生存することを確認(regression無し)。
  - 次にすべきこと: (1) 残りの資格試験区分の問題数増量、(2) モデル
    重み(GPT-2系・multilingual-e5-small)のAndroid自動配置——現状は
    ユーザー自身が内部ストレージへ手動配置する前提のまま、
    (3) `aruaru-llm`側にmacOS向けCIビルドを追加するかの検討、
    (4) 実際にタグをpushしてLinux/macOSインストーラーが正しく
    GitHub Releaseへ添付されることの実CI検証(今回はローカルでの
    `install.sh`単体動作確認のみ、CI経由の実ビルドは未実施)。

- **2026-08-11(続き9) aruaru-llm(AI応答エンジン)をAndroidへ実際に同梱・
  実機で動作確認+JLPT N1〜N5・日本語検定・学びたい言語選択・aruaru-db
  セットアップ導線を追加(ユーザー指示「最優先で…アンドロイド…に開発」
  +「英会話か日本語会話か学びたい言語を選べるように」+「日本語検定と
  日本語能力検定の擬似的な模擬試験機能も搭載してテスト後に日本語教室に
  移って」+「open-easy-webとPostgreSQLとaruaru-dbをSETUP…なるべく
  簡単に」への対応)**:
  1. **aruaru-llm本体をAndroidへ実際に同梱**: `cargo ndk`で
     `aarch64-linux-android`向けにクロスコンパイル(初回はモデル埋め込み
     ディレクトリがコンパイル時固定パス〈`CARGO_MANIFEST_DIR`〉だった
     ため、`ARUARU_LLM_EMBED_MODEL_DIR`環境変数での上書きに対応
     〈`aruaru-llm`側で修正・別コミット〉、`ARUARU_LLM_BIND`も追加し
     `127.0.0.1`限定で起動可能にした)。`libaruarullm.so`として
     `jniLibs/arm64-v8a/`へ同梱し、`MainActivity.kt`が
     `libopenenglishserver.so`と並行して`ProcessBuilder`で起動する
     よう拡張。**実機検証**: `ps -A`で両バイナリが子プロセスとして
     生存し続けることを確認、WebView側の既存ヘルスチェックが
     「aruaru-llm installed and connected!」を実際に検出、
     `curl`で`/healthz`(`ok`)・`/v1/chat`(実際の意図分類応答)への
     到達も確認した。**正直な開示**: モデル重み(GPT-2系・
     multilingual-e5-small)はAPKに同梱していない(サイズ・著作権の
     都合、既存のPC版と同じ)——現時点では「サーバーが端末内で起動
     すること」の実証までで、実用的な応答品質にはモデル重みの別途
     配置が必要。
  2. **「学びたい言語」選択**(`app.js`の`trainerRoleByTarget`):
     英会話/日本語会話のいずれを学ぶかをプロンプトの役割部分で
     切り替える新設セレクター。既存の`reply-lang`(応答言語の混在
     方針)とは独立した軸。
  3. **JLPT N1〜N5+日本語検定1〜3級**(オリジナル模擬問題、実際の
     過去問は不使用): JLPT(外国語としての日本語能力)と日本語検定
     (母語話者も対象、敬語・語彙・慣用表現重視)の出題傾向の違いを
     踏まえて作問。採点後、「トレーナーと練習する」ボタンで
     `learn-target`を自動的に日本語へ、`reply-lang`をhybrid(英日
     併記の表示・読み上げ)へ切り替えて「日本語教室」へ引き継ぐ設計
     (`practiceExamPrepWithTrainer`の`isJlpt`判定を拡張)。
  4. **「Setup aruaru-db」ボタン**: `aruaru-db`が既に持つ
     `install.sh`/`install.ps1`をそのまま案内する簡易モーダル
     (新規スクリプトは書いていない)。**正直な開示**: これは共有
     インフラであり、マシン/サーバーごとに一度だけセットアップすれば
     十分(`open-easy-web`の分身の術構成で他アプリ/ドメインが再利用
     できる)——open-englishアプリ自体は無くても動作する旨を明記。
  5. **実機検証**: 上記1〜4すべてをAPK再ビルド・実機再インストールの
     上でスクリーンキャプチャで確認(「Setup aruaru-db」モーダル表示・
     試験選択肢に日本語検定/JLPTが追加されていること・日本語検定の
     開示文中の`nihongokentei.jp`へのリンクが機能すること等)。
  - 次にすべきこと: (1) 実際のモデル重み(GPT-2系・埋め込みモデル)を
    Android内部ストレージへ配置する導線(現状はユーザー自身が別途
    配置する前提のまま、自動ダウンロード機能は未実装)、(2) Windows/
    Linux/macOS版インストーラーの「まとめてインストール」対応の
    横展開(Windows版は既にaruaru-llm同梱オプションあり、Linux/macOS
    版は今回未着手)、(3) JLPT/日本語検定の各レベルの問題数を増やす
    (現状5問のみ)。

- **2026-08-11(続き8) 資格試験対策コーナー(英検1〜5級・TOEIC・TOEFL)+
  採点後のトレーナー連携+実機で発見したモーダルのスクロール不能バグを
  修正(ユーザー指示「英検1級2級3級4級5級とTOEICとTOEFLの様々な
  レベル別擬似的模擬試験機能とその資格対策コーナーを搭載して」+
  「英検で採点後の英会話学習をつなぐようにして」への対応)**:
  1. **`EXAM_PREP_QUESTIONS`(`app.js`)**: 英検1級/2級/準2級/3級/4級/
     5級・TOEIC・TOEFLの8区分、各5問(計40問)のオリジナル練習問題を
     新規作成。**正直な開示(最重要)**: 実際の試験団体(日本英語検定
     協会等)の過去問は一切使用・転載していない——本アプリ用に書き
     下ろしたオリジナル問題であり、難易度も大まかな目安に過ぎず
     公式のスコア予測ではないことをモーダル内に日英併記で明記した。
  2. **採点後のトレーナー連携**(ユーザー指示「採点後の英会話学習を
     つなぐようにして」への対応): 採点結果から間違えた問題(満点なら
     全問)を「Practice these with your trainer / トレーナーと練習する」
     ボタンで通常のチャット(`askTrainer`)へ引き継ぐ設計
     (`practiceExamPrepWithTrainer`)。モーダルを閉じ、間違えた問題と
     正解の一覧を含む練習リクエスト文を自動生成してチャット送信欄
     経由で送信する。
  3. **実機検証で発見・修正した重大なUXバグ**: Android実機
     (adb接続確認済み)で実際に問題を解いたところ、**モーダルが画面
     内に収まらず下へスクロールできず、後半の問題に回答できない**
     という実バグが発覚(ユーザー報告)。原因調査の結果、
     `google-search-modal`・新設`exam-prep-modal`が使う`.setup-panel`
     クラスに**CSSルールが一切定義されておらず**(既存の`.setup-card`
     〈`setup-modal`用〉にはあった`max-height: 85vh; overflow-y: auto;`
     が無かった)、内容が長くなると画面外にはみ出してスクロール不能に
     なっていた——`.setup-panel`へ同等のスタイルを追加して修正。
  4. **実機再検証**: 修正後、実際にAndroid実機で英検3級の問題5問
     全てを最後まで解答→スクロールでSubmitボタンへ到達→採点
     (1/5と正しく表示)→「トレーナーと練習する」ボタンでモーダルが
     閉じ、間違えた問題の要約(正解付き)がチャットログへ正しく
     送信されることを画面キャプチャで確認した(`aruaru-llm`未起動の
     ため応答自体は`Failed to fetch`エラーになるが、これは想定通りの
     既存の正直なエラー表示であり、メッセージ送信自体は成功している)。
  5. **Android版への反映**: `android/app/src/main/assets/webroot/`へ
     `index.html`/`app.js`/`style.css`をコピーし直し、APK再ビルド・
     実機再インストールの上で上記4番の検証を実施。
  - 次にすべきこと: (1) `aruaru-llm`自体のAndroidクロスコンパイル・
    同梱検討(実現すればチャット応答も含めた単体動作が完成する、
    次項目参照)、(2) 資格試験対策コーナーの問題数を増やす(現状は
    各区分5問のみ、実用的な模擬試験としては少ない)、(3)
    `google-search-modal`と同じ`.setup-panel`バグが今回の修正で
    同時に解消されたため、Google Search Setupモーダル側も併せて
    改善されているはず(次回、こちらも長いコンテンツで確認すると
    なお良い)。

- **2026-08-11(続き7) Android版を「PC/LinuxのWEBサーバー不要の単体動作
  アプリ」へ全面刷新+実機検証で発見した重大バグを修正(ユーザー指示
  「アンドロイドスマホとタブレットにインストーラー付きアプリでPCや
  LINUXのWEBサーバー不要でオンラインでUPDATEする機能を搭載して」+
  「単体で動作する設計で」+実機接続の上でのTEST依頼)**:
  1. **アーキテクチャ変更**: 従来はPC上で起動済みの`server/`へ同一
     Wi-Fi経由で接続する薄いクライアントだった。`cargo ndk`で
     `open-english-server`をaarch64-linux-android向けにクロス
     コンパイルし、`jniLibs/arm64-v8a/libopenenglishserver.so`として
     APKへ同梱(`open-web-server`Android版と同じTermux方式、
     `useLegacyPackaging=true`)。静的アセット(`index.html`等)は
     `assets/webroot/`に同梱し、初回起動時にアプリの内部ストレージへ
     展開してから`OPEN_ENGLISH_SERVER_ROOT`環境変数で渡す
     (`server/src/main.rs`に`OPEN_ENGLISH_SERVER_ROOT`環境変数による
     配信元ディレクトリ上書きを新設、コンパイル時固定パスの代替)。
     `MainActivity.kt`が`ProcessBuilder`でこのバイナリを`127.0.0.1`
     限定でローカル起動し、WebViewが`http://127.0.0.1:<port>/`を読み
     込む——**外部ネットワーク・PC・Linuxサーバーは一切不要**。
  2. **クロスコンパイルで発覚・修正した実バグ1(`RS-SmartTCP`側)**:
     `open-english/server`のAndroidビルドが`zfs_accel_hlsl`
     (Windows専用D3D12、`rs-smarttcp`が同日追加した依存)で225件の
     コンパイルエラーを起こして失敗。`RS-SmartTCP`側でターゲット別に
     依存を切り替えて修正(詳細は`RS-SmartTCP/CLAUDE.md`同日HANDOFF
     参照)。
  3. **実機検証で発覚・修正した実バグ2(このリポジトリ側、より重大)**:
     修正1の後、実際にAndroid実機(`adb`接続確認済み)へインストールし
     起動したところ、アプリ画面には正しくWebUIが表示されたにも
     関わらず、内蔵サーバーの子プロセスが実際には存在しないことを
     `ps -A`で発見。`/data/local/tmp/`へバイナリを手動配置して直接
     実行したところ、`self_update.rs`の自動更新機構(元々Windows専用の
     設計)が**プラットフォーム判定を一切行っておらず**、Android上でも
     GitHub Releaseの新バージョンを検出するたびにWindows用インストーラー
     の起動を試み(実機ログで`cmd: Can't find service: /C`エラーを実際に
     確認)、直後に`std::process::exit`でサーバー自身を強制終了させて
     いた。`check_and_apply_update()`冒頭に`if !cfg!(target_os =
     "windows") { return; }`を追加して修正。
  4. **実機再検証**: 修正後のバイナリを実機へ配置して単体実行し、
     `open-english self-update: skipped (this update mechanism is
     Windows-only)`のログの後もサーバーがタイムアウトまで生存し
     続けることを確認。続けてAPKを再ビルド・再インストールし、
     `ps -A`で`libopenenglishserver.so`(親PIDがアプリ本体)が実際に
     子プロセスとして存在し続けることを確認した(型チェック・ビルド
     成功だけで完了と報告しない方針の徹底、実機で2段階の不具合を
     発見・修正)。
  5. **正直な開示・今回のスコープ外**: (a) アプリ自体の自動更新
     (Windows版のようなサイレント差し替え)はAndroidの仕組み上
     実現できない(Play Store配布ではないため)——引き続きKotlin側の
     `checkForAppUpdate`がGitHub Releasesページへのリンクを表示する
     のみ。(b) 英検/TOEIC/TOEFLレベル別模擬試験機能は別リクエストとして
     受領済みだが、今回のセッションでは未着手(次回対応)。(c) x86_64
     エミュレータ向けjniLibsは今回同梱していない(実機〈arm64-v8a〉
     での検証を優先)。(d) `aruaru-llm`(AI応答エンジン)は依然として
     別プロセス・別インストールが前提のまま——今回同梱したのは静的
     フロントエンド配信サーバーのみで、チャット機能自体は`aruaru-llm`
     が別途起動している必要がある(このAndroid版単体では静的UIの
     表示までが実証範囲)。
  - 次にすべきこと: (1) 英検1級〜5級・TOEIC・TOEFLの模擬試験機能
    (オリジナル問題、著作権保護対象の実際の試験問題は使わない)の実装、
    (2) `aruaru-llm`自体もAndroid向けにクロスコンパイル・同梱できるかの
    検討(実現すればチャット機能も完全に単体動作可能になる、現状は
    静的UIの単体配信のみ)、(3) x86_64エミュレータ向けjniLibs追加、
    (4) APK署名・正式リリースビルド。

- **2026-08-11(続き6) リポジトリを公開(PUBLIC)化+過去の履歴書き換え+
  小さい文字サイズの底上げ(ユーザー指示「自動アップデート機能が
  動くように、個人情報は削除した上で公開して」+「一番小さな文字の
  フォントは二周り大きく、中間のは一回りフォントサイズを大きくして」)**:
  1. **公開化に伴う実機監査**: `git log --all -p`で全履歴を監査した
     結果、(a) 全コミットの作者情報(氏名+個人Gmail)——ただしこれは
     `open-english`固有ではなくエコシステム全体の標準git設定であり、
     既に`aruaru-llm`等の公開リポジトリで同じ情報が公開済みのため、
     このリポジトリだけ書き換える実質的なプライバシー効果は無いと
     判断し現状維持、(b) 過去のコミットに`server/target/`(ビルド
     成果物)が誤って含まれており、ローカルWindows環境のユーザー名
     パス(`C:\Users\<username>\...`)が履歴に残っていた——こちらは
     `open-english`固有の問題であり、他リポジトリへの影響なく除去
     可能と判断。ユーザーに両者の違いを説明し、(b)のみ除去する方針の
     承認を得た。
  2. **`git-filter-repo`で履歴を書き換え**(`pip install git-filter-repo`)。
     書き換え前にリポジトリ全体をバックアップ(`open-english.backup-
     before-history-rewrite`、検証後に削除済み)。`--path server/target
     --invert-paths`で該当パスを全23コミットの履歴から除去し、
     `git log --all -p | grep "C:\\Users\\<username>"`が0件になった
     ことを確認した上で`git push --force`。`v0.5.2`タグもコミット
     ハッシュが変わるため削除→再作成→再push。
  3. **`gh repo edit --visibility public`で公開化**、`gh repo view`で
     `PUBLIC`になったことを確認。**実際に匿名で`GET .../releases/
     latest`を叩き、200 OKで`v0.5.2`のリリース情報が取得できることを
     実機で確認**(公開化前は404だった状態からの変化を実証、型
     チェックだけで完了と報告しない方針の徹底)。
  4. **フォントサイズの底上げ**(`style.css`): 最小サイズ層
     (0.7rem/0.75rem)を+0.2remの二段階、中間サイズ層(0.8rem〜
     0.95rem)を+0.1remの一段階、それぞれ底上げ(Pythonの正規表現
     一括置換、元の値を基準に単一パスで変換することで二重適用を
     回避)。見出し(1.1rem/1.2rem)は対象外。実際にブラウザで
     `getComputedStyle`により`.settings`が14.4px(0.9rem)へ変わって
     いることを確認済み。
  - 次にすべきこと: 特になし(今回のスコープは完了)。今後
    `server/target/`等のビルド成果物が誤ってコミットされないよう、
    `.gitignore`の内容を都度確認する習慣を継続すること。

- **2026-08-11(続き5) 起動時の自動アップデート機能を新設(GitHubから
  最新版を検出→自動アンインストール→自動インストール、ユーザー指示
  「起動時の自動メンテナンスで自動UPDATEの自動バージョンアップ機能も
  搭載して、自動でGithubから最新版を見つけて古いのは自動でアンインス
  トールして最新版を自動インストールする機能も搭載して」への対応)**:
  1. **なぜサーバー側(Rust)に実装したか**: ブラウザ側のJS(`auto-
     update.js`)はページの再読み込みしかできず、Windowsアプリの
     アンインストール・インストールという特権操作は行えない——この
     機能は必ずネイティブコード側(`open-english-server`)に実装する
     必要がある。
  2. **新規`server/src/self_update.rs`**: `check_and_apply_update()`を
     サーバー起動時に非同期タスクとして起動(サーバー自体の起動を
     ブロックしない)。GitHub Releases API
     (`https://api.github.com/repos/aon-co-jp/open-english/releases/
     latest`)で最新タグを取得し、実行ファイルと同じディレクトリの
     `version.json`のバージョンと比較(セマンティックバージョン単純
     比較)。新しい版があれば、ファイル名に"setup"を含む`.exe`アセットを
     ダウンロードし、一時バッチスクリプトを生成してデタッチ起動した上で
     **このプロセス自身を`std::process::exit`で終了する**(実行中の
     自分自身のexeファイルはWindows上ではロックされたまま削除できない
     ため、先にプロセスを終了させてファイルロックを解放する設計)。
     バッチスクリプトは少し待った後、既存の`unins000.exe`(存在すれば)
     をサイレント実行→新しいインストーラーをサイレント実行、という
     順で進む。新インストーラー側の`[Run]`セクション(既存の
     `open-english.iss`)がインストール後に自動でアプリを再起動する
     ため、ユーザー操作なしで新バージョンが立ち上がる。
  3. **`.github/workflows/release.yml`を新設**: `v*.*.*`タグpushで
     `server/`をビルド→Inno Setup(`choco install innosetup`)で
     インストーラーをパッケージ化→GitHub Releasesへ添付する
     (`aruaru-llm`の既存`release.yml`と同じパターン、sibling path依存
     〈RPoem〉のcheckoutも同様に追加)。
  4. **正直な開示(最重要)**: 2026-08-11時点で`aon-co-jp/open-english`に
     GitHub Releaseが1件も存在しない(`GET .../releases/latest`は
     `404`)。本機能は正しく実装・単体テスト済み(バージョン比較ロジック
     4件、`cargo test --release`全green)で、実際にサーバーを起動して
     「リリースが存在しない場合に正直にログを出すだけでクラッシュせず
     継続する」ことも実機で確認済みだが、**「新バージョン検出→実際に
     アンインストール→インストール」という一連の流れそのものは、
     まだ実際のリリースが存在しないため実機での最初から最後までの
     検証は行っていない**——次回、実際にタグをpushして最初のリリースを
     作成した後、2つ目のバージョンをリリースして実際にこの自動更新が
     動作することを確認する必要がある。
  - 次にすべきこと: (1) `v0.5.2`タグをpushして最初のGitHub Releaseを
    作成、(2) 何らかの変更を加えた次バージョンをリリースし、実際に
    起動中の旧バージョンが自動でアンインストール→新バージョンへ
    アップデートされることを実機で確認、(3) Android/Linux版の同等機能
    (今回はWindows Inno Setupインストーラーのみ対応)は未着手。

- **2026-08-11(続き4) 就職・転職・観光の話題検出+エコシステム内サイト
  紹介機能を追加(`aruaru-llm`側新設`POST /v1/referrals/check`)、
  **通常チャットモードでも動作**する設計**:
  1. **`referralsSuffix(userText)`新設**(`app.js`): 発話文を
     `/v1/referrals/check`へ渡し、就職・転職・観光の話題と判定された
     場合のみaruaru.tokyo・audiocafe.tokyo/aruaru・audiocafe.tokyo/
     aruaru-lady・nasa.tokyoへのリンクを日英併記で会話に追記する。
     `askTrainer()`(通常チャット)・`advanceTrainingMode()`(研修モード)
     の両方から呼ぶ——ユーザー指示が「しゃべったり文字入力すると」と
     研修モード限定でなかったため、両モードに配線した。
  2. **実機検証**: 実際に`aruaru-llm`(60件全green)+`open-english-
     server`を起動し、通常チャットモードで"I want to change my job
     soon."と発話→実際に4件のリンクすべてが日英併記で表示される
     ことをブラウザ上で確認済み。
  - 次にすべきこと: 特になし(今回のスコープは完了)。
  - **正直な開示・今回スコープ外として明確に分離**: ユーザーから同時に
    依頼のあった「Android/タブレット向けインストーラー付きアプリを
    PC/LinuxのWebサーバー不要でオンラインアップデート対応にする」は、
    既存のAndroid WebViewアプリ(PCサーバーへの接続が前提)とは根本的に
    異なるアーキテクチャ(LLM推論・DBを端末上で完結させる、または
    クラウド側にバックエンドを新設する、のいずれか)を要する大きな
    構想であり、今回は着手していない——過去に検討済みのWASM移植調査
    (`open-cuda`側`open-cuda-llm-wasm`、worktree-agentブランチ、未マージ)
    や「スマホ単体動作」の議論を踏まえ、次回は専用のスコープを切って
    ホスティング方針(オンデバイス推論 vs クラウドバックエンド)を
    ユーザーと確認した上で着手すべき。

- **2026-08-11(続き3) 富士山の安全案内+観光ツアー検索をトレーニング
  モードへ連携(`aruaru-llm`側新設`/v1/geo/fuji`・`/v1/geo/tours`、
  詳細は`aruaru-llm/CLAUDE.md`同日HANDOFF参照)**:
  1. **`fujiInfoText()`新設**(`app.js`): 国名DB検索の結果、ランドマークが
     富士山("Fuji"/"富士山")だった場合、`GET /v1/geo/fuji`を追加取得し、
     安全上の注意(スキーウェア+ヘルメット着用、山小屋事前予約・一泊を
     強く推奨)・山小屋の例・登山バス/通行予約先・登山用品レンタル店を
     日英併記で会話に追記する。
  2. **`tourSearchText()`新設**(`app.js`): 話題に出た国・地域について
     `POST /v1/geo/tours`を呼び、観光ツアー紹介+オンライン予約の
     検索結果(上位3件のタイトル)+YouTube検索結果ページへの直リンクを
     常に日英併記で案内する。Google Search APIキー未設定時は、その旨を
     正直に案内するのみに留める(黒く結果を偽装しない)。
  3. **実機検証**: 実際に`aruaru-llm`+`open-english-server`を起動し、
     研修モードで"I love Japan and Mount Fuji."と発話→
     富士山の安全案内・山小屋例(富士山みはらし)・バス予約先
     (富士急行バス)・登山用品店(やまどうぐレンタル屋)が実際に画面へ
     表示されることを確認した。観光ツアー検索は、このセッションでは
     Google Search APIキー未設定のため正直な「未設定」案内が表示される
     ことも確認済み(APIキー設定後の実検索結果表示は次回検証項目)。
  - 次にすべきこと: (1) 実際にGoogle Search APIキーを設定した状態での
    `/v1/geo/tours`のE2E検証、(2) 富士山以外の観光地での同様の安全案内
    パターンの拡張は今回のスコープ外。

- **2026-08-11(続き2) 実際にサーバーを2つとも起動しブラウザで実機
  テストを実施(ユーザー指示「実際のopen-englishでTESTしたい」)、
  発見した3件の実バグを修正**:
  1. **実施内容**: `aruaru-llm.exe`+`open-english-server.exe`を実際に
     起動し、ブラウザで研修モードを選択→名前入力→「I from Japan.」と
     発話→トラさんへの引き継ぎ、まで一連の流れを実際に操作して検証した
     (型チェック・ビルド成功だけで完了と報告しない既存方針の実践)。
  2. **発見・修正した実バグ1(`aruaru-llm`側)**: 地理DB国名検索が
     完全一致のみで、実際の発話文全体("I from Japan.")を渡すと
     空振りしていた——詳細・修正内容は`aruaru-llm/CLAUDE.md`同日
     HANDOFF参照。修正後、実際にブラウザで再現し
     "I love Mount Fuji and Sushi! / 私は富士山と寿司が大好きです!\n
     A popular souvenir there is Folding fan. / そこの人気のお土産は
     扇子です。"という正しいDB駆動の応答が表示されることを確認した。
  3. **発見・修正した実バグ2(音声合成の誤発音、ユーザー報告「17歳を
     英語の部分でジュウナナイヤーズと発音していた」)**: `pickVoice`が
     要求言語(例: en-US)に合う声が1件も無い場合、無関係な言語
     (日本語)の声へフォールバックしていた(`cachedVoices[0]`)。多くの
     ブラウザは`utterance.voice`設定時にその声自身の言語の発音規則
     (数字の読み方等)を優先するため、英語の"17 years old"が日本語の
     声で読まれ誤発音される実バグだった。要求言語に合う声が無い場合は
     `voice`を設定せず`utterance.lang`側の既定発音に委ねるよう修正。
  4. **YouTube検索結果リンクを追加**(ユーザー提供のURL、秋葉原メイド
     カフェの接客技法紹介の直下): 表示テキストは短い説明ラベルのみで
     長いURL自体は画面に表示しない(ユーザー指示「長いURLは見せずに、
     LINKだけ貼って」、`<a href="...">説明ラベル</a>`のhref属性内に
     留める)。
  5. **検証**: `aruaru-llm`側`cargo test --release`56件全green。実際に
     2回サーバーを再起動して修正を確認済み(上記2番・3番はいずれも
     実機での再現・修正確認まで実施)。
  - 次にすべきこと: 特になし(今回発見した3件はいずれも解消済み)。

- **2026-08-11(続き) 地理DB連携・年齢層/レベル/ビジネス英会話選択・
  メンテナンスバナー・研修モードの英日両方読み上げ・トラさん自動引き継ぎ・
  インストーラーへのaruaru-llm同梱オプションを追加**:
  1. **地理・観光DB連携**(`aruaru-llm`側新設`/v1/geo/lookup`・
     `/v1/geo/random`、詳細は`aruaru-llm/CLAUDE.md`同日HANDOFF参照):
     `findCountryFunFact`をDB検索(国名一致)へ変更、DB未接続時は既存の
     固定マップへフォールバック。「今度オーストラリアに旅行の予定が…」
     のような発話向けの`replyToTravelPlanMention`も追加。
  2. **年齢層・レベル・ビジネス英会話の選択UI**(ユーザー指示「保育園児、
     幼稚園児、小学生、中学生、高校生、大学などのどれか一つ選択と、
     学生向けレベル分け。超初心者、初心者、中級者、ネイティブなどの
     レベル分け選択ともう一つ複数選択でビジネス英会話も追加選択可能」+
     「保育園児以下も選択可能に、社会人と社会人以上も選択可能にして」):
     `#age-group`(乳幼児/保育園児/幼稚園児/小学生/中学生/高校生/大学生/
     社会人/シニアの9択、単一選択)+既存`#level`のラベルを整理
     (超初心者/初心者/中級者/ネイティブ)+`#business-english-toggle`
     (チェックボックス、他の選択と併用可)を追加。プロンプトへの指示文
     (`ageGroupInstructions`/`BUSINESS_ENGLISH_INSTRUCTION`)で調整する
     設計——GPT-2系は指示追従を保証しないため、確実な遵守は主張しない
     (既存の`levelInstructions`と同じ正直な開示方針)。
  3. **メンテナンス中バナー**(ユーザー指示「open-englishを起動中に2分間、
     ただいまメンテナンス中です。2分ほどお待ち下さいと日本語と英語で
     表示して」): ページ読み込み時に固定120秒のカウントダウンバナーを
     表示する簡易実装。**正直な開示**: バックエンド(aruaru-db側の地理
     データseed投入等)の実処理完了通知と連動しているわけではなく、
     固定タイマーでの表示に留まる。
  4. **研修モードの英日両方読み上げ**(ユーザー指示「メイドカフェ研修
     モードで英語で一言ワンフレーズしゃべったら対応する日本語でも
     しゃべってを繰り返して」): 新設`speakBilingual(text)`——通常モードの
     `speak()`は`reply-lang`設定に応じて英語/日本語の一方だけを読むが、
     研修モード専用のこの関数は常に英語→日本語の順で両方読む(音声合成
     キューへ2つのutteranceを積むだけ、`cancel()`を挟まなければブラウザが
     順番に再生する)。
  5. **トラさんへの自動引き継ぎ**(ユーザー指示「メイドが一通りしゃべったら、
     次はトラさんのテーマのBGMが流れて、今度はトラさんがしゃべって」):
     研修モードの最終ステップの発話が終わった後、既存の`switchCharacter()`
     (ジングル再生を含む)を自動的に呼び、トラさんが引き継ぎの一言を話す
     よう`advanceTrainingMode`を拡張した。
  6. **Windowsインストーラーへの「まとめてインストール」オプション**
     (ユーザー指示「他にも必要な関連リポジトリやプロジェクトも
     インストールする時にまとめてインストールしますか？などの機能が
     欲しい」): `[Tasks]`に既定チェック済みの`installaruarullm`を追加、
     選択時は`fetch-aruaru-llm.ps1`(GitHub Releases APIから最新の
     Windows向けzipを取得・展開)を実行する。**正直な開示**: 取得するのは
     aruaru-llm本体(実行ファイル)のみ——GPT-2/DistilGPT-2の実モデル重み・
     `aruaru-db`・PostgreSQL本体は含まない(それぞれ別途セットアップが
     必要)。ダウンロード失敗時もインストーラー全体は止めない(可用性
     優先の既存方針)。`ISCC.exe`での実際の再コンパイルに成功
     (`open-english-setup-0.4.0.exe`)。
  7. **実機検証**: `open-english-server`を実際にビルド・起動し、ブラウザ
     で全コントロール(年齢層/レベル/ビジネス英会話/検索トグル等)が
     正しく表示されること、新規JS関数(`speakBilingual`・
     `advanceTrainingMode`・`toraHandoffLine`)がエラー無くロードされる
     こと、コンソールエラーが`aruaru-llm`未起動による接続エラー
     (想定通り・既存の挙動)のみであることを確認した(白画面バグ等を
     見逃さない検証徹底ルールに基づく)。
  8. **正直な開示・今回スコープ外(ユーザーから複数回要望あり、次回以降の
     課題として明記)**: (a) 現在のハードウェア環境からの推薦LLM・
     一段階小さい/大きいLLMの特徴・メリットデメリットを日英表示する機能、
     (b) 起動時メンテナンス中に最新LLM・最新NVIDIA/AMD/Intel GPU情報を
     収集してDB化する機能、(c) 3Dオンラインゲーム(基本無料・課金制含む)
     の解像度(フルHD/WQHD/4K/5K)別120FPS対応・推定FPS表示、
     (d) AmazonのUSA版/日本版でのGPU購入リンク表示——いずれも広範な
     最新情報の継続調査を要し、このセッションのスコープには含めていない。
     特に(d)は実際の購入操作を伴うため、ユーザー自身の確認・操作が必要な
     範囲(このエコシステムの安全ルール)である点も次回検討時に踏まえる
     こと。Facebookアプリ版への移植も「後で」と指示された優先度の低い
     項目として未着手のまま記録する。
  - 次にすべきこと: (1) 上記8番の各項目を専用の調査セッションとして
    スコープを切って着手するかどうかの判断、(2) 実際に稼働中の
    `aruaru-db`への接続・seed投入の実機検証(`aruaru-llm`側の課題、
    詳細は`aruaru-llm/CLAUDE.md`参照)、(3) Facebookアプリ版への移植。

- **2026-08-11 Google検索設定パネル+Windowsインストーラーの実機検証完了
  (ユーザー指示「利用者がAPIキーの取得とCOPYペーストが簡単な機能を
  搭載して」+一区切りとしてのインストーラー完成)**:
  1. **Google Search設定パネル**: `aruaru-llm`側に新規`POST/GET/DELETE
     /v1/settings/google-search`(メモリ上保持のみ、ディスクへ保存
     しない、詳細は`aruaru-llm/CLAUDE.md`参照)を実装し、`index.html`に
     「🔎 Google Search Setup」モーダル(日英併記の手順・入力欄・
     保存/消去ボタン・設定状態表示)を追加。実ブラウザで動作確認済み
     (モーダル表示・設定状態の反映を確認)。
  2. **ユーザーとの実機デバッグセッション(正直な記録)**: ユーザーが
     実際にGoogle Cloud ConsoleでAPIキー・検索エンジンID(cx)を取得する
     過程に同行し、複数の実際のトラブルを発見・解決した: (a) 誤った
     種類のキー(Agent Platform API用、サービスアカウント紐付き)を
     最初に使ってしまっていた、(b) Custom Search APIがプロジェクトで
     有効化されていなかった、(c) 請求先アカウントがプロジェクトに
     リンクされていなかった。これらを診断しやすくするため、
     `web_search.rs`のエラーメッセージにGoogle側の実際のレスポンス本文
     (詳細な理由)を含めるよう改善した(`aruaru-llm/CLAUDE.md`参照)。
     **正直な開示**: 最終的に請求先アカウントをリンクした後も
     403エラー(`This project does not have the access to Custom
     Search JSON API`)が解消されなかった——Google Cloud側の設定変更が
     反映されるまでの遅延(公式に報告されている既知の現象、数十分〜
     数時間かかることがある)と判断し、ユーザー側で後日再確認して
     いただくこととした。**実際にAPIキーを使った完全なE2E成功
     (`used_search:true`)はこの時点でまだ確認できていない**——
     コード自体が正しくGoogleへリクエストを送り、正確なエラーを
     表示できることは実証済みだが、成功レスポンスの実物は未確認のまま
     次回へ持ち越す。
  3. **Windowsインストーラーの実機検証(型チェックのみで完了と
     報告しない方針を徹底)**: `installer/windows/open-english.iss`に
     `PrivilegesRequired=lowest`を追加(UAC昇格を不要にする、通常の
     ユーザー権限で動作するアプリのため)。`ISCC.exe`で実際にコンパイル
     し、`open-english-setup-0.3.0.exe`(約3MB)を生成。**実際に
     `/VERYSILENT`でインストール→生成されたサーバーバイナリを実際に
     起動しHTTP 200を確認→アンインストーラーを実行しインストール
     ディレクトリが完全に削除されることを確認**、という一気通貫の
     検証を実施した(Git Bashの`/VERYSILENT`等のパス誤変換という
     ハマりどころも解消済み、`MSYS_NO_PATHCONV=1`が必要)。
  4. **バージョン**: `version.json`は`0.3.0`のまま(既存のインストーラー
     関連の変更のみ、機能追加は別コミットで記録済み)。
  - 次にすべきこと: (1) Google Cloud側の反映待ちが解消された後、
    実際に`used_search:true`となるE2E成功例を確認する、(2) Android
    アプリの実機↔PC間接続(通常のWi-Fiでの)最終確認、(3) タブレット
    向けの実機検証(現状は「同じAndroidアプリがそのまま動く」という
    設計上の判断のみで、実タブレット端末での確認はまだ)。

- **2026-08-10(続き5) Google Custom Search連携(ブリッジ式)+
  Android/Windowsインストーラー着手(ユーザー指示「人がしゃべったり
  文字を入力したら、その都度Google検索するような仕様にして」+
  「WindowsとAndroidスマホとタブレット用のインストーラー付きアプリを
  バージョン管理機能付きで開発して」への対応)**:
  1. **Google検索ブリッジ**: `aruaru-llm`側に新規`POST /v1/generate-
     with-search`(詳細は`aruaru-llm/CLAUDE.md`同日HANDOFF参照)を実装。
     `app.js`に「Google search boost / Google検索で補強」トグル
     (`#web-search-toggle`)を追加し、ONの場合はこの新エンドポイントを
     呼ぶよう配線した。**正直な開示**: (1) Google Custom Search JSON
     APIの利用にはユーザー自身のAPIキー・検索エンジンID取得が必要
     (`aruaru-llm`はキーを同梱・保持しない)。(2) このパスでは
     **実際にAPIキーを設定した状態でのE2E検証(本当にGoogle検索結果が
     返り応答に反映されること)は未実施**——実ブラウザで確認できたのは
     APIキー未設定時のフォールバック動作(`used_search:false`が正しく
     UIに表示されること)のみ。次回、実際のAPIキーを用意できた際に
     本番のE2E検証を行うことを次にすべきことへ記録する。(3) 検索結果
     タイトルは外部サイト由来のテキストのため`innerHTML`は使わず
     `textContent`(プレーンテキスト)で表示する設計とし、XSSリスクを
     回避した。
  2. **Android WebViewアプリの実機検証**(前回セッションの続き):
     `android/`(`tokyo.runo.openenglish`)をビルドし、実機
     (moto g53y 5G、adb経由)へインストール・起動を確認。**正直な開示・
     判明した制約**: 実機のWi-Fiが`FreeWifi aon.co.jp TP Guest 5G1`
     という**ゲスト用ネットワーク**(クライアント分離が有効な可能性が
     高い)に接続されていたため、同一サブネット上のPCサーバーへの接続が
     `net::ERR_CONNECTION_TIMED_OUT`で失敗した——これはアプリ側の
     バグではなくネットワーク環境側の制約(ゲストWi-Fiのデバイス間
     通信ブロック)であることを確認済み。実際のPC↔スマホ間接続の
     最終確認は、スマホを通常のWi-Fiへ接続し直した上で次回行う必要が
     ある。
  3. **タブレット対応**: 専用の別アプリは作らず、同じ`android/`
     (WebViewラッパー)がタブレットでもそのまま動作する設計とした
     (レスポンシブなWebページをそのまま表示するだけのため、画面
     サイズに応じた特別な分岐は不要と判断)。
  4. **Windowsインストーラー**(前回セッションの続き): `installer/
     windows/open-english.iss`(Inno Setup)を作成済み。アンインストーラー
     はInno Setupが自動生成・レジストリ登録する標準機能を利用。
     **正直な開示**: このパスでは`ISCC.exe`での実際のコンパイル・
     生成された`.exe`インストーラーの実行検証までは実施していない
     (次にすべきこと参照)。
  5. **バージョン管理**: `version.json`を`0.3.0`へ更新。
  6. **検証**: `aruaru-llm`側`cargo test --release`51件全green
     (既存46件+cache_optimizer3件+web_search2件)、実HTTP経由での
     フォールバック動作・実ブラウザでのUI表示を確認済み。
  - 次にすべきこと: (1) 実際のGoogle Custom Search APIキーを用いた
    E2E検証(検索結果が実際に応答へ反映されることの確認)、
    (2) スマホを通常のWi-Fiへ接続し直した上でのPC↔スマホ間接続の
    最終確認、(3) `ISCC.exe`での実際のWindowsインストーラー
    ビルド・実行検証、(4) 東芝SBM/DeepSeek技術組み込み構想は
    `aruaru-llm`側で実装済み(`cache_optimizer.rs`、詳細は
    `aruaru-llm/CLAUDE.md`参照)。

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
