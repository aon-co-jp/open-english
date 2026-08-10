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
