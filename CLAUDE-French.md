# Philosophie de développement et règles d'environnement (open-english)

> **Note** : ceci est une traduction condensée de l'état actuel. Le
> journal historique détaillé des HANDOFF (des dizaines d'entrées
> depuis le 2026-08-10) reste disponible uniquement en japonais dans
> [CLAUDE.md](CLAUDE.md) par souci de concision — s'y référer pour le
> détail de chaque session.

Disque de travail : `F:\runo`. Cette section suit la pratique
consistant à copier la section correspondante du `CLAUDE.md` de
[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z) dans chaque
projet comme référence. Dépôt GitHub :
[aon-co-jp/open-english](https://github.com/aon-co-jp/open-english).

**Début du développement : 2026-08-10.**

## Rôle de ce projet

Une application web d'apprentissage de l'anglais pour PC/tablette/
smartphone. Dans le style d'un « cours d'anglais en maid café », un
personnage de maid magique (design original, animé) accompagne les
apprenants du débutant à l'avancé. Les réponses IA sont assurées par
[`aruaru-llm`](https://github.com/aon-co-jp/aruaru-llm).

## Architecture (selon les instructions de l'utilisateur, 2026-08-10)

- **Côté Linux (VPS)** : uniquement un serveur de distribution pour le
  téléchargement. La gestion de l'application est assurée par
  [`open-easy-web`](https://github.com/aon-co-jp/open-easy-web).
- **Côté appareil de l'utilisateur** : le frontend web statique de ce
  dépôt + un serveur natif exécuté localement par `aruaru-llm`
  (utilise en interne les backends d'inférence de `open-directx`/
  `open-cuda`), que l'utilisateur télécharge et exécute lui-même. Le
  navigateur se connecte localement (en ligne/hors ligne) à
  `http://localhost:4600`.

## Divulgation honnête / limitations connues (au 2026-08-10)

- `/v1/generate` de `aruaru-llm` est basé sur GPT-2 (centré sur
  l'anglais, sans fine-tuning pour le dialogue) — la qualité des
  réponses et le respect du niveau ne sont pas garantis.
- Le CORS a été corrigé le 2026-08-10 (`.with_cors()` côté
  `aruaru-llm`).
- La synthèse vocale (TTS)/synchronisation labiale pour les
  personnages tuteurs est branchée via la Web Speech API ; l'animation
  du personnage maid elle-même reste une simple boucle CSS
  d'ouverture/fermeture de bouche.

## Vision future (selon les instructions de l'utilisateur, 2026-08-10, pas encore commencée)

Il existe une idée de faire tourner `open-directx`/`open-cuda`/
`aruaru-llm` également de manière autonome dans le navigateur (WASM/
WebGPU) et de les intégrer avec `RPoem` (une plateforme GraphQL
Federation). Étant donné qu'il s'agit d'un changement architectural
majeur par rapport à la conception actuelle de la Phase 0 (serveur
résident local + connexion localhost), cela ne sera abordé qu'après
l'achèvement du MVP, avec un périmètre dédié.

## Autres mises à jour (2026-08-19 à 2026-08-20)

Depuis lors ont notamment été ajoutés : un message affiché à
l'atteinte de la limite d'utilisation quotidienne, renvoyant vers les
offres payantes et les quotas gratuits d'autres fournisseurs ; un
nouveau panneau « Data & Model Storage » avec interface de
sauvegarde/restauration ; une bannière incitant à mobiliser les
smartphones inutilisés (`PhoneAccelWorker` avec détection NNAPI) ;
une sauvegarde rsync combinée pour aruaru-db/PostgreSQL ; une page de
feuille de route provisoire pour les portages consoles
(PlayStation/Switch/Wii/Wii U, en attente de l'accord des
constructeurs) ; les mises à jour automatiques étendues à tous les
composants embarqués (aruaru-llm/aruaru-db) avec vérification
périodique toutes les 6 heures et rétrogradation manuelle par
composant ; un correctif faisant démarrer aruaru-llm automatiquement
sous Windows (plus aucune étape manuelle) ; une bannière d'information
sur les quotas gratuits des fournisseurs IA/recherche (incluant
Claude/Anthropic) ; un correctif critique empêchant la perte de la
base de données de conversation lors d'une mise à jour/rétrogradation
automatique Windows (vérifié de bout en bout via un véritable
installateur) ; ainsi que de nouvelles fonctionnalités Android
(détection automatique de l'URL du PC par balayage de sous-réseau,
vérifiée sur un appareil Snapdragon réel avec détection NNAPI
réussie, le déchargement de calcul NNAPI proprement dit restant non
implémenté). Détails dans les entrées HANDOFF du 2026-08-19 et du
2026-08-20 de [CLAUDE.md](CLAUDE.md).

## Mises à jour récentes liées à l'installation (2026-08-19)

Installateur Windows unifié `open-english-install.exe` ; la mise à jour
automatique intégrée (`server/src/self_update.rs`) a été étendue à
Linux puis à macOS, avec un retour arrière (rollback) automatique basé
sur un contrôle de santé `/healthz` si la nouvelle version échoue à
démarrer correctement ; nouvelle page d'entrée `facebook.html` pour les
utilisateurs n'ayant accès qu'à Facebook (divulgation honnête : ce
n'est pas un partenariat officiel avec Meta) ; et, depuis le
2026-08-18, l'application encourage et automatise l'installation de
`rsync` lorsqu'il est absent, avant de lancer directement la
sauvegarde. Détails complets uniquement dans les entrées HANDOFF
japonaises ci-dessous.

*(Note de traduction automatique : ce résumé a été traduit par l'agent
IA lui-même, sans relecture par un locuteur natif.)*

---

Autres langues : [日本語 (original, avec l'historique complet des HANDOFF)](CLAUDE.md) ·
[Deutsch](CLAUDE-German.md) · [Italiano](CLAUDE-Italian.md) ·
[Русский](CLAUDE-Russian.md) · [Українська](CLAUDE-Ukrainian.md) ·
[עברית](CLAUDE-Hebrew.md) · [فارسی](CLAUDE-Persian.md)

## Mise à jour 2026-08-22 : examens pour langues du monde, sélection des langues, lecture multilingue

Avant l'implémentation, le code a été vérifié : **il n'existait aucune
liste de langues de traduction ni table i18n** (`learn-target` ne
proposait que l'anglais et le japonais). La liste a donc été définie par
cette fonctionnalité — **38 langues** (17 européennes dont le romanche
suisse, le russe, 4 du Moyen-Orient, 7 d'Asie du Sud, 8 d'Asie de l'Est
et du Sud-Est, 1 africaine), plus l'anglais et le japonais toujours actifs.

- Nouvelles données : `world-language-exams.json` (questions originales à
  choix multiples avec niveaux de style CECR, 3 à 6 par langue) et
  `world-language-phrases.json` (5 phrases de base × 40 langues).
- Nouvelle API : `GET /v1/world-languages` ne renvoie que le résumé, sans
  les énoncés. Aucun nouvel endpoint n'a été créé pour l'enregistrement
  en base : `POST /v1/db/history` suffit.
- Interface : bannière bilingue, panneau « 🌐 Languages » avec cases à
  cocher, « Tout sélectionner » et « Tout désélectionner sauf EN et JA »,
  choix de **2 à 5 langues** (limite imposée par l'interface), affichage
  et lecture séquentiels rejouables à volonté (tout ou une seule langue),
  copie, téléchargement .txt et enregistrement dans SQLite.
- Après la correction, les questions manquées mènent à la conversation
  avec le tuteur de la langue (réutilisation d'`examPrepMissedQuestions`).

**Divulgation honnête** : questions originales, sans lien avec des
certifications officielles ; seulement 3 à 6 questions par langue ;
aucune relecture par des locuteurs natifs ; la lecture réelle dans les 38
langues n'a pas été vérifiée avec de vraies voix (uniquement via un stub
d'observation). Détails dans le HANDOFF du 2026-08-22 de
[CLAUDE.md](CLAUDE.md).

## Réponses à texte fixe déclenchées par des règles (2026-08-23)

Sur les sujets où une erreur factuelle fait le plus de dégâts, l'application
répond **sans inférence IA**, avec un texte écrit à la main. Trois cas
suivent le même schéma : `isCreatorQuestion()` (présentation de l'auteur),
`isReligionHistoryQuestion()` / `RELIGION_HISTORY_TEXTS` (islam, Iran, monde
arabe) et `isMarkOfBeastQuestion()` / `MARK_OF_BEAST_TEXTS` (666 et marque
de la bête). Raison : un GPT-2 brut invente, et sur l'histoire religieuse le
préjudice est élevé. Le quota quotidien n'est pas consommé ; les textes sont
dans une table indexée par code langue (aujourd'hui `ja` et `en` seulement).

**Histoire religieuse neutre** : diversité religieuse de l'Arabie
préislamique, formation du Coran décrite par la recherche comme une
**tradition autonome** (selon la doctrine islamique, révélation faite à
Mahomet), différence entre civilisations iranienne et arabe, influence
zoroastrienne présentée seulement comme la thèse de certains chercheurs.
Deux affirmations initialement souhaitées (Coran issu d'une traduction
biblique ; un frère de Mahomet traducteur) ont été **écartées faute de
sources**, après plusieurs échanges ; le texte dit *pourquoi* elles ne sont
pas avancées. La note sur les limites de la traduction prémoderne ne porte
**que** sur les traductions arabes de la Bible historiquement attestées et
ne doit **jamais** être reliée à la formation du Coran. En conclusion, le
message sur la traduction, le dialogue multilingue et la paix.

**666 / marque de la bête** : le passage de l'Apocalypse est cité
neutralement, sans affirmer d'interprétation « correcte ». Le jeu de mots
**« 666 = WWW »** (guématrie, vav = 6) est présenté **seulement comme une
lecture répandue depuis les années 1990**, pas comme une doctrine.
L'histoire du 666 caché dans les codes-barres est **qualifiée de légende
urbaine** et expliquée techniquement : les **barres de garde** marquent le
début, la fin et le milieu pour le scanner, ne ressemblent au chiffre 6 que
visuellement et relèvent d'un autre encodage (3 modules contre 7) ; Snopes
la juge FAUSSE — **aucun sens occulte, aucun fondement technique**.
Conclusion positive (le web et les scanners rendent les achats commodes
**sans marque corporelle**) et note sur Python (logo serpent, nom venu de
Monty Python) **déclarée pure coïncidence**.

**Ajout du 2026-08-23 (parallèle intéressant — présenter, non affirmer)** :
Apocalypse 13,16-17 contient bien un passage disant que nul ne peut acheter
ni vendre sans la marque (l'existence de ce verset peut être citée comme un
fait). Une phrase ajoutée signale que **certaines personnes y voient un
parallèle intéressant** avec le fait que les achats modernes reposent de plus
en plus sur les codes-barres et les paiements en ligne comme Amazon.
**Contrainte impérative** : présenter cela uniquement comme une possibilité
(« certains le disent »), **jamais** affirmer qu'une prophétie se serait
accomplie — le texte le précise lui-même.

**En cas de modification** : les quatre garanties d'honnêteté — « présenter
et non affirmer », « légende déclarée légende », « coïncidence déclarée
coïncidence » — ne doivent être ni affaiblies ni omises.
