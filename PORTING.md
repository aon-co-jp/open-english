# PORTING.md — open-english を他プロジェクトへお引越しする際のガイド

他プロジェクトへ`open-english`の実装パターンを移植する際の要点をまとめる。

## 1. aruaru-llm連携パターン(CORS+繰り返しペナルティ前提)

`app.js`の`askTrainer`関数は、`aruaru-llm`の`POST /v1/generate`を
直接fetchする最小構成。移植先で同じパターンを使う場合、以下2点が
`aruaru-llm`側に実装済みであることを確認すること(2026-08-10以降):

1. **CORS対応**: `aruaru-llm/src/main.rs`の`Route`チェーン末尾に
   `.with_cors()`(`open-runo-poem-compat`提供、`RPoem`側実装)が
   呼ばれていること。無い場合、別オリジン(別ポート/`file://`)からの
   `fetch()`はブラウザにブロックされる。
2. **繰り返しペナルティ**: `open-cuda`側`GptModel::generate_with_
   repetition_penalty`が`aruaru-llm`の`generate()`から呼ばれている
   こと(既定`penalty=1.3`)。無い場合、対話ファインチューニング無しの
   素のGPT-2貪欲デコードが同一文字列の無限ループに陥る既知の劣化モードに
   遭遇しうる。

## 2. 発話テキストの言語抽出パターン(`extractSpeechText`)

`app.js`のメッセージは全て`"English sentence / 日本語訳"`という
バイリンガル1行形式の規約で書かれている。この文字列をそのまま
`SpeechSynthesisUtterance`へ渡すと、単一言語の`utter.lang`設定と
実際の混在テキストが噛み合わず、TTSエンジンが不自然に途切れる
(ユーザー報告「喋りが途切れ途切れ」の実際の原因)。

移植手順: `extractSpeechText(text, lang)`関数(各行を`" / "`で分割し、
`lang`が`ja`始まりなら日本語側、それ以外なら英語側だけを抽出して
句点/ピリオドで連結する)をそのままコピーし、`speak()`関数内で
`new SpeechSynthesisUtterance(text)`を`new SpeechSynthesisUtterance(
extractSpeechText(text, lang))`に置き換えること。

## 3. ランチャーアイコン一式(`icons/` + `manifest.json` + `launchers/`)

外部画像ツール(ImageMagick/PIL/Inkscape等)非依存で、Node.js標準の
`zlib`のみを使った手書きPNGエンコーダ(`tools/gen-icons.js`)・Windows
`.ico`エンコーダ(`tools/gen-ico.js`、PNGバイト列をそのまま埋め込む
モダンICO形式)でアイコンを生成している。移植手順:

1. `tools/gen-icons.js`・`tools/gen-ico.js`をコピーし、
   `drawIcon`関数内の配色・図形(角丸背景・顔・髪・襟等)を対象アプリの
   ブランドに合わせて書き換える。
2. `node tools/gen-icons.js && node tools/gen-ico.js`を実行し
   `icons/`配下にPNG(32/180/192/512px)+`.ico`を生成。
3. `manifest.json`(PWA)・`index.html`の`<link rel="manifest">`/
   `<link rel="icon">`/`<link rel="apple-touch-icon">`をコピーし
   パスを調整。
4. `launchers/windows/create-shortcut.ps1`(Windows `.lnk`作成)・
   `launchers/linux/install-launcher.sh`(`.desktop`)・
   `launchers/mac/build-app.sh`(macOS上で実行する`.app`ビルド
   スクリプト、`sips`/`iconutil`必須)・`launchers/mobile/README.md`
   (PWA「ホーム画面に追加」手順)をコピーし、パス・アプリ名を書き換える。
   **注意**: `create-shortcut.ps1`は`-RepoRoot`引数を明示的に渡す運用
   にすること(`$PSScriptRoot`/`$MyInvocation.MyCommand.Path`が
   一部の実行経路〈ネストしたpowershell呼び出し等〉でnullになる実例が
   あったため、自動検出に頼らずフォールバック引数を用意している)。

## 4. 自動更新機能(`auto-update.js` + `version.json`)

