# PORTING.md — Guide de portage d'open-english (version condensée)

> **Note** : ceci est une traduction condensée. Le guide technique
> complet avec détails de code et pièges reste disponible uniquement
> en japonais dans [PORTING.md](PORTING.md) — s'y référer avant
> d'adopter réellement un pattern.

Résumé des patterns d'implémentation réutilisables de ce projet, au
cas où ils seraient portés vers un autre projet :

1. **Pattern d'intégration `aruaru-llm`** : nécessite un support CORS
   (`.with_cors()`) et une pénalité de répétition
   (`generate_with_repetition_penalty`, par défaut 1,3) côté
   `aruaru-llm`, sinon le navigateur bloque `fetch()` ou le décodage
   glouton de GPT-2 tombe dans une boucle de répétition.
2. **Extraction de la langue pour le TTS** (`extractSpeechText`) :
   sépare les lignes bilingues au format "English / 日本語" avant de
   les passer à `SpeechSynthesisUtterance`, pour éviter une
   prononciation saccadée.
3. **Jeu d'icônes de lancement** (`icons/` + `manifest.json` +
   `launchers/`) : encodeur PNG/ICO écrit à la main sans outils
   graphiques externes, plus des scripts pour `.lnk` Windows,
   `.desktop` Linux et `.app` macOS.
4. **Mise à jour automatique** (`auto-update.js` + `version.json`) :
   simple polling du `buildId` toutes les 5s avec `location.reload()`.
5. **Serveur statique basé sur RPoem** (`server/`) : remplace
   `python3 -m http.server` par un crate Rust réutilisant le
   `static_file_handler` de `open-runo-poem-compat`.
6. **Garantie structurelle des réponses hybrides**
   (`ensureHybridReply`) : ajoute automatiquement une courte note en
   japonais si la réponse du modèle ne contient pas de japonais — sans
   exagérer la qualité de traduction.
7. **Gestion des versions + nettoyage côté navigateur des anciennes
   versions** : `version.json` avec un champ `version`, suppression
   des clés `localStorage` propres à l'application lors d'une nouvelle
   version.
8. **Bridge Google Custom Search JSON API + panneau de paramètres dans
   le navigateur** : identifiants uniquement en mémoire de processus,
   jamais sur disque.
9. **Installateur Windows (Inno Setup) — étapes de vérification
   réelle** : `PrivilegesRequired=lowest` contre les blocages UAC,
   `MSYS_NO_PATHCONV=1` pour les installations silencieuses depuis Git
   Bash.
10. **Base de données géo/tourisme avec avis de sécurité pilotés par le
    sujet** : recherche par sous-chaîne plutôt que correspondance
    exacte ; pour les sujets dangereux (ex. alpinisme), toujours
    joindre un avis de sécurité.
11. **Mise à jour automatique via GitHub Releases (Windows/Linux/macOS,
    voir aussi le point 14 pour le rollback)** : doit être implémentée
    côté natif (pas dans le JS du navigateur) ; son propre .exe en cours
    d'exécution ne peut pas être supprimé sous Windows — lancer d'abord
    un script batch détaché, puis terminer son propre processus.
12. **Les classes CSS des modales nécessitent `max-height`/
    `overflow-y`** : chaque classe de conteneur de modale doit définir
    la scrollabilité, sinon le contenu long est inaccessible sur
    mobile.
13. **Inclusion d'un serveur Rust sur Android** : rendre les chemins
    surchargeables au runtime via une variable d'environnement ;
    toujours protéger la logique de mise à jour automatique spécifique
    à Windows avec `cfg!(target_os = "windows")`.

