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
