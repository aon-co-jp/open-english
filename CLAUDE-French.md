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
