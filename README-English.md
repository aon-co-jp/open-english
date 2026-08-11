# open-english

> 📌 **Recent update (2026-08-11, continued 2)**: Added detection for
> job-hunting/career-change/tourism topics that introduces aruaru.tokyo
> (AI-driven development, Claude Code Desktop), audiocafe.tokyo/aruaru
> (IT/construction jobs), audiocafe.tokyo/aruaru-lady (jobs for women),
> and nasa.tokyo in both English and Japanese — works in both normal
> chat and training mode, verified live. See the 2026-08-11 (continued
> 4) HANDOFF entry in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 就職・転職・観光の話題を検出し、aruaru.tokyo・
> audiocafe.tokyo/aruaru・aruaru-lady・nasa.tokyoへのリンクを日英併記で
> 案内する機能を追加(通常チャット・研修モード両方)。

> 📌 **Older update (2026-08-11, continued)**: Linked to a new geo/
> tourism database (all 47 Japanese prefectures, 50 US states, major
> world capitals with landmarks/food/souvenirs) to make the self-
> introduction training dynamic. When Mount Fuji comes up, the app now
> shows a bilingual safety advisory (wear ski gear + a helmet, reserve
> a mountain hut in advance) plus real hut/bus/gear-shop info and a
> tour-booking search. Added age-group/level/business-English selection
> UI. Verified live against a real running `aruaru-llm` + static server
> (found and fixed 3 real bugs in the process). See the 2026-08-11
> HANDOFF entries in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 地理・観光DBと連携し自己紹介研修の話題を動的化。富士山の
> 話題では安全上の注意・山小屋/バス/登山用品店情報・観光ツアー検索を
> 日英併記で案内。年齢層・レベル・ビジネス英会話選択UIも追加。実機で
> 検証し3件のバグを修正済み。

> 📌 **Older update (2026-08-11)**: Added a settings panel for saving
> your Google Search API key/cx directly from the browser
> (`POST /v1/settings/google-search`, in-memory only, never written to
> disk). The Windows installer (`installer/windows/`, Inno Setup) has
> now been actually built, installed, launched, and uninstalled on real
> hardware (no admin rights required). See the 2026-08-11 HANDOFF entry
> in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: ブラウザから直接Google検索APIキー・cxを保存できる設定パネルを
> 追加(メモリ上保持のみ)。Windowsインストーラーを実際にビルド・
> インストール・起動・アンインストールまで実機検証済み。

> 📌 **Older update (2026-08-10, continued)**: (1) Switched the default
> model from `gpt2` (124M) to `distilgpt2` (82M), ~42% faster (see
> `aruaru-llm/CLAUDE.md`). (2) Decided **against** porting the frontend
> JS to Rust/WASM (no performance benefit, and `SpeechRecognition` has
> no standard web-sys binding) — instead **ported the local file
> server to Rust** (new `server/` crate, built on RPoem's
> `open-runo-poem-compat`, removing the `python3 -m http.server`
> dependency). (3) Improved Japanese input handling so hybrid
> (English+Japanese) replies are always guaranteed
> (`app.js`'s `ensureHybridReply` — if the model's reply contains no
> Japanese, the frontend appends a short honest Japanese note itself;
> it does not fake machine-translation quality). (4) Added version
> management (`version.json` now has a semantic `version` field, shown
> in the footer) and automatic cleanup of old versions' browser-side
> traces (`auto-update.js` clears this app's own `localStorage` and
> does a cache-busting reload on update — since this is a static web
> app with no native installer, "uninstalling old versions" is scoped
> to browser-side leftovers only, not disk files). See the 2026-08-10
> (continued 3) HANDOFF entry in [CLAUDE.md](CLAUDE.md) for details.
>
> *日本語*: 既定モデルをdistilgpt2へ切替(約42%高速化)、フロントエンドJSの
> Rust移植は見送り配信サーバー側をRust化、日本語入力時のハイブリッド
> 応答を保証、バージョン管理+旧バージョンのブラウザ側クリーンアップを
> 追加した。詳細は[CLAUDE.md](CLAUDE.md)参照。

> 📌 **Recent update (2026-08-10)**: Added CORS support (`.with_cors()`
> on the `aruaru-llm` side), fixed the root cause of GPT-2 greedy-
> decode's degenerate repetition loop (`open-cuda`'s `GptModel::
> generate_with_repetition_penalty`, default `penalty=1.3`), tweaked
> the Tora-san character's look (bigger light-brown bag, straw-sandal-
> style feet) + added a switch-in jingle + fixed his self-introduction,
> added a training step based on a real Akihabara maid cafe's
> (@ほぉ～むカフェ) actual customer-service technique, researched (in
> Japanese and English) and added a step covering the current overseas
> boom in Japanese culture (anime/manga, anime songs, games, Japanese-
> language learners, goshuin stamp collecting, onsen ryokan tourism,
> Japanese food), added launcher icons for Windows/Mac/Linux/Android/
> iPhone/iPad (`icons/`+`launchers/`+`manifest.json`), and added an
> auto-update mechanism (`auto-update.js` polling `version.json`). See
> the 2026-08-10 HANDOFF entry in [CLAUDE.md](CLAUDE.md) for details.
>
> *日本語*: CORS対応・GPT-2反復ループの根本解決・トラさんの見た目調整・
> 実際のメイドカフェ接客技法の反映・日本文化ブームの調査反映・全プラット
> フォーム向けランチャーアイコン・自動更新機能を追加した。詳細は
> [CLAUDE.md](CLAUDE.md)参照。

