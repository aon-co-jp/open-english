# PORTING.md — Guida al porting di open-english (versione condensata)

> **Nota**: questa è una traduzione condensata. La guida tecnica
> completa con dettagli di codice e insidie resta disponibile solo in
> giapponese in [PORTING.md](PORTING.md) — consultarla prima di
> adottare effettivamente un pattern.

Riepilogo dei pattern di implementazione riutilizzabili di questo
progetto, nel caso vengano portati in un altro progetto:

1. **Pattern di integrazione con `aruaru-llm`**: richiede supporto
   CORS (`.with_cors()`) e una penalità di ripetizione
   (`generate_with_repetition_penalty`, predefinita 1,3) lato
   `aruaru-llm`, altrimenti il browser blocca `fetch()` oppure il
   greedy decoding di GPT-2 cade in un loop di ripetizione.
2. **Estrazione della lingua per il TTS** (`extractSpeechText`): separa
   le righe bilingue nel formato "English / 日本語" prima di passarle a
   `SpeechSynthesisUtterance`, per evitare una pronuncia a scatti.
3. **Set di icone di avvio** (`icons/` + `manifest.json` +
   `launchers/`): encoder PNG/ICO scritto a mano senza strumenti
   grafici esterni, più script per `.lnk` Windows, `.desktop` Linux e
   `.app` macOS.
4. **Auto-aggiornamento** (`auto-update.js` + `version.json`): semplice
   polling del `buildId` ogni 5s con `location.reload()`.
5. **Server statico basato su RPoem** (`server/`): sostituisce
   `python3 -m http.server` con un crate Rust che riusa lo
   `static_file_handler` di `open-runo-poem-compat`.
6. **Garanzia strutturale delle risposte ibride** (`ensureHybridReply`):
   aggiunge automaticamente una breve nota in giapponese se la risposta
   del modello non contiene giapponese — senza esagerare la qualità
   della traduzione.
7. **Gestione delle versioni + pulizia lato browser delle versioni
   vecchie**: `version.json` con campo `version`, cancellazione delle
   chiavi `localStorage` proprie dell'app alla nuova versione.
8. **Bridge Google Custom Search JSON API + pannello impostazioni nel
   browser**: credenziali solo in memoria di processo, mai su disco.
9. **Installer Windows (Inno Setup) — passi di verifica reale**:
   `PrivilegesRequired=lowest` contro i blocchi UAC,
   `MSYS_NO_PATHCONV=1` per installazioni silenziose da Git Bash.
10. **DB geo/turistico con avvisi di sicurezza guidati dal tema**:
    ricerca per sottostringa invece di corrispondenza esatta; per
    argomenti pericolosi (es. alpinismo) allegare sempre un avviso di
    sicurezza.
11. **Auto-aggiornamento tramite GitHub Releases (solo Windows)**: deve
    essere implementato lato nativo (non nel JS del browser); il
    proprio .exe in esecuzione non può essere eliminato su Windows —
    avviare prima uno script batch distaccato, poi terminare il
    proprio processo.
12. **Le classi CSS dei modali richiedono `max-height`/`overflow-y`**:
    ogni classe contenitore di modale deve definire la scorribilità,
    altrimenti i contenuti lunghi non sono raggiungibili su mobile.
13. **Inclusione di un server Rust su Android**: rendere i percorsi
    sovrascrivibili a runtime tramite variabile d'ambiente; proteggere
    sempre la logica di auto-aggiornamento specifica di Windows con
    `cfg!(target_os = "windows")`.

**Avvertenza importante**: questo progetto è un prototipo di Fase 0 —
qualità delle risposte AI (basata su GPT-2), naturalezza della sintesi
vocale e adattamento affidabile al livello non sono garantiti. Questo
va reso esplicito anche in ogni porting.

---

Altre lingue: [日本語 (originale, dettagli completi)](PORTING.md) ·
[Deutsch](PORTING-German.md) · [Français](PORTING-French.md) ·
[Русский](PORTING-Russian.md) · [Українська](PORTING-Ukrainian.md) ·
[עברית](PORTING-Hebrew.md) · [فارسی](PORTING-Persian.md)