`version.json`の`buildId`を5秒間隔でポーリングし、初回読み込み時の値と
異なれば`location.reload()`する単純な方式。移植手順:

1. `auto-update.js`・`version.json`をコピーし、`index.html`に
   `<script src="auto-update.js"></script>`を追加(他のscriptタグより
   後に置くこと)。
2. コード変更を行うたびに`version.json`の`buildId`を手動で更新する
   運用ルールを、移植先のCLAUDE.md等に明記すること(自動ビルド
   パイプラインが無い前提の設計、CI/CDがある場合はビルド時刻を
   自動注入する形に発展させてもよい)。
3. **正直な開示**: `file://`直開き環境では、ブラウザによってはローカル
   ファイルへの`fetch()`がブロックされ機能が無効化される(エラーを
   握りつぶして静かに何もしないだけで、他機能への影響は無い)。確実に
   動かすには簡易HTTPサーバー経由での配信が必要。

## 4a. RPoemベースの静的配信サーバー(`server/`)——`python3 -m http.server`代替

`python3 -m http.server`はPythonの有無・バージョンに依存し、
`auto-update.js`の`fetch()`が`file://`ではブロックされうるという既存の
制約への恒久対応として、`server/`(独立Rustクレート`open-english-server`)を
新設した。移植手順:

1. `server/Cargo.toml`を作成し、`open-runo-poem-compat = { path =
   "<RPoemリポジトリへの相対パス>/crates/open-runo-poem-compat" }`を
   path依存として追加(このエコシステムのsibling-repo path依存パターン、
   `aruaru-llm`の`opencuda-llm`依存等と同じ)。
2. `server/src/main.rs`で`open_runo_poem_compat::hyper_compat::
   static_file_handler(path, content_type)`(**既存関数の再利用、
   新規実装は不要**)を配信したい静的ファイルの数だけ`Route::new()
   .at(url_path, get(static_file_handler(...)))`へ登録し、
   `Server::new(TcpListener::bind(addr)).run(app)`で起動する。
3. **正直な開示**: このハンドラは`tokio::fs::read`でリクエストごとに
   ディスクから読み直す設計(埋め込み配信ではない)——配布時はこの
   バイナリと静的ファイル群を同じ相対位置(`server/`の親ディレクトリ)に
   置く必要がある。HEADメソッドは未登録(GETのみ)。

## 4b. ハイブリッド(英日併記)応答の構造的保証(`ensureHybridReply`)

英語中心のBPE語彙で事前学習された小型LLM(GPT-2系)は、プロンプトで
「英語と日本語を混ぜて返答して」と指示しても、日本語を一切含まない
応答を返すことが多い(モデルの本質的な限界であり、プロンプト調整では
解決しない)。移植手順:

1. `containsJapanese(text)`(ひらがな・カタカナ・漢字のUnicodeプロパティ
   エスケープ`\p{Script=Hiragana}`等での判定)と`ensureHybridReply
   (completion, userText)`をコピーする。
2. モデルの応答に日本語が含まれない場合、機械翻訳の質を偽って主張せず、
   定型の短い日本語の一言(「このAIはまだ日本語が苦手です」等)を
   フロントエンド側で追記するに留める——これにより「英日併記」という
   *構造*だけは常に保証できる(意味内容の翻訳精度は保証しない)。
3. ユーザー発話が日本語の場合はその事実をプロンプトへ明示すると
   (`containsJapanese(userText)`で判定)、モデルが日本語混じりの応答を
   返す確率がやや上がる(保証はされない、実測ベースで確認すること)。

## 4c. バージョン管理+旧バージョンのブラウザ側クリーンアップ

`version.json`に`buildId`(自動更新のトリガー用)に加え`version`
(セマンティックバージョン、画面への表示用)を追加。移植手順:

1. `version.json`に`{"version": "x.y.z", "buildId": "..."}`の形式で
   両方保持する。
2. アプリのフッター等に`fetch("version.json")`で取得した`version`を
   表示する(`app.js`の`showAppVersion`参照)。