A browser-based (Phase 0) English-conversation learning web app for
PC/tablet/smartphone. In the style of a "maid cafe English lesson,"
a magical-girl maid character (original design, animated) coaches
students from complete beginner to advanced.

## Architecture (per user instruction, 2026-08-10)

- **Linux (VPS) side**: only a download-distribution server (not where
  this app actually runs). App management is handled by
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web).
- **User's device (PC/tablet/phone) side**: this repo's static web
  frontend (HTML/CSS/JS, runs in the browser) + a locally-run native
  server from [`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm)
  (which internally uses `open-directx`/`open-cuda`'s inference
  backend), which the user downloads and runs themselves. The browser
  connects to `http://localhost:4600` (aruaru-llm's default port)
  locally, online or offline — a "hybrid" design.

## Current scope (Phase 0) — honest disclosure

- **AI response quality**: `aruaru-llm`'s `/v1/generate` performs
  autoregressive text generation via GPT-2 (124M-1.5B, English-centric,
  no dialogue fine-tuning). Response quality is not guaranteed to be
  fluent or level-appropriate — this is disclosed on-screen, not
  overstated. As of 2026-08-10, a repetition penalty (default 1.3,
  `ARUARU_LLM_REPETITION_PENALTY` env var) fixes the previously-reported
  degenerate infinite-repeat bug (e.g. endless "Student: Hello").
- **CORS**: fixed as of 2026-08-10 — `aruaru-llm`'s HTTP server now
  sends `Access-Control-*` headers via `open-runo-poem-compat`'s
  `.with_cors()`, so this frontend can be opened cross-origin (or via
  `file://`) and still reach `http://localhost:4600`.
- **Level selection**: the beginner-to-advanced level picker exists in
  the UI, but actual level enforcement is limited to a short prompt
  instruction — GPT-2 is not guaranteed to honor it.
- **Voice**: real Web Speech API (SpeechSynthesis for output,
  SpeechRecognition for mic input) is wired up, with per-character
  pitch/rate tuning (maid vs. Tora-san helper) and a
  language-extraction fix (2026-08-10) so mixed English/Japanese lines
  no longer sound choppy when spoken.
- **Training mode**: a deterministic self-introduction script (not
  AI-generated) that now also includes a real Akihabara maid cafe's
  (@ほぉ～むカフェ) word-based conversation technique (e.g. "Where are
  you from?" -> "Australia!" -> "Kangaroo!!"), and a step summarizing
  the current overseas boom in Japanese culture (researched in both
  Japanese and English): anime/manga (Demon Slayer, Attack on Titan),
  anime song live events (Animelo Summer Live), Japanese video games,
  ~3.79 million Japanese-language learners worldwide, goshuin (temple/
  shrine stamp) collecting among tourists, onsen ryokan and shrine/
  temple tourism, and Japanese food.
- **Launcher icons**: `icons/` + `manifest.json` (PWA) + `launchers/`
  (Windows `.lnk` creation script, Linux `.desktop` file, macOS `.app`
  builder script, and a mobile PWA "Add to Home Screen" guide) let
  users launch this app from a Windows/Mac/Linux desktop icon or an
  Android/iPhone/iPad home-screen icon.
- **Auto-update**: `auto-update.js` polls `version.json` every 5s and
  reloads the page when the build ID changes. **Known limitation**:
  some browsers block `fetch()` of local files under the `file://`
  scheme for security reasons — this feature is guaranteed to work when
  served over a local HTTP server (see `launchers/mobile/README.md`
  for a one-line `python3 -m http.server` example), and silently
  no-ops (doesn't break anything) if blocked under `file://`.

## How to run

1. Run [`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm) with
   `cargo run --release` (default `http://localhost:4600`, default
   model is now `distilgpt2`).
2. In `server/`, run `cargo run --release` to serve this repo's static
   frontend at `http://127.0.0.1:4601/` (RPoem-based — `python3 -m
   http.server` is no longer needed; override the port with the
   `OPEN_ENGLISH_SERVER_BIND` env var).
3. Open `http://127.0.0.1:4601/` in a browser. Opening `index.html`
   directly via `file://` still works, but some browsers block
   `fetch()` there and disable auto-update — the server in step 2 is
   recommended.

## Next steps

1. ~~CORS support on the `aruaru-llm` side~~ **Done (2026-08-10)**.
2. ~~GPT-2 greedy-decode repetition loop~~ **Root cause fixed
   (2026-08-10, repetition penalty)**.
3. ~~Speed up the default model~~ **Done (2026-08-10, switched to
   distilgpt2, ~42% faster)**.
4. ~~Guarantee hybrid replies for Japanese input~~ **Done
   (2026-08-10)**.
5. ~~Rust-ify the local file server~~ **Done (2026-08-10, `server/`
   crate)**. Porting the frontend JS itself to Rust/WASM was
   evaluated and skipped (no performance benefit — see `CLAUDE.md`).
6. Add TTS/lip-sync animation polish.
7. Implement a per-level curriculum (grammar, vocabulary lists, etc.).
8. **(per user instruction, 2026-08-10)** A future idea to run
   `open-directx`/`open-cuda`/`aruaru-llm` in-browser (WASM/WebGPU) and
   integrate with `RPoem` (a GraphQL Federation platform). This is a
   large, separate architectural direction from the current Phase 0
   (local resident server + localhost connection) design, deferred
   until after the MVP is complete and scoped as its own effort.
9. Investigate whether Toshiba SBM or DeepSeek-family techniques have
   any genuine application here (not yet started).
