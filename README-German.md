# open-english

> 📌 **Neuestes Update (2026-08-18)**: Eine echte lokale Datenbank für
> Chatverlauf/Einstellungen hinzugefügt (SQLite + optionaler
> selbstheilender `aruaru-db`/PostgreSQL-Spiegel), plus APIs für
> Speicherort-Auswahl, rsync-Backup und Import alter Daten. Fehlt
> `rsync`, zeigt die App zweisprachig **"Let's install RSync!"** an und
> kann es automatisch über den Paketmanager des Betriebssystems
> installieren und danach sofort das Backup ausführen. Details siehe
> [CLAUDE.md](CLAUDE.md) (2026-08-18 HANDOFF-Einträge, Japanisch).

> 📌 **Älteres Update (2026-08-11–12, v0.6.0)**: Android/Tablet läuft
> jetzt vollständig eigenständig — kein PC oder Linux-Server mehr
> nötig. Die KI-Antwort-Engine (`aruaru-llm`) selbst ist jetzt in die
> APK gebündelt; die Verifikation auf dem Gerät bestätigte, dass beide
> Prozesse am Leben bleiben und auf `/healthz`/`/v1/chat` antworten.
> Außerdem hinzugefügt: eine Prüfungsvorbereitungs-Ecke (Eiken 1-5,
> TOEIC, TOEFL, JLPT N1-N5, Nihongo Kentei 1-3, je 10 Originalfragen),
> die falsch beantwortete Fragen nach der Bewertung an den KI-Trainer
> weiterreicht (automatischer Wechsel in einen "Japanisch-Klassenzimmer"
> -Modus für JLPT/Nihongo Kentei), eine Auswahl "welche Sprache lernen",
> sowie Linux/macOS-Installer (`installer/unix/install.sh`). Ehrliche
> Offenlegung: Modellgewichte (GPT-2-Familie, Embedding-Modell) sind
> nicht in der APK gebündelt — KI-Chat unter Android erfordert weiterhin
> manuelles Ablegen der Modelldateien im internen Speicher (noch kein
> automatischer Download). Siehe die HANDOFF-Einträge vom 2026-08-11
> (Fortsetzung 7-10) in [CLAUDE.md](CLAUDE.md).

