# Entwicklungsphilosophie & Umgebungsregeln (open-english)

> **Hinweis**: Dies ist eine kondensierte Übersetzung des aktuellen
> Zustands. Das ausführliche historische HANDOFF-Änderungsprotokoll
> (Dutzende von Einträgen seit 2026-08-10) bleibt aus Gründen der
> Kürze nur auf Japanisch in [CLAUDE.md](CLAUDE.md) verfügbar — siehe
> dort für Details zu einzelnen Sitzungen.

Arbeitslaufwerk: `F:\runo`. Dieser Abschnitt folgt der Praxis, den
entsprechenden Abschnitt aus dem `CLAUDE.md` von
[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) als
Referenz zu übernehmen und in jedes Projekt zu kopieren. GitHub-Repo:
[aon-co-jp/open-english](https://github.com/aon-co-jp/open-english).

**Entwicklungsbeginn: 2026-08-10.**

## Rolle dieses Projekts

Eine Web-App zum Englischlernen für PC/Tablet/Smartphone. Im Stil
einer "Maid-Café-Englischstunde" begleitet eine magische Maid-Figur
(Originaldesign, animiert) Lernende vom Anfänger bis zum
Fortgeschrittenen. Die KI-Antworten übernimmt
[`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm).

## Architektur (gemäß Nutzeranweisung, 2026-08-10)

- **Linux (VPS)-Seite**: nur ein Download-Verteilserver. Die
  App-Verwaltung übernimmt
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web).
- **Nutzergerät-Seite**: das statische Web-Frontend dieses Repos +
  ein lokal laufender nativer Server aus `aruaru-llm` (nutzt intern
  die Inferenz-Backends von `open-directx`/`open-cuda`), den der
  Nutzer selbst herunterlädt und ausführt. Der Browser verbindet
  sich lokal (online/offline) mit `http://localhost:4600`.

## Ehrliche Offenlegung / bekannte Einschränkungen (Stand 2026-08-10)

- `aruaru-llm`s `/v1/generate` ist GPT-2 (englischzentriert, kein
  Dialog-Finetuning) — Antwortqualität und Level-Einhaltung sind
  nicht garantiert.
- CORS wurde am 2026-08-10 behoben (`.with_cors()` bei `aruaru-llm`).
- Sprachsynthese (TTS)/Lippensynchronisation für die Trainer-
  Charaktere ist per Web Speech API angebunden; die Animation der
  Maid-Figur selbst ist weiterhin eine einfache CSS-Mundöffnungs-
  schleife.

## Zukunftsvision (gemäß Nutzeranweisung, 2026-08-10, noch nicht begonnen)

Es gibt die Idee, `open-directx`/`open-cuda`/`aruaru-llm` auch
eigenständig im Browser (WASM/WebGPU) laufen zu lassen und mit
`RPoem` (einer GraphQL-Federation-Plattform) zu integrieren. Da dies
eine große architektonische Änderung gegenüber dem aktuellen
Phase-0-Design (lokaler residenter Server + localhost-Verbindung)
bedeutet, wird dies erst nach Abschluss des MVP und mit eigenem
Scope in Angriff genommen.

## Weitere Updates (2026-08-19 bis 2026-08-20)

Seither kamen u. a. hinzu: ein Hinweis bei Erreichen des täglichen
Nutzungslimits mit Hinweisen auf kostenpflichtige Pläne und die
kostenlosen Kontingente anderer Anbieter; ein neues Panel „Data &
Model Storage" mit Backup-/Wiederherstellungs-UI; ein Banner zur
Mobilisierung ungenutzter Smartphones (`PhoneAccelWorker` mit
NNAPI-Erkennung); eine kombinierte rsync-Sicherung für aruaru-db/
PostgreSQL; eine Roadmap-Platzhalterseite für Konsolen-Ports
(PlayStation/Switch/Wii/Wii U, abhängig von der Zustimmung der
Plattformhalter); automatische Updates, erweitert auf alle
mitgelieferten Komponenten (aruaru-llm/aruaru-db) mit
6-Stunden-Intervall-Prüfung und manuellem Downgrade pro Komponente;
ein Fix, der aruaru-llm unter Windows automatisch startet (kein
manueller Schritt mehr nötig); ein Info-Banner zu kostenlosen
Kontingenten von KI-/Suchanbietern (inkl. Claude/Anthropic); ein
kritischer Fix, der den Verlust der Gesprächsdatenbank bei
Windows-Auto-Update/Downgrade verhindert (End-to-End über einen
echten Installer verifiziert); sowie neue Android-Funktionen
(Subnetz-Scan zur automatischen PC-URL-Erkennung, auf einem echten
Snapdragon-Gerät mit erfolgreicher NNAPI-Erkennung verifiziert,
wobei die tatsächliche NNAPI-Rechenauslagerung noch nicht
implementiert ist). Details in den HANDOFF-Einträgen vom 2026-08-19
und 2026-08-20 in [CLAUDE.md](CLAUDE.md).

## Neueste installationsbezogene Updates (2026-08-19)

Vereinheitlichter Windows-Installer `open-english-install.exe`; das
eingebaute Auto-Update (`server/src/self_update.rs`) wurde auf Linux
und dann macOS erweitert, mit automatischem Rollback basierend auf
einem `/healthz`-Health-Check, falls die neue Version nicht korrekt
startet; neue Einstiegsseite `facebook.html` für Nutzer mit reinem
Facebook-Zugang (ehrliche Offenlegung: keine offizielle
Meta-Partnerschaft); und seit 2026-08-18 fördert und automatisiert die
App die Installation von `rsync`, falls es fehlt, und startet
anschließend direkt das Backup. Vollständige Details nur in den
japanischen HANDOFF-Einträgen unten.

*(Hinweis zur maschinellen Übersetzung: Diese Zusammenfassung wurde vom
KI-Agenten selbst übersetzt, ohne Korrekturlesen durch einen
Muttersprachler.)*

---

Weitere Sprachen: [日本語 (Original, mit vollständigem HANDOFF-Verlauf)](CLAUDE.md) ·
[Italiano](CLAUDE-Italian.md) · [Français](CLAUDE-French.md) ·
[Русский](CLAUDE-Russian.md) · [Українська](CLAUDE-Ukrainian.md) ·
[עברית](CLAUDE-Hebrew.md) · [فارسی](CLAUDE-Persian.md)

## Update 2026-08-22: Weltsprachen-Übungsprüfungen, Sprachauswahl, mehrsprachige Wiedergabe

Vor der Umsetzung wurde der Code geprüft: **es existierte bisher keine
Liste unterstützter Übersetzungssprachen und keine i18n-Tabelle**
(`learn-target` kannte nur Englisch/Japanisch). Die Sprachliste wurde
daher mit dieser Funktion neu definiert — **38 Sprachen** (17 europäische
inkl. Rätoromanisch/Schweiz, Russisch, 4 nahöstliche, 7 südasiatische,
8 ost-/südostasiatische, 1 afrikanische) plus Englisch und Japanisch als
stets aktive Standardsprachen.

- Neue Daten: `world-language-exams.json` (Original-Multiple-Choice-Aufgaben
  mit CEFR-artigen Stufen, 3–6 pro Sprache) und `world-language-phrases.json`
  (5 Basissätze × 40 Sprachen).
- Neue API: `GET /v1/world-languages` liefert nur die Übersicht (ohne
  Aufgabentexte). Für die DB-Speicherung wurde **kein** neuer Endpunkt
  angelegt — der bestehende `POST /v1/db/history` genügt.
- UI: zweisprachiges Banner, Panel "🌐 Languages" mit Checkboxen,
  "Alle auswählen" und "Alle außer EN & JA abwählen", Auswahl von
  **2–5 Sprachen** (Grenze in der UI erzwungen), sequentielle Anzeige +
  Vorlesen mit beliebiger Wiederholung (gesamt oder pro Sprache),
  Kopieren, .txt-Download und Speichern in SQLite.
- Nach der Auswertung führen die falschen Aufgaben in das Gespräch mit
  dem Tutor der jeweiligen Sprache (bestehende `examPrepMissedQuestions`-
  Logik wiederverwendet).

**Ehrliche Offenlegung**: Originalaufgaben, keine echten Prüfungsfragen
und ohne Bezug zu offiziellen Zertifikaten; nur 3–6 Aufgaben pro Sprache;
keine muttersprachliche Prüfung der Texte; das tatsächliche Vorlesen in
allen 38 Sprachen wurde mangels installierter Stimmen nicht real
verifiziert (nur mit einem Beobachtungs-Stub). Details siehe HANDOFF vom
2026-08-22 in [CLAUDE.md](CLAUDE.md).

## Regelbasierte Festtext-Antworten zu sensiblen Themen (2026-08-23)

Bei Themen, bei denen falsche Angaben besonders schaden, antwortet die App
**ohne KI-Inferenz** mit von Hand geschriebenem Text. Drei Fälle folgen
demselben Muster: `isCreatorQuestion()` (Vorstellung des Autors),
`isReligionHistoryQuestion()` / `RELIGION_HISTORY_TEXTS` (Islam, Iran,
arabische Welt) und `isMarkOfBeastQuestion()` / `MARK_OF_BEAST_TEXTS`
(666 und das Zeichen des Tieres). Grund: ein blankes GPT-2 erfindet
Inhalte, und gerade bei Religionsgeschichte ist der Schaden hoch. Das
Tageskontingent wird nicht verbraucht; die Texte liegen in einer Tabelle
mit Sprachcode als Schlüssel (derzeit nur `ja` und `en`).

**Neutrale Religionsgeschichte**: beschrieben werden die religiöse Vielfalt
des vorislamischen Arabiens, die Entstehung des Korans als in der Forschung
**eigenständige Überlieferung** (nach islamischer Lehre Offenbarung an
Mohammed), der Unterschied zwischen iranischer und arabischer Kultur sowie
der zoroastrische Einfluss — Letzteres ausdrücklich nur als These einiger
Forschender. Zwei ursprünglich gewünschte Behauptungen (Koran aus einer
Bibelübersetzung; ein Bruder Mohammeds als Übersetzer) wurden **mangels
Quellenbelegen nach mehrfacher Rücksprache weggelassen**; der Text legt
offen, *warum* sie nicht behauptet werden. Eine Anmerkung zu den Grenzen
vormoderner Übersetzungsarbeit bezieht sich **ausschließlich** auf die
historisch belegten arabischen Bibelübersetzungen und darf **nie** mit der
Entstehung des Korans verknüpft werden. Am Ende steht die Botschaft, dass
Übersetzung und mehrsprachiger Austausch zu Verständigung und Frieden
beitragen können.

**666 / Zeichen des Tieres**: die Offenbarungsstelle wird neutral genannt,
ohne eine "richtige" Deutung zu behaupten. Das Wortspiel **"666 = WWW"**
(Gematrie, Waw = 6) wird **nur als Lesart einiger Leute seit den 1990ern**
vorgestellt, nicht als Lehre. Die Geschichte vom versteckten 666 im Barcode
wird **als Großstadtlegende gekennzeichnet** und technisch aufgelöst:
**Guard Bars** markieren Start, Ende und Mitte für den Scanner, ähneln der
Ziffer 6 nur optisch und sind eine andere Kodierung (3 statt 7 Module);
Snopes bewertet die Behauptung als FALSCH — **keine okkulte Bedeutung, keine
technische Grundlage**. Der Schluss ist positiv (Web und Scanner machen das
Einkaufen bequem, **ohne Zeichen am Körper**), und die Python-Fußnote
(Schlangenlogo, Name von Monty Python) wird **ausdrücklich als reiner
Zufall und Wortspiel** gekennzeichnet.

**Ergänzung 2026-08-23 (interessante Parallele — vorstellen, nicht
behaupten)**: Offenbarung 13,16-17 enthält tatsächlich eine Stelle, wonach
niemand ohne das Zeichen kaufen oder verkaufen kann (die Existenz dieser
Bibelstelle darf als Tatsache genannt werden). Hinzugefügt wurde ein Satz,
dass **manche Leute darin eine interessante Parallele sehen** zu der
Tatsache, dass modernes Einkaufen immer stärker auf Barcodes und
Online-Bezahldienste wie Amazon angewiesen ist. **Zwingende Einschränkung**:
nur als Möglichkeit vorstellen ("manche sagen das"), **niemals** behaupten,
eine Prophezeiung habe sich erfüllt — der Text sagt das ausdrücklich selbst.

**Beim Ändern beachten**: Die vier Ehrlichkeitsgarantien — "Vorstellung
statt Behauptung", "Legende ausdrücklich als Legende", "Zufall ausdrücklich
als Zufall" — dürfen weder abgeschwächt noch weggelassen werden.

## Nur Formulierung: leichterer Ton bei "666" (2026-08-23)

Drei Stellen von `MARK_OF_BEAST_TEXTS` wurden **rein sprachlich**
überarbeitet: das Wortspiel "666 = WWW", die Barcode-Legende und der
Nebensatz zu Offenbarung 13,16-17 (niemand kann ohne das Zeichen kaufen
oder verkaufen) neben dem heutigen Einkaufen per Barcode und Amazon. Die
steifen Distanzierungsformeln wurden durch einen wärmeren Ton ersetzt.

- **Vorher**: "… keinesfalls als Behauptung, eine Prophezeiung habe sich
  erfüllt" / JA: 「〜と断定するものではありません」
- **Nachher**: "… eher als nette Trivia denn als harter Beweis" / JA:
  「話のタネとして」「真偽のほどは分かりませんが、こういう見方をすると
  聖書の世界も少し身近に感じられるかもしれません」

**Die Auflagen selbst sind unverändert** und weiterhin einzuhalten:
(1) nichts wird als erfüllte Prophezeiung behauptet; (2) das Barcode-666
bleibt ausdrücklich eine **technisch haltlose Großstadtlegende** (Snopes:
FALSCH; Guard Bars 3 Module statt 7 pro Ziffer); (3) die Ähnlichkeit von
Python-Schlange und "Tier" bleibt **reiner Zufall**; (4) Deutungen werden
nur vorgestellt, nicht gelehrt. Geändert wurde allein der Stil zugunsten
von Lesbarkeit — **keine Aufweichung der Ehrlichkeit**.

## Quizfunktion: Original-Rätsel des Autors (2026-08-23, Fortsetzung 4)

Auf Anfragen wie "Stell mir eine Aufgabe" / "give me a quiz" antwortet die
App mit einem Original-Rätsel des Autors **Masahiro Ishizuka (石塚正浩)**:
`9 ◯ 9 ◯ 9 ◯ 9 = 10` — in die Kreise kommt je eines der Zeichen `+`, `-`,
`×`, `÷` (Wiederholung erlaubt), Klammern ( ) dürfen die Reihenfolge
ändern; Lösung `(9 × 9 + 9) ÷ 9 = 10`. Im Text ist **ausdrücklich**
festzuhalten, dass es sich **nicht um ein Scherz- oder Fangrätsel**
handelt, sondern um **reine Grundrechenarithmetik**, nachprüfbar mit
Taschenrechner oder Abakus. Dazu die Episode, dass die bisher jüngste
richtige Lösung von einem Kind der **ersten Grundschulklasse** kam.

Bausteine (in `app.js`): `isQuizRequest()`, `isQuizAnswerRequest()`,
`QUIZ_TEXTS`, `quizQuestionText()`, `quizAnswerText()`,
`quizPreferredLangCode()` sowie die Zustandsvariable `quizAwaitingAnswer`.
Der Ablauf ist **zweistufig**: zuerst nur die Aufgabe, dann — nach "Ich
weiß es nicht" / "Sag mir die Lösung" — die Antwort; der Flag merkt sich
den Zwischenzustand.

**Warum kein KI-Pfad**: Wie bei den übrigen Festtext-Antworten (Autor,
Religionsgeschichte, 666) läuft dies **ohne KI-Inferenz** über
regelbasierte Verzweigungen. Ein blankes GPT-2 erzeugt beim Rechnen
überzeugend formulierte, aber **falsche** Ergebnisse — bei einer Aufgabe
mit genau einer nachprüfbaren Lösung ist das inakzeptabel. Das
Tageskontingent wird nicht verbraucht.

**Mehrsprachigkeit und ihre bewusste Grenze**: Standard ist die
zweisprachige Ausgabe JA + EN; bei Lern- oder Muttersprache es/fr/de/zh/ko
wird die jeweilige Übersetzung vorangestellt. Übersetzungen existieren nur
für diese **sieben Sprachen** (ja/en/es/fr/de/zh/ko). Die 130 unterstützten
Sprachen wurden **absichtlich nicht** maschinell aufgefüllt, um "alle
Sprachen unterstützt" vorzutäuschen; nicht erfasste Sprachen erhalten die
japanisch-englische Standardausgabe. Diese Grenze ist beim Ändern
beizubehalten und offenzulegen.
