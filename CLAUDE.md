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
