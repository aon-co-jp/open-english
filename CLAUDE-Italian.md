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

## Altri aggiornamenti (2026-08-19 - 2026-08-20)

Da allora sono stati aggiunti, tra l'altro: un avviso al
raggiungimento del limite di utilizzo giornaliero, con indicazioni sui
piani a pagamento e sui livelli gratuiti di altri provider; un nuovo
pannello "Data & Model Storage" con interfaccia di backup/ripristino;
un banner per mobilitare gli smartphone inutilizzati
(`PhoneAccelWorker` con rilevamento NNAPI); un backup rsync combinato
per aruaru-db/PostgreSQL; una pagina roadmap segnaposto per i port
console (PlayStation/Switch/Wii/Wii U, in attesa dell'autorizzazione
dei produttori); gli aggiornamenti automatici estesi a tutti i
componenti inclusi (aruaru-llm/aruaru-db) con controllo periodico
ogni 6 ore e downgrade manuale per singolo componente; una correzione
che avvia automaticamente aruaru-llm su Windows (nessun passaggio
manuale residuo); un banner informativo sui livelli gratuiti dei
provider IA/ricerca (incluso Claude/Anthropic); una correzione
critica che previene la perdita del database delle conversazioni
durante l'aggiornamento/downgrade automatico su Windows (verificata
end-to-end tramite un installer reale); e nuove funzionalità Android
(rilevamento automatico dell'URL del PC tramite scansione di
sottorete, verificato su un dispositivo Snapdragon reale con
rilevamento NNAPI riuscito, mentre l'effettivo offload di calcolo
NNAPI non è ancora implementato). Dettagli nelle voci HANDOFF del
2026-08-19 e del 2026-08-20 in [CLAUDE.md](CLAUDE.md).

## Aggiornamenti recenti relativi all'installazione (2026-08-19)

Installer Windows unificato `open-english-install.exe`; l'aggiornamento
automatico integrato (`server/src/self_update.rs`) è stato esteso a
Linux e poi a macOS, con rollback automatico basato su un controllo di
salute `/healthz` se la nuova versione non si avvia correttamente;
nuova pagina di ingresso `facebook.html` per utenti con accesso solo a
Facebook (divulgazione onesta: non è una partnership ufficiale con
Meta); e dal 2026-08-18 l'app promuove e automatizza l'installazione di
`rsync` quando manca, avviando subito dopo il backup. Dettagli completi
solo nelle voci HANDOFF in giapponese qui sotto.

*(Nota sulla traduzione automatica: questo riepilogo è stato tradotto
dall'agente IA stesso, senza revisione da parte di un madrelingua.)*

---

Altre lingue: [日本語 (originale, con la cronologia completa degli HANDOFF)](CLAUDE.md) ·
[Deutsch](CLAUDE-German.md) · [Français](CLAUDE-French.md) ·
[Русский](CLAUDE-Russian.md) · [Українська](CLAUDE-Ukrainian.md) ·
[עברית](CLAUDE-Hebrew.md) · [فارسی](CLAUDE-Persian.md)
