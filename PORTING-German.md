# PORTING.md — Leitfaden zur Portierung von open-english (Kurzfassung)

> **Hinweis**: Dies ist eine kondensierte Übersetzung. Die vollständige
> technische Anleitung mit Code-Details und Fallstricken bleibt nur auf
> Japanisch in [PORTING.md](PORTING.md) verfügbar — dort nachschlagen,
> bevor ein Muster tatsächlich übernommen wird.

Zusammenfassung der wiederverwendbaren Implementierungsmuster aus
diesem Projekt, falls sie in ein anderes Projekt portiert werden:

1. **`aruaru-llm`-Integrationsmuster**: erfordert CORS-Unterstützung
   (`.with_cors()`) und einen Wiederholungs-Penalty
   (`generate_with_repetition_penalty`, Standard 1,3) auf der
   `aruaru-llm`-Seite, sonst blockiert der Browser `fetch()` bzw. das
   GPT-2-Greedy-Decoding gerät in eine Wiederholungsschleife.
2. **Sprachextraktion für TTS** (`extractSpeechText`): trennt
   zweisprachige Zeilen im Format "Englisch / 日本語" vor der Übergabe
   an `SpeechSynthesisUtterance`, um abgehackte Aussprache zu
   vermeiden.
3. **Launcher-Icon-Set** (`icons/` + `manifest.json` + `launchers/`):
   handgeschriebener PNG/ICO-Encoder ohne externe Bildwerkzeuge, plus
   Skripte für Windows-`.lnk`, Linux-`.desktop` und macOS-`.app`.
4. **Auto-Update** (`auto-update.js` + `version.json`): einfaches
   Polling der `buildId` alle 5s mit `location.reload()`.
5. **RPoem-basierter statischer Server** (`server/`): ersetzt
   `python3 -m http.server` durch eine Rust-Crate, die
   `open-runo-poem-compat`s `static_file_handler` wiederverwendet.
6. **Strukturelle Garantie hybrider Antworten** (`ensureHybridReply`):
   ergänzt automatisch eine kurze japanische Notiz, wenn die
   Modellantwort kein Japanisch enthält — ohne die Übersetzungsqualität
   zu beschönigen.
7. **Versionsverwaltung + browserseitige Bereinigung alter Versionen**:
   `version.json` mit `version`-Feld, Löschen von app-eigenen
   `localStorage`-Schlüsseln bei neuer Version.
8. **Google Custom Search JSON API-Bridge + Browser-Einstellungspanel**:
   Zugangsdaten nur im Prozessspeicher, nie auf Festplatte.
9. **Windows-Installer (Inno Setup) — reale Verifikationsschritte**:
   `PrivilegesRequired=lowest` gegen UAC-Hänger,
   `MSYS_NO_PATHCONV=1` für Git-Bash-Silent-Installs.
10. **Geo-/Tourismus-DB mit themengesteuerten Sicherheitshinweisen**:
    Teilstring-Suche statt exakter Übereinstimmung; bei gefährlichen
    Themen (z. B. Bergsteigen) immer einen Sicherheitshinweis
    beifügen.
11. **Automatisches Update via GitHub Releases (Windows/Linux/macOS,
    siehe Punkt 14 für Rollback)**: muss
    zwingend nativ (nicht im Browser-JS) implementiert werden; die
    eigene laufende .exe kann unter Windows nicht gelöscht werden —
    erst eine losgelöste Batch-Datei starten, dann den eigenen
    Prozess beenden.
12. **Modale CSS-Klassen brauchen `max-height`/`overflow-y`**: jede
    Modal-Container-Klasse muss Scroll-Fähigkeit definieren, sonst
    sind lange Inhalte auf Mobilgeräten nicht erreichbar.
13. **Android-Bündelung eines Rust-Servers**: Pfade zur Laufzeit per
    Umgebungsvariable überschreibbar machen; Windows-spezifische
    Auto-Update-Logik unbedingt mit `cfg!(target_os = "windows")`
    absichern.

14. **Plattformübergreifendes Auto-Update mit Health-Check-gestütztem
    Rollback**: `apply_update_linux` zu `apply_update_unix`
    verallgemeinern (Linux und macOS teilen sich das Ersetzen der
    laufenden Binärdatei); die aktuelle Binärdatei vor dem Update
    sichern; nach dem Start der neuen Version einen neuen
    `/healthz`-Endpunkt innerhalb kurzer Frist prüfen und bei
    Fehlschlag die Sicherung wiederherstellen. Android/iOS bleiben
    ausdrücklich außerhalb des Umfangs (Plattformbeschränkung, kein
    Versäumnis).
15. **Werbung + automatische Installation von RSync**: fehlt `rsync`,
    eine einladende zweisprachige Meldung ("Let's install RSync!")
    statt eines trockenen Fehlers anzeigen, dann eine automatische
    Installation über eine Paketmanager-Kette je nach OS versuchen
    (winget→choco unter Windows, apt-get→dnf→pacman unter Linux, brew
    unter macOS, pkg unter Termux/Android) und bei Erfolg direkt das
    Backup anschließen.
16. **Dedizierte Einstiegsseite für einen eingeschränkten Zugangskanal**
    (z. B. `facebook.html` für Nutzer mit reinem Facebook-Zugang): eine
    einfache statische Seite, die die bereits im README vorhandenen
    Download-Links wiederverwendet, mit expliziter ehrlicher
    Offenlegung, dass kein offizieller kostenloser "Zero-Rating"-Zugang
    erreicht wird — nur ein alternativer Einstiegspunkt.

*(Hinweis zur maschinellen Übersetzung: Die Einträge 14–16 wurden vom
KI-Agenten selbst übersetzt, ohne Korrekturlesen durch einen
Muttersprachler.)*

**Wichtiger Vorbehalt**: Dieses Projekt ist ein Phase-0-Prototyp — KI-
Antwortqualität (GPT-2-basiert), Natürlichkeit der Sprachsynthese und
verlässliche Level-Anpassung sind nicht garantiert. Dies sollte auch
bei jeder Portierung klar offengelegt werden.

---

Weitere Sprachen: [日本語 (Original, vollständige Details)](PORTING.md) ·
[Italiano](PORTING-Italian.md) · [Français](PORTING-French.md) ·
[Русский](PORTING-Russian.md) · [Українська](PORTING-Ukrainian.md) ·
[עברית](PORTING-Hebrew.md) · [فارسی](PORTING-Persian.md)

## Muster: mehrsprachige Original-Prüfungen + Sprachauswahl + Vorlesen (2026-08-22)

1. Datenschicht in zwei Dateien trennen: Prüfungsaufgaben und Phrasen für
   das Vorlesen haben unterschiedliche Zwecke.
2. Übersichts-API ohne Aufgabentexte anbieten (`GET /v1/world-languages`),
   die vollständige JSON-Datei erst beim tatsächlichen Test laden.
3. Bestehende Prüfungs-UI nicht umbauen: nur eine `<optgroup>` mit Werten
   `world:<code>` ergänzen und Bewertung/Weiterleitung wiederverwenden.
4. Ober- und Untergrenze der Sprachauswahl in den Checkboxen erzwingen;
   bei erreichter Grenze **nur nicht angehakte** Boxen deaktivieren.
5. Vorhandene TTS wiederverwenden, nur eine Zuordnung Sprachcode → BCP-47
   ergänzen; ehrlich offenlegen, dass ohne installierte Stimme nur Text
   erscheint.
6. Texte in `readonly`-`<textarea>` ausgeben (kopierbar); zum Speichern
   den bestehenden Persistenz-Endpunkt nutzen statt neuer Tabellen.