3. **正直な開示・スコープ**: このアプリはネイティブインストーラーを
   持たない静的Webアプリのため、「旧バージョンの自動アンインストール」は
   ディスク上のファイル削除としては実装していない(他のファイルを
   誤って削除するリスクを避けるため)。代わりに、新バージョン検出時に
   このアプリ専用の名前空間(`openEnglish.`接頭辞)を持つ
   `localStorage`キーを削除し、キャッシュ破棄付き(`?v=<buildId>`
   クエリ付き)で再読み込みする、という「ブラウザ側の痕跡クリーンアップ」
   に限定している(`auto-update.js`の`clearOwnLocalStorage`/
   `reloadBustingCache`参照)。ネイティブアプリへ移植する場合は、
   この節の設計をそのまま流用せず、OS標準のアンインストーラー機構
   (Windowsのレジストリアンインストールエントリ等、`RPoem/apps/
   desktop-tray`のInno Setup採用例を参照)を使うこと。

## 4d. Google Custom Search JSON APIブリッジ + ブラウザ設定パネル(2026-08-11)

`aruaru-llm`側`POST /v1/generate-with-search`(検索結果でプロンプトを
補強するブリッジ式)と、ブラウザから直接APIキー/cxを保存できる設定
パネル(`POST /v1/settings/google-search`、`aruaru-llm/src/web_search.rs`の
`RUNTIME_CREDENTIALS`——プロセスメモリ上のみ、ディスク非保存)を追加した。
移植手順:

1. `web_search.rs`をコピーし、`RUNTIME_CREDENTIALS`(実行時設定)→
   環境変数の順にフォールバックする`read_credentials()`パターンを
   踏襲する。
2. **正直な開示**: ユーザー自身のGoogle Cloud Console契約(無料枠あり、
   超過分は課金)が前提——このエコシステム共通の「契約不要」方針への
   意図的な例外であることを利用規約・UI双方に明記すること。
3. **セキュリティ配慮(最重要)**: APIキーはログ・ディスクに一切残さない。
   `GET /v1/settings/google-search`はキー値自体を返さず設定済みかどうかの
   真偽値のみ返す。Google側のエラー応答本文はキー値をエコーバックしない
   仕様を確認済みのため、そのままユーザーへ表示しても安全。

## 4e. Windowsインストーラー(Inno Setup)の実機検証パターン(2026-08-11)

`installer/windows/open-english.iss`で得られた、他リポジトリでも
再利用できる落とし穴と対処:

1. **UAC昇格ハング対策**: `PrivilegesRequired=lowest`を`[Setup]`へ追加
   しないと、管理者権限を要求しないアプリでも既定でUAC昇格を要求し、
   非対話的な`/VERYSILENT`インストール検証がGUIプロンプト待ちで無限に
   ハングする。
2. **Git Bashでのスイッチ誤変換対策**: Git Bash/MSYSは`/VERYSILENT`
   `/SUPPRESSMSGBOXES`のような`/`始まりの引数を偽のWindowsパスへ自動
   変換してしまう。`MSYS_NO_PATHCONV=1`を先頭に付けて回避する。
3. **検証手順**: `ISCC.exe`でビルド→`MSYS_NO_PATHCONV=1 ./setup.exe
   /VERYSILENT /SUPPRESSMSGBOXES /DIR=... /LOG=...`でサイレント
   インストール→インストール済みバイナリを実際に起動しHTTP応答を確認→
   `unins000.exe /VERYSILENT /SUPPRESSMSGBOXES`でアンインストール→
   インストールディレクトリが実際に消えたことを確認、まで一気通貫で
   行うこと(ビルド成功だけで「完成」と報告しない既存方針の徹底)。

## 4f. 地理・観光DB連携+話題駆動の安全案内パターン(2026-08-11)

`aruaru-llm`側`geo_content.rs`(`GET /v1/geo/random`・`POST /v1/geo/
lookup`・`GET /v1/geo/fuji`・`POST /v1/geo/tours`)と連携し、会話の
話題(国名・山名)に応じて動的にコンテンツを差し込むパターン。移植手順:

