# open-english

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
   `cargo run --release` (default `http://localhost:4600`).
2. Open this repo's `index.html` in a browser.

## Next steps

1. ~~CORS support on the `aruaru-llm` side~~ **Done (2026-08-10)**.
2. ~~GPT-2 greedy-decode repetition loop~~ **Root cause fixed
   (2026-08-10, repetition penalty)**. Swapping to a genuinely
   dialogue-tuned model is in progress (see "In progress in a separate
   session" below).
3. Add TTS/lip-sync animation polish.
4. Implement a per-level curriculum (grammar, vocabulary lists, etc.).
5. **(per user instruction, 2026-08-10)** A future idea to run
   `open-directx`/`open-cuda`/`aruaru-llm` in-browser (WASM/WebGPU) and
   integrate with `RPoem` (a GraphQL Federation platform). This is a
   large, separate architectural direction from the current Phase 0
   (local resident server + localhost connection) design, deferred
   until after the MVP is complete and scoped as its own effort.

## In progress in a separate session (started 2026-08-10)

Per user instruction, the following large-scope tasks are being
worked on in a separate background session (task `task_076ef43b`):
1. Investigating swapping GPT-2 for a genuinely dialogue-tuned model,
   with Japanese+English web research (including best practices from
   existing English-conversation apps).
2. Migrating the frontend (`app.js`) to Rust+RPoem
   (`RPoem/apps/desktop-wasm` pattern).
3. Investigating/implementing GPT-2 generation-speed bottleneck
   improvements across `open-directx`/`open-cuda`/`aruaru-llm`/RPoem
   (measured, not just theoretical).
4. Investigating whether Toshiba SBM or DeepSeek-family techniques
   (Multi-Token Prediction, speculative decoding, etc.) have any
   genuine application here (no forced-fit; honest findings recorded
   either way).

Check that session and each repo's CLAUDE.md HANDOFF for progress.