> 📌 **Älteres Update (2026-08-11, Fortsetzung 3)**: Automatische
> Selbstaktualisierung (nur Windows) hinzugefügt, die beim Start die
> neueste GitHub-Version prüft und bei einer neueren Version die alte
> automatisch deinstalliert und die neue installiert. Ehrliche
> Offenlegung: Es existiert noch kein GitHub-Release, daher wurde der
> vollständige Deinstallations-/Installationsablauf noch nicht
> Ende-zu-Ende verifiziert (Versionsvergleichslogik und der Pfad "kein
> Release gefunden, sicher fortfahren" wurden verifiziert).

> 📌 **Älteres Update (2026-08-11, Fortsetzung 2)**: Erkennung für
> Jobsuche-/Karrierewechsel-/Tourismus-Themen hinzugefügt, die
> aruaru.tokyo, audiocafe.tokyo/aruaru, audiocafe.tokyo/aruaru-lady und
> nasa.tokyo auf Englisch und Japanisch vorstellt — funktioniert sowohl
> im normalen Chat als auch im Trainingsmodus, live verifiziert.

> 📌 **Älteres Update (2026-08-11, Fortsetzung)**: Anbindung an eine
> neue Geo-/Tourismus-Datenbank (alle 47 japanischen Präfekturen, 50
> US-Bundesstaaten, wichtige Welthauptstädte mit Sehenswürdigkeiten/
> Essen/Souvenirs), um das Selbstvorstellungs-Training dynamisch zu
> machen. Beim Thema Fuji zeigt die App jetzt einen zweisprachigen
> Sicherheitshinweis (Skibekleidung + Helm tragen, Berghütte im Voraus
> reservieren) sowie echte Hütten-/Bus-/Ausrüstungsladen-Infos und eine
> Tourbuchungssuche. Auswahl-UI für Altersgruppe/Level/Business-Englisch
> hinzugefügt. Live gegen ein laufendes `aruaru-llm` + statischen Server
> verifiziert (3 echte Bugs gefunden und behoben).

> 📌 **Älteres Update (2026-08-11)**: Einstellungsfeld zum Speichern des
> eigenen Google-Search-API-Schlüssels/cx direkt aus dem Browser
> hinzugefügt (`POST /v1/settings/google-search`, nur im Speicher,
> nie auf Festplatte geschrieben). Der Windows-Installer
> (`installer/windows/`, Inno Setup) wurde tatsächlich gebaut,
> installiert, gestartet und auf echter Hardware deinstalliert (keine
> Admin-Rechte nötig).

> 📌 **Älteres Update (2026-08-10, Fortsetzung)**: (1) Standardmodell
> von `gpt2` (124M) auf `distilgpt2` (82M) umgestellt, ~42% schneller.
> (2) Entschieden, die Frontend-JS **nicht** nach Rust/WASM zu
> portieren (kein Performancegewinn, `SpeechRecognition` hat keine
> standardisierte web-sys-Anbindung) — stattdessen wurde **der lokale
> Datei-Server nach Rust portiert** (neue `server/`-Crate, basierend
> auf RPoems `open-runo-poem-compat`, entfernt die Abhängigkeit von
> `python3 -m http.server`). (3) Japanische Eingaben verbessert, sodass
> hybride (Englisch+Japanisch) Antworten immer garantiert sind. (4)
> Versionsverwaltung hinzugefügt (`version.json` mit semantischer
> `version`, in der Fußzeile angezeigt) und automatisches Aufräumen der
> browserseitigen Spuren alter Versionen.

> 📌 **Aktuelles Update (2026-08-10)**: CORS-Unterstützung hinzugefügt,
> die Ursache der entarteten Wiederholungsschleife beim GPT-2-Greedy-
> Decoding behoben (`open-cuda`s `generate_with_repetition_penalty`,
> Standard `penalty=1.3`), das Aussehen der Tora-san-Figur angepasst +
> Umschalt-Jingle + Vorstellungs-Fix hinzugefügt, einen Trainingsschritt
> basierend auf der echten Kundenbetreuungstechnik eines Akihabara-
> Maid-Cafés (@ほぉ～むカフェ) hinzugefügt, einen Schritt über den
> aktuellen Boom der japanischen Kultur im Ausland recherchiert und
> hinzugefügt (Anime/Manga, Anime-Songs, Spiele, Japanisch-Lernende,
> Goshuin-Sammeln, Onsen-Ryokan-Tourismus, japanisches Essen), Launcher-
> Icons für Windows/Mac/Linux/Android/iPhone/iPad hinzugefügt und einen
> Auto-Update-Mechanismus implementiert.

Eine browserbasierte (Phase 0) Web-App zum Englischlernen für
PC/Tablet/Smartphone. Im Stil einer "Maid-Café-Englischstunde"
begleitet eine magische Maid-Figur (Originaldesign, animiert) Lernende
vom absoluten Anfänger bis zum Fortgeschrittenen.

## Architektur (gemäß Nutzeranweisung, 2026-08-10)

- **Linux (VPS)-Seite**: nur ein Download-Verteilserver (nicht der Ort,
  an dem diese App tatsächlich läuft). Die App-Verwaltung übernimmt
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web).
- **Nutzergerät (PC/Tablet/Handy)-Seite**: das statische Web-Frontend
  dieses Repos (HTML/CSS/JS, läuft im Browser) + ein lokal laufender
  nativer Server aus [`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm)
  (der intern die Inferenz-Backends von `open-directx`/`open-cuda`
  nutzt), den der Nutzer selbst herunterlädt und ausführt. Der Browser
  verbindet sich lokal (online oder offline) mit `http://localhost:4600`
  (Standardport von aruaru-llm) — ein "hybrides" Design.

## Aktueller Umfang (Phase 0) — ehrliche Offenlegung

- **Qualität der KI-Antworten**: `aruaru-llm`s `/v1/generate` führt
  autoregressive Textgenerierung mit GPT-2 (124M-1,5B, englischzentriert,
  ohne Dialog-Finetuning) durch. Flüssigkeit oder Level-Angemessenheit
  der Antworten sind nicht garantiert — dies wird auf dem Bildschirm
  offengelegt, nicht beschönigt. Seit 2026-08-10 behebt ein
  Wiederholungs-Penalty (Standard 1,3) den zuvor gemeldeten Bug der
  endlosen Wiederholungsschleife.
- **CORS**: seit 2026-08-10 behoben — der HTTP-Server von `aruaru-llm`
  sendet jetzt `Access-Control-*`-Header, sodass dieses Frontend
  Cross-Origin (oder über `file://`) geöffnet werden kann und trotzdem
  `http://localhost:4600` erreicht.
- **Level-Auswahl**: der Level-Wähler von Anfänger bis Fortgeschritten
  existiert in der UI, aber die tatsächliche Durchsetzung des Levels
  beschränkt sich auf eine kurze Prompt-Anweisung — GPT-2 folgt ihr
  nicht garantiert.
- **Sprache/TTS**: echte Web-Speech-API (SpeechSynthesis für Ausgabe,
  SpeechRecognition für Mikrofoneingabe) ist eingebunden, mit
  charakterspezifischer Tonhöhen-/Geschwindigkeitseinstellung.
- **Trainingsmodus**: ein deterministisches Selbstvorstellungs-Skript
  (nicht KI-generiert), das jetzt auch die wortbasierte Gesprächstechnik
  eines echten Akihabara-Maid-Cafés sowie einen Schritt über den
  aktuellen Boom der japanischen Kultur im Ausland enthält.
- **Launcher-Icons**: `icons/` + `manifest.json` (PWA) + `launchers/`
  ermöglichen das Starten der App über ein Desktop-Icon
  (Windows/Mac/Linux) oder ein Homescreen-Icon (Android/iPhone/iPad).
- **Auto-Update**: `auto-update.js` fragt `version.json` alle 5s ab und
  lädt die Seite bei einer neuen Build-ID neu. **Bekannte Einschränkung**:
  manche Browser blockieren `fetch()` lokaler Dateien unter dem
  `file://`-Schema — garantiert funktionsfähig bei Bereitstellung über
  einen lokalen HTTP-Server, deaktiviert sich sonst stillschweigend.

## Erforderliche Installationsprogramme (hinzugefügt am 2026-08-17)

Um open-english auszuführen, müssen die folgenden zwei Programme
heruntergeladen und installiert werden (kein Bauen aus dem Quellcode
nötig, nahezu ein Ein-Klick-Verfahren).

| # | Was | Windows | Linux | Android/Tablet |
|---|---|---|---|---|
| 1 | **open-english selbst** (dieses Repository — statisches Frontend + Auslieferungsserver) | [open-english-install.exe](https://github.com/aon-co-jp/open-english/releases/latest/download/open-english-install.exe) | [tar.gz](https://github.com/aon-co-jp/open-english/releases/latest) | [APK](https://github.com/aon-co-jp/open-english/releases/latest) (die `.apk`-Datei aus der Asset-Liste wählen) |
| 2 | **aruaru-llm** (die KI-Antwort-Engine — erforderlich, ohne sie funktioniert der Chat nicht) | [aruaru-llm-windows-x86_64.zip](https://github.com/aon-co-jp/aruaru-llm/releases/latest/download/aruaru-llm-windows-x86_64.zip) | [tar.gz](https://github.com/aon-co-jp/aruaru-llm/releases/latest) | Bereits enthalten (in der APK von open-english eingebettet, keine separate Installation nötig) |

**Ehrliche Offenlegung**: Die "latest"-Links oben verweisen immer auf das
neueste GitHub-Release (nutze die
[Releases-Seite](https://github.com/aon-co-jp/open-english/releases)
direkt, wenn du eine bestimmte, fest angegebene Version möchtest). Für
`aruaru-llm` gibt es derzeit noch keine vorgefertigte macOS-Binärdatei
(open-english selbst bietet ein macOS-tar.gz, aber `aruaru-llm` nur
Linux/Windows) — unter macOS muss `aruaru-llm` aus dem Quellcode gebaut
werden.

Unter Windows/Linux/macOS prüft die eingebaute Auto-Update-Funktion
(`server/src/self_update.rs`, seit 2026-08-19 auf Linux und macOS
erweitert) nach der Installation beim Start die GitHub Releases und
aktualisiert automatisch, falls eine neuere Version verfügbar ist
(Windows: Deinstallation→Neuinstallation; Linux/macOS: die laufende
Binärdatei ersetzt sich selbst) — ohne Zutun des Benutzers. Vor jedem
Update wird die aktuelle Binärdatei gesichert; erreicht die neue
Version den neuen `/healthz`-Endpunkt nicht innerhalb einer kurzen
Frist, wird automatisch auf die gesicherte Version zurückgerollt.
**Ehrlicher Hinweis**: Android/iPhone/iPad sind von diesem Auto-Update/
Rollback-Mechanismus ausgenommen (das Betriebssystem erlaubt keine
stille APK-Installation) — dort bleibt die manuelle Installation durch
den Nutzer nötig, ein Rollback-Pfad existiert dort nicht.

Außerdem wurde `facebook.html` als neue Einstiegsseite hinzugefügt, für
Nutzer, deren Mobilfunktarif nur Facebook-Zugriff erlaubt — ehrlicher
Hinweis: es handelt sich nicht um eine offizielle "Free Basics"-
Partnerschaft mit Meta, sondern lediglich um eine normale Seite, die
über den in Facebook integrierten Browser erreichbar ist und auf die
bestehenden Installationsprogramme verweist.

*(Hinweis zur maschinellen Übersetzung: Dieser Absatz wurde vom
KI-Agenten selbst übersetzt, ohne Prüfung durch einen Muttersprachler.)*

## Ausführung

1. [`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm) mit
   `cargo run --release` starten (Standard `http://localhost:4600`,
   Standardmodell jetzt `distilgpt2`).
2. Im Verzeichnis `server/` `cargo run --release` ausführen, um das
   statische Frontend dieses Repos unter `http://127.0.0.1:4601/`
   bereitzustellen (RPoem-basiert — `python3 -m http.server` wird nicht
   mehr benötigt; Port über die Umgebungsvariable
   `OPEN_ENGLISH_SERVER_BIND` änderbar).
3. `http://127.0.0.1:4601/` im Browser öffnen. Direktes Öffnen von
   `index.html` über `file://` funktioniert weiterhin, aber manche
   Browser blockieren dabei `fetch()` und deaktivieren Auto-Update —
   der Server aus Schritt 2 wird empfohlen.

## Nächste Schritte

1. ~~CORS-Unterstützung auf `aruaru-llm`-Seite~~ **Erledigt (2026-08-10)**.
2. ~~GPT-2-Wiederholungsschleife~~ **Ursache behoben (2026-08-10,
   Wiederholungs-Penalty)**.
3. ~~Standardmodell beschleunigen~~ **Erledigt (2026-08-10, Wechsel zu
   distilgpt2, ~42% schneller)**.
4. ~~Hybride Antworten bei japanischer Eingabe garantieren~~ **Erledigt
   (2026-08-10)**.
5. ~~Lokalen Datei-Server nach Rust portieren~~ **Erledigt (2026-08-10,
   `server/`-Crate)**. Die Portierung des Frontend-JS selbst nach
   Rust/WASM wurde geprüft und verworfen (kein Performancegewinn —
   siehe `CLAUDE.md`).
6. TTS/Lippensynchronisation verfeinern.
7. Ein Curriculum pro Level (Grammatik, Vokabellisten usw.)
   implementieren.
8. **(gemäß Nutzeranweisung, 2026-08-10)** Zukünftige Idee:
   `open-directx`/`open-cuda`/`aruaru-llm` im Browser (WASM/WebGPU)
   laufen lassen und mit `RPoem` (einer GraphQL-Federation-Plattform)
   integrieren. Dies ist eine große, separate architektonische
   Richtung gegenüber dem aktuellen Phase-0-Design und wird auf nach
   dem MVP verschoben.
9. Untersuchen, ob Techniken aus Toshiba SBM oder der DeepSeek-Familie
   hier echten Nutzen hätten (noch nicht begonnen).

---

Weitere Sprachen: [日本語](README.md) · [English](README-English.md) ·
[Italiano](README-Italian.md) · [Français](README-French.md) ·
[Русский](README-Russian.md) · [Українська](README-Ukrainian.md) ·
[עברית](README-Hebrew.md) · [فارسی](README-Persian.md)