1. **部分一致検索を使うこと(最重要、実際に踏んだ落とし穴)**: ユーザーの
   発話文全体("I from Japan."等)を検索クエリとして渡す設計の場合、
   完全一致(`==`)ではなく部分一致(`contains`/SQLの`LIKE '%...%'`)で
   検索すること。完全一致のみだと実際の発話文では一致しない(実機
   テストで発見した実バグ、`aruaru-llm/CLAUDE.md`同日HANDOFF参照)。
2. **危険な話題(登山等)には安全案内を必ず添える**: 富士山のように
   実際に死亡事故が起きうる話題を扱う場合、教材として面白い情報
   (ランドマーク・名物)だけでなく、安全上の注意(装備・事前予約の
   必要性)を必ず併記すること。出典元(公式サイトのURL)と収集時点を
   明記し、「営業期間・電話番号は毎年変わるため利用前に直接確認する
   こと」を正直に開示する(`fuji_source_ja`/`fuji_source_en`パターン)。
3. **観光ツアー検索は既存のGoogle Custom Search連携を再利用**:
   新規の検索APIクライアントを増やさず、`web_search.rs`
   (`/v1/generate-with-search`と同じAPIキー設定)をそのまま呼ぶ。
   YouTube検索は専用API連携までは実装せず、URLエンコード済みの検索
   結果ページへの直リンクで代替してよい(過剰実装を避ける)。

## 4g. 起動時の自動アップデート(GitHub Releases検出→自動アンインストール
→自動インストール)パターン(2026-08-11)

`server/src/self_update.rs`——ネイティブ配信サーバー(Rust)が起動時に
GitHub Releasesの最新版を確認し、新しければWindowsアプリを自動更新する
パターン。移植手順・落とし穴:

1. **ブラウザ側JSでは絶対に実装できない**: アンインストール・
   インストールという特権操作はネイティブコード側でしか行えない
   ——`auto-update.js`(ページ再読み込みのみ)とは別レイヤーとして
   位置づけること。
2. **実行中の自分自身のexeは削除できない(Windows特有の制約)**:
   アンインストーラー/新インストーラーを起動する前に、必ず**別プロセス
   (一時バッチスクリプト)をデタッチ起動してから、このプロセス自身を
   `std::process::exit`で終了する**こと。同一プロセス内で
   `Command::new(uninstaller).status()`のように同期的に呼んでしまうと、
   実行中のexe自身をアンインストーラーが削除しようとして失敗する。
3. **バージョン比較は安全側に倒す**: パース不能なバージョン文字列は
   `(0,0,0)`扱いにし、「不明時は最新と誤判定しない」設計にすること
   (`parse_version`参照)。
4. **GitHub Releaseが存在しない場合(初回リリース前)を必ず考慮**:
   `GET .../releases/latest`は`404`を返す——これを異常終了させず、
   ログに記録して通常起動を継続すること(このリポジトリ自身、
   2026-08-11時点でまだリリースが1件も無い状態でこの分岐を実機確認
   済み)。
5. **CI(`.github/workflows/release.yml`)でインストーラーアセットを
   自動生成**: `choco install innosetup`でInno Setupを導入し
   `ISCC.exe`でビルド、ファイル名に"setup"を含む`.exe`としてGitHub
   Releaseへ添付する(`self_update.rs`側のアセット検出条件
   `name.contains("setup")`と対応させること)。

## 5. 実在の接客技法・文化コンテンツを教材へ翻案する際の著作権配慮

参考記事(ブログ等)の技法・実例を教材化する際は、記事本文を丸ごと
転載せず、技法自体を短い引用・要約(15語未満の直接引用+出典明記)に
留めて独自の練習問題へ翻案すること(`trainingSteps`内の秋葉原メイド
カフェ技法の実装を参考にすること)。出典クレジットはページ内の
disclosure領域に明記する。

## 注意事項

本プロジェクトはPhase 0の試作品であり、AI応答品質(GPT-2ベース)・
音声合成の自然さ・レベル別対応の確実性は保証されていない旨を、
移植先でも必ず明記すること(誇大表示の回避、このエコシステム共通の
「正直な開示」規約)。
