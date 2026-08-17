# Filosofia di sviluppo e regole d'ambiente (open-english)

> **Nota**: questa è una traduzione condensata dello stato attuale. Il
> log storico dettagliato degli HANDOFF (decine di voci dal
> 2026-08-10) resta disponibile solo in giapponese in
> [CLAUDE.md](CLAUDE.md) per brevità — consultarlo per i dettagli
> delle singole sessioni.

Unità di lavoro: `F:\runo`. Questa sezione segue la pratica di copiare
la sezione corrispondente del `CLAUDE.md` di
[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) in ogni
progetto come riferimento. Repository GitHub:
[aon-co-jp/open-english](https://github.com/aon-co-jp/open-english).

**Inizio sviluppo: 2026-08-10.**

## Ruolo di questo progetto

Un'app web per l'apprendimento dell'inglese per PC/tablet/smartphone.
In stile "lezione di inglese in un maid café", un personaggio maid da
maga (design originale, animato) accompagna gli studenti dal
principiante all'avanzato. Le risposte AI sono gestite da
[`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm).

## Architettura (su indicazione dell'utente, 2026-08-10)

- **Lato Linux (VPS)**: solo un server di distribuzione per il
  download. La gestione dell'app è affidata a
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web).
- **Lato dispositivo dell'utente**: il frontend web statico di questo
  repo + un server nativo eseguito localmente da `aruaru-llm` (usa
  internamente i backend di inferenza di `open-directx`/`open-cuda`),
  scaricato ed eseguito dall'utente stesso. Il browser si connette
  localmente (online/offline) a `http://localhost:4600`.

## Divulgazione onesta / limitazioni note (al 2026-08-10)

- `/v1/generate` di `aruaru-llm` è GPT-2 (centrato sull'inglese, senza
  fine-tuning per il dialogo) — qualità delle risposte e rispetto del
  livello non sono garantiti.
- Il CORS è stato risolto il 2026-08-10 (`.with_cors()` lato
  `aruaru-llm`).
- La sintesi vocale (TTS)/lip-sync per i personaggi tutor è collegata
  tramite Web Speech API; l'animazione del personaggio maid stesso
  resta un semplice loop CSS di apertura/chiusura della bocca.

## Visione futura (su indicazione dell'utente, 2026-08-10, non ancora iniziata)

C'è l'idea di far girare `open-directx`/`open-cuda`/`aruaru-llm`
anche in modo autonomo nel browser (WASM/WebGPU) e integrarli con
`RPoem` (una piattaforma GraphQL Federation). Trattandosi di un grande
cambiamento architetturale rispetto all'attuale design di Fase 0
(server residente locale + connessione localhost), verrà affrontato
solo dopo il completamento dell'MVP, con un ambito dedicato.

---

Altre lingue: [日本語 (originale, con la cronologia completa degli HANDOFF)](CLAUDE.md) ·
[Deutsch](CLAUDE-German.md) · [Français](CLAUDE-French.md) ·
[Русский](CLAUDE-Russian.md) · [Українська](CLAUDE-Ukrainian.md) ·
[עברית](CLAUDE-Hebrew.md) · [فارسی](CLAUDE-Persian.md)
