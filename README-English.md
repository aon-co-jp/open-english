# open-english

*日本語*: [README.md](README.md) ·
*Other languages*: [Deutsch](README-German.md) · [Italiano](README-Italian.md) ·
[Français](README-French.md) · [Русский](README-Russian.md) ·
[Українська](README-Ukrainian.md) · [עברית](README-Hebrew.md) ·
[فارسی](README-Persian.md) · [العربية](README-Arabic.md)

> 📌 **Latest update (2026-08-22, continued)**: Added **persistent settings, a native
> language setting, customisable display order, a topic briefing, and a 130-language
> registry**.
> - **Install / uninstall languages**: in the "🌐 Languages" panel, ticking a box
>   installs (adds) a language and unticking uninstalls (removes) it. You can also pick
>   **one native language**; together with the languages you are learning you can have
>   **up to 6 entries** (English and Japanese are always on, plus up to 3 more and your
>   native language). The 130-language list can be **filtered by language name or by
>   country**, and each row shows a **flag emoji and country name**.
> - **Ordering**: set the display and read-aloud order in three interlinked ways —
>   (1) type a number, (2) pick a radio button 1–6, (3) use ▲ / ▼. Changing one updates
>   the others. Picking a number another language already uses swaps the two, so numbers
>   never collide.
> - **Settings survive maintenance and automatic updates**: they are written to both
>   browser localStorage and the local SQLite database, and restored from the database if
>   localStorage is cleared. `auto-update.js` now explicitly preserves these keys when it
>   purges old-version data.
> - **Topic briefing**: after you choose your languages, a progress display ("gathering
>   information / maintenance in progress") collects background about the region of your
>   top-ranked language. **News headlines are genuinely fetched from the internet** (a
>   public Google News RSS feed, headlines only — never article text). Capital, major
>   cities, sights, food, famous people and well-known companies (with one-line summaries)
>   come from static data written for this app. A button then hands the topics to the AI
>   tutor for conversation practice.
> - **Language registry expanded to 130** — but **only 40 of them (English, Japanese and
>   38 more) actually have practice questions and detailed background data**. The other 90
>   are listed with name, flag and country only, and the UI says so plainly. This is a
>   staged rollout, **not full support for 130 languages**. The 130 per-language doc
>   folders in [`docs/i18n/`](docs/i18n/) are mostly untranslated placeholders; no machine
>   translation has been pasted in and presented as a finished translation.
>
> *日本語*: 設定の永続化(localStorage+ローカルSQLite)、母国語の指定、表示順の
> 3系統連動指定(数字入力・ラジオボタン・▲▼)、公開RSSからの実ニュース取得を含む
> 話題ブリーフィング、国旗・国名付きの言語一覧と国名検索、対応言語一覧の130言語化を
> 追加しました。ただし実際に問題・地域データがあるのは40言語のみで、残り90言語は
> 名前のみの段階的拡大です。詳細は[CLAUDE.md](CLAUDE.md)の2026-08-22 HANDOFF参照。

> 📌 **Latest update (2026-08-22)**: Added **world-language practice
> exams, a language-selection UI, and sequential multilingual display &
> read-aloud**. English and Japanese remain the defaults, but a
> bilingual banner and the "🌐 Languages" panel let you enable original
> practice sets for **38 languages** (Europe, Middle East, Asia, India,
> Africa). After scoring, the missed items flow into conversation
> practice with the tutor for that language, exactly like the existing
> Eiken/TOEIC/TOEFL/JLPT flow. You can also select **2–5 languages**
> (including English and Japanese) and have the same phrase displayed
> and read aloud in order, replayable as often as you like (all at once
> or one language at a time), with copy/paste, .txt download, and
> save-to-local-SQLite. Honest disclosure: these are original questions
> written for this app — not past questions from, and not affiliated
> with or endorsed by, any real certification exam (DELE, DELF,
> Goethe-Zertifikat, HSK, TOPIK, ...). CEFR-style levels (A1–C2) are
> loose approximations only, item counts are uneven (3–6 per language),
> and read-aloud uses the browser's built-in Web Speech API, so a
> language with no installed voice is displayed but not spoken. See the
> 2026-08-22 HANDOFF entry in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 世界38言語のオリジナル擬似模擬試験・言語選択UI(2〜5か国語)・
> 多言語の連続表示/読み上げ(何度でも再生可)・コピー&ペースト/ファイル
> 保存/DB保存を追加しました。実在の資格試験の過去問ではなく、それらとは
> 無関係なオリジナル問題です。詳細は[CLAUDE.md](CLAUDE.md)参照。

> 📌 **Latest update (2026-08-20)**: Added periodic automatic update
> checks (every 6 hours, in addition to the startup check) and a
> manual downgrade feature. If a new version turns out to be buggy
> after a while, `GET /v1/updates/history` (current + retained
> previous versions) and `POST /v1/updates/downgrade` (roll back
> open-english itself, aruaru-llm, or aruaru-db individually to a
> specific version) let you revert just that one component. UI: the
> "🔄 Updates & Rollback" section inside the "💾 Data & Model Storage"
> panel. Honest disclosure: only the last 3 generations are retained
> by default (disk-space consideration) — you cannot roll back
> further, or to a version that was never actually applied on this
> machine. See the 2026-08-20 HANDOFF entry in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 定期的な自動アップデートチェック(起動時に加え6時間ごと)+
> 手動ダウングレード機能を追加しました。`GET /v1/updates/history`・
> `POST /v1/updates/downgrade`で、open-english本体・aruaru-llm・
> aruaru-dbのいずれかを個別に旧バージョンへ戻せます。保持世代は
> 既定3世代までです。詳細は[CLAUDE.md](CLAUDE.md)の2026-08-20 HANDOFF
> 参照。

> 📌 **Update (2026-08-19, continued 8)**: When the daily usage
> counter (default 100, client-side `localStorage`) is reached, the
> chat now shows a bilingual notice — "You've exceeded today's free
> usage limit. Would you like to switch to a paid plan?" — plus the
> free-tier info for other AI providers (Google Search/DeepSeek/
> ChatGPT/Gemini/Claude), read dynamically from
> `provider-free-tiers.json`. Honest disclosure: this is a
> notice-only, client-side implementation with no real billing/
> upgrade flow. See the 2026-08-19 (continued 8) HANDOFF entry in
> [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 1日の利用回数上限(既定100回)に到達した際、チャット上に
> 「有料版に切り替えますか？」+他のAIプロバイダの無料枠情報を日英併記
> で表示するようにしました。実際の課金処理は行いません。詳細は
> [CLAUDE.md](CLAUDE.md)の2026-08-19(続き8)HANDOFF参照。

> 📌 **Update (2026-08-19)**: Added Claude (Anthropic) to the
> AI/search free-tier banner as a paid-by-default option (honestly
> noted as having no ongoing free tier, only a small signup credit if
> any). See the 2026-08-19 (continued 5) HANDOFF entry in
> [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 上部のAI/検索無料枠バナーに、有料前提のClaude(Anthropic)
> を選択肢として追加しました。詳細は[CLAUDE.md](CLAUDE.md)の
> 2026-08-19(続き5)HANDOFF参照。

> 📌 **Latest update (2026-08-19)**: Added `facebook.html`, an entry
> page meant to be shared as a link on a Facebook Page or in Messenger,
> for users whose mobile plan only allows Facebook access. Honest
> disclosure: true Facebook "Free Basics"-style zero-rated free access
> is not achievable without an official partnership with Meta, which
> this project does not have — `facebook.html` works as a normal page
> reachable from Facebook's in-app browser and points to the existing
> installers (Windows/Linux/macOS/Android); the app itself still runs
> on a local server on your own device (`server/`). See the 2026-08-19
> HANDOFF entry in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: Facebookしかアクセスできないスマホ契約の利用者向けに、
> Facebookページ/Messengerで共有するリンク先`facebook.html`を新設
> しました。Facebookとの正式提携が無いため「完全無料アクセス」自体は
> 実現できておらず、既存インストーラーへの導線を案内するのみです。

> 📌 **Latest update (2026-08-11–12, v0.6.0)**: Android/tablet now runs
> fully standalone — no PC or Linux server required. The AI response
> engine (`aruaru-llm`) itself is now bundled into the APK; on-device
> verification confirmed both processes stay alive and respond to
> `/healthz`/`/v1/chat`. Also added: a certification exam-prep corner
> (Eiken 1-5, TOEIC, TOEFL, JLPT N1-N5, Nihongo Kentei 1-3, 10 original
> questions each) that hands missed questions to the AI trainer after
> scoring (auto-switching to a "Japanese classroom" mode for JLPT/
> Nihongo Kentei), a "which language to learn" selector, and Linux/
> macOS installers (`installer/unix/install.sh`). Honest disclosure:
> model weights (GPT-2 family, embedding model) are not bundled in the
> APK — using AI chat on Android still requires placing model files in
> internal storage manually (no auto-download yet). See the 2026-08-11
> (continued 7-10) HANDOFF entries in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: Android/タブレットがPC/Linuxサーバー不要で単体動作する
> アプリになりました。AI応答エンジン自体もAPKへ同梱し実機で動作
> 確認済み。資格試験対策コーナー(英検/TOEIC/TOEFL/JLPT/日本語検定、
> 各10問)+採点後のAI講師連携、学びたい言語選択、Linux/macOS版
> インストーラーを追加。モデル重みは未同梱(手動配置が必要)。

> 📌 **Latest update (2026-08-18)**: Started building a proper local
> database for conversation history/settings. **Why not SQLite alone**
> — SQLite remains the always-available local baseline, but when
> `aruaru-db` (PostgreSQL) is configured, writes are also mirrored
> there automatically. Combined with `aruaru-db`'s own
> `DUAL_DATABASE_URL` feature (self-healing mirroring between two
> PostgreSQL instances), **if one database instance fails, the other
> automatically repairs it and protects your data** — a safer setup
> than SQLite alone. If no mirror is configured or the connection
> fails, the app keeps working on SQLite only, so availability is
> never sacrificed. This increment implements SQLite persistence for
> messages/settings plus the `/v1/db/*` API, verified over real HTTP
> (see `server/src/db.rs`). Also added: a storage-location picker
> (`/v1/db/storage-path`), rsync backup (`/v1/db/rsync-backup`), and a
> generic legacy-data import endpoint (`/v1/db/migrate-legacy`). If
> rsync isn't installed, the API replies with a bilingual **"Let's
> install RSync!"** prompt, and `/v1/db/install-rsync` auto-installs it
> via the right package manager for the OS (winget/choco on Windows,
> apt-get/dnf/pacman on Linux, brew on macOS, pkg on Android) and
> immediately retries the backup on success. Usage pie chart display
> and multi-device sync are still planned for the next increment.
>
> *日本語*: 会話履歴・設定の本格的なローカルデータベース化に着手。
> SQLiteを常時利用可能な基盤にしつつ、`aruaru-db`(PostgreSQL)を
> 設定すればそちらへも自動ミラー。`aruaru-db`のDUAL DB自己修復機能と
> 組み合わせ、片側障害時にもう片側から自動修復する安全性の高い構成。
> 未設定時はSQLiteのみで継続動作。保存先選択・rsyncバックアップ・
> 旧データ取り込みも実装済み。rsync未導入時は「RSyncをインストール
> しましょう！」と案内し、自動インストール+成功後の自動バックアップ
> まで1回で完了。円グラフ表示・複数端末同期は次の増分で対応。

> 📌 **Older update (2026-08-11, continued 3)**: Added an automatic
> self-update feature (Windows only) that checks GitHub for the latest
> release at startup and, if newer, automatically uninstalls the old
> version and installs the new one. Honest disclosure: no GitHub
> Release exists yet, so the full uninstall→install flow hasn't been
> end-to-end verified (version-comparison logic and the "no release
> found, continue safely" path have been). See the 2026-08-11
> (continued 5) HANDOFF entry in [CLAUDE.md](CLAUDE.md).
>
> *日本語*: 起動時にGitHubの最新版を自動確認し、新しければ自動で
> アンインストール→自動インストールする機能を追加(Windowsのみ)。
> リリースがまだ無いため一気通貫の動作確認は次回持ち越し。

> 📌 **Older update (2026-08-11, continued 2)**: Added detection for
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

## Required installers (added 2026-08-17)

To run open-english, you need to download and install the following two pieces of
software (no build-from-source required, close to a one-tap install).

| # | What | Windows | Linux | Android/tablet |
|---|---|---|---|---|
| 1 | **open-english itself** (this repo — static frontend + delivery server) | [open-english-install.exe](https://github.com/aon-co-jp/open-english/releases/latest/download/open-english-install.exe) | [tar.gz](https://github.com/aon-co-jp/open-english/releases/latest) | [APK](https://github.com/aon-co-jp/open-english/releases/latest) (pick the `.apk` asset) |
| 2 | **aruaru-llm** (the AI response engine — required, chat won't work without it) | [aruaru-llm-windows-x86_64.zip](https://github.com/aon-co-jp/aruaru-llm/releases/latest/download/aruaru-llm-windows-x86_64.zip) | [tar.gz](https://github.com/aon-co-jp/aruaru-llm/releases/latest) | Bundled already (included inside open-english's APK, no separate install needed) |

**Honest disclosure**: the "latest" links above always point to the newest GitHub
Release (use the [Releases page](https://github.com/aon-co-jp/open-english/releases)
directly if you want a specific pinned version). There is no prebuilt macOS binary for
`aruaru-llm` yet (open-english itself ships a macOS tar.gz, but `aruaru-llm` only ships
Linux/Windows) — on macOS you'll need to build `aruaru-llm` from source.

On Windows/Linux/macOS, after installation the app's built-in auto-update feature
(`server/src/self_update.rs`, extended to Linux on 2026-08-19 and to macOS the same day)
checks GitHub Releases at startup and, if a newer version exists, automatically updates
(Windows: uninstall→install; Linux/macOS: the running binary replaces itself in place) —
no user action required. Before applying an update, the current binary is backed up; after
the new version starts, a health check against the new `/healthz` endpoint must succeed
within a short grace period, or the app automatically rolls back (downgrades) to the
backed-up previous version. **Honest disclosure**: Android/iPhone/iPad are excluded from
this auto-update/auto-rollback mechanism, since the OS does not allow fully silent APK
install — update notifications still require the user to tap through the install
manually (and there is no downgrade path there either).

*(Machine-translation note: this paragraph and the note below were translated by the AI
agent itself, without native-speaker proofreading.)*

There is also a new entry page, `facebook.html`, for users whose mobile plan only allows
access to Facebook — see the 2026-08-19 banner above for details and its honest
disclosure about the limits of this approach.

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


---

## Update 2026-08-23 — `GET /v1/cpu-runtime` extended with ISA *combination* information

Previously the endpoint returned a flat list of CPU feature booleans. Since
real CPUs carry several instruction sets simultaneously (AVX2+FMA3,
AVX-512F+BW+VNNI, …), a flat list does not reveal the actual dispatch
conditions. Using the new combination API in `open-cpu`, the response now
also includes:

- `isa_profile` / `isa_profile_raw` — the satisfied combination tier
- `float_impl` / `bit_impl` — the implementation chosen per kernel
- `combination_examples` — whether `avx2+fma3`, `avx512f+bw+vl`,
  `avx512f+bw+vnni`, `ssse3+pclmulqdq` and `gfni+avx2` hold
- `cpu_vendor`, `cpu_family`, `fast_bmi2`
- `detected_but_unused` — features detected but not exploited
- `gfni` and `vpclmulqdq` detection

Verified end-to-end by starting the server and running
`curl http://127.0.0.1:4601/v1/cpu-runtime`. On the development machine
(Ryzen 9 3950X, Zen 2): `isa_profile: "avx2+fma3"`, `fast_bmi2: false`,
`detected_but_unused: "aes sha"`.

### ⚠️ Honest disclosure: this remains display-only

We searched `server/src` for anything worth SIMD-accelerating and found
**no CPU-bound hot loop**: chat responses are HTTP calls to `aruaru-llm`,
and the learning features are static-data lookups with no heavy text
similarity computation. Rather than inventing a theoretical use, the
response now carries `disclosure_ja` / `disclosure_en` fields stating that
the genuinely accelerated consumers are `open-raid-z` (GF(2^8)),
`open-cuda` / `aruaru-llm` (CPU inference) and `open-cg-cad`
(cross-section derivative).