14. **Mise à jour automatique multiplateforme avec retour arrière
    (rollback) piloté par un health-check** : généraliser
    `apply_update_linux` en `apply_update_unix` (Linux et macOS
    partagent le remplacement du binaire en cours d'exécution) ;
    sauvegarder le binaire actuel avant d'appliquer une mise à jour ;
    après le démarrage de la nouvelle version, sonder un nouveau point
    de terminaison `/healthz` dans un court délai, et restaurer la
    sauvegarde en cas d'échec. Android/iOS restent explicitement hors
    de portée (contrainte de la plateforme, pas un oubli).
15. **Promotion + installation automatique de RSync** : si `rsync` est
    introuvable, afficher un message bilingue engageant ("Let's install
    RSync!") plutôt qu'une erreur sèche, puis tenter une installation
    automatique via une chaîne de gestionnaires de paquets selon l'OS
    (winget→choco sur Windows, apt-get→dnf→pacman sur Linux, brew sur
    macOS, pkg sur Termux/Android), et enchaîner directement sur la
    sauvegarde en cas de succès.
16. **Page d'entrée dédiée pour un canal d'accès restreint** (ex. :
    `facebook.html` pour les utilisateurs limités à Facebook) : une
    simple page statique réutilisant les liens de téléchargement déjà
    présents dans le README, avec une divulgation honnête explicite
    précisant qu'aucun accès "zero-rated" gratuit officiel n'est
    obtenu — seulement un point d'entrée alternatif.

*(Note de traduction automatique : les entrées 14 à 16 ont été
traduites par l'agent IA lui-même, sans relecture par un locuteur
natif.)*

**Avertissement important** : ce projet est un prototype de Phase 0 —
la qualité des réponses IA (basée sur GPT-2), le naturel de la synthèse
vocale et l'adaptation fiable au niveau ne sont pas garantis. Cela doit
également être clairement indiqué lors de tout portage.

---

Autres langues : [日本語 (original, détails complets)](PORTING.md) ·
[Deutsch](PORTING-German.md) · [Italiano](PORTING-Italian.md) ·
[Русский](PORTING-Russian.md) · [Українська](PORTING-Ukrainian.md) ·
[עברית](PORTING-Hebrew.md) · [فارسی](PORTING-Persian.md)

## Modèle : examens originaux multilingues + sélection des langues + lecture (2026-08-22)

1. Séparer les données en deux fichiers : les questions d'examen et les
   phrases lues n'ont pas le même usage.
2. Exposer une API de résumé sans les énoncés (`GET /v1/world-languages`)
   et ne charger le JSON complet qu'au lancement du test.
3. Ne pas réécrire l'interface d'examen : ajouter seulement un
   `<optgroup>` avec des valeurs `world:<code>` et réutiliser la
   correction ainsi que le passage au tuteur.
4. Imposer le minimum et le maximum de langues au niveau des cases à
   cocher ; à la limite atteinte, ne désactiver que les cases **non
   cochées**.
5. Réutiliser la synthèse vocale existante en n'ajoutant que la table
   code → BCP-47 ; indiquer honnêtement qu'en l'absence de voix installée
   le texte est seulement affiché.
6. Afficher les textes dans des `<textarea>` `readonly` (copiables) et
   enregistrer via l'endpoint de persistance existant, sans nouvelle table.

## Motif : réponses à texte fixe déclenchées par des règles (2026-08-23)

1. Trois briques : fonction de détection par mots-clés + table de textes
   indexée par code langue + fonction qui compose la réponse bilingue.
   À placer dans le gestionnaire d'envoi **avant** le chemin IA.
2. Combiner mot-thème ET mot-intention, pour qu'un « Iran » lâché au
   passage ne détourne pas la conversation ordinaire. En anglais, ne mettre
   la limite de mot `\b` **qu'au début**, sinon « Zoroastrianism » échappe
   à la détection (bug réellement rencontré).
3. Tester d'abord les sujets les plus précis (666 avant l'histoire
   religieuse).
4. Écrire les garanties d'honnêteté dans le texte lui-même : on
   **présente** les thèses, on **déclare** les légendes urbaines comme
   telles (avec l'explication technique quand c'est possible, comme pour
   les barres de garde), on **nomme** les coïncidences comme coïncidences ;
   les « parallèles intéressants » entre textes anciens et situation actuelle
   sont **présentés seulement comme une possibilité** (« certains le disent »)
   et jamais comme une prophétie accomplie (exemple : Apocalypse 13,16-17
   « sans la marque, ni acheter ni vendre » rapproché des codes-barres et des
   paiements en ligne comme Amazon).
5. Documenter ce qui a été écarté et pourquoi — dans la réponse et dans un
   commentaire de code (« à lire avant toute modification »).
