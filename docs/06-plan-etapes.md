# 06 — Plan d'étapes affiné Phase 1 (ANDREA Découverte)

> Document produit le 6 mai 2026, à valider avant attaque.
> Méthodologie : étapes validées une à une, jamais de saut, commit + push à
> chaque étape, agent reviewer systématique avant validation.

---

## Légende

- 🎯 **Livrable concret** : ce qui doit exister à la fin
- ✅ **Critère de validation** : ce qui prouve que l'étape est terminée
- ⏱️ **Estimation** : temps calendaire, en jours-personne (JP) à plein temps
- 🤝 **Mobilisation** : sub-agents et skills mobilisés
- ⚠️ **Risques** : pièges identifiés en amont

---

## Étape 1 — Fondations techniques (≈ 7 JP)

🎯 **Livrables**

- Repo initialisé (déjà fait, branche `claude/andrea-architecture-design-fIbrP`)
- Workspace pnpm + cargo : `apps/desktop` Tauri + `apps/license-server` Node
- Tauri 2.6 + React 19 + Vite + Tailwind 4 + shadcn installé et build OK
- Architecture séparation données/code (paths système macOS/Windows/Linux)
- Migration runner maison (`db/migrations.rs`) + migration `0001_initial.sql`
  (toutes tables documentées dans `01-architecture.md`)
- Module `license` : encode/decode Crockford, MAC verify, blocklist
- Outil CLI `apps/license-server/cli.ts` capable de générer une clé de test
- Tauri plugin updater configuré (clés Minisign générées hors repo)
- Système d'export/import `.andrea-backup` (squelette + tests round-trip)
- CI GitHub Actions : `lint`, `test`, `build` sur macOS arm64 + Windows
- Hooks pre-commit (clippy, prettier, eslint, rustfmt)

✅ **Validation**

- `pnpm tauri dev` lance une fenêtre vide signée ANDREA
- `cargo test` passe (tests migrations + tests license verify avec fixtures)
- Génération d'une clé `ANDREA-DECO-XXXXX-...` puis vérification offline OK
- Export `.andrea-backup` puis import sur DB vide reconstitue le profil
- Build CI verte sur les 2 OS

⏱️ **Estimation** : 7 JP (1 semaine)

🤝 **Mobilisation**
- Agent **Plan** pour l'architecture détaillée du squelette monorepo
- Agent **claude-code-guide** pour points spécifiques Tauri 2.6 / sidecar
- Skill **simplify** appliqué une fois la base posée
- Skill **fewer-permission-prompts** pour ajouter les commandes courantes au
  allowlist du projet
- Agent reviewer (general-purpose) : revue croisée du module licence (crypto)

⚠️ **Risques**
- Notarization macOS : tester dès cette étape un build signé end-to-end pour
  ne pas découvrir 6 mois plus tard un blocage Apple
- Sidecar non encore intégré : prévoir le placeholder

---

## Étape 2 — Moteur conversationnel local (≈ 10 JP)

🎯 **Livrables**

- Sidecar Ollama embarqué (binaire MIT) avec téléchargement automatique au
  premier lancement (script Rust + UI progress bar)
- Détection RAM système → choix du modèle (Mistral Small 3.2 24B / Mistral
  Nemo 12B / Phi-4-mini 3.8B selon profil)
- Wrapper `llm/ollama.rs` derrière trait `LlmProvider`
- STT via whisper-rs 0.16 + modèle `small` français (téléchargement auto)
- TTS sidecar Piper (OHF-Voice/piper1-gpl) + voix `fr_FR-siwis-medium` et
  `fr_FR-tom-medium`
- Pipeline conversation complet : capture audio → VAD → STT → LLM streaming →
  TTS streaming
- Gestion barge-in (kill TTS si voix utilisateur détectée)
- Bench rapide : 3 scénarios français mesurés sur M1 et PC i5

✅ **Validation**

- Démo : « ANDREA, peux-tu me dire bonjour ? » → réponse vocale correcte
  en moins de 2 s sur M1 16 Go
- Switch texte ↔ vocal pendant une session sans perte de contexte
- Barge-in fonctionnel : interrompre ANDREA en parlant
- Aucun hardcode anglais : tous les modèles configurés FR

⏱️ **Estimation** : 10 JP (2 semaines)

🤝 **Mobilisation**
- Agent **Explore** : repérer les patterns d'intégration Ollama+Tauri en open
  source (peu de précédents, recherche large utile)
- Agent **claude-code-guide** : Tauri sidecar + permissions
- Skill **security-review** sur la couche subprocess (injection, escape)

⚠️ **Risques**
- Sandbox macOS bloque le sidecar silencieusement → tester en CI macOS notarized
- Premier lancement (5-15 Go download) UX critique : reprise sur coupure réseau
- Latence STT trop élevée sur Windows CPU i5 → fallback `tiny` voire `base`

---

## Étape 3 — Référentiel FPA en RAG (≈ 6 JP)

🎯 **Livrables**

- Téléchargement manuel des PDFs officiels REAC V07 + RE V07 + arrêtés
  (placés dans `data/referentiel-fpa/`)
- Pipeline de chunking (`scripts/chunk-referentiel.ts`) : extraction texte,
  parsing structuré CCP/CP/section, génération JSONL
- ~800-1500 chunks avec métadonnées (citation_human, source_doc, ccp, cp)
- Embarquement du JSONL dans `apps/desktop/src-tauri/resources/`
- Au premier lancement : ingestion en sqlite-vec (avec embedding via Ollama
  bge-m3, ~3 min sur M1)
- Module `rag/` : embed query, top-K retrieval, formatage citations
- 30 questions-test sur le référentiel (set d'évaluation)

✅ **Validation**

- 90 % des 30 questions-test retrouvent au moins 1 chunk pertinent dans le top-3
- Citations correctement formatées : « Source : REAC V07 21/12/2022, CP3 »
- Latence retrieval < 100 ms sur sqlite-vec
- Tests CI : ingestion idempotente, ré-ingestion ne duplique pas

⏱️ **Estimation** : 6 JP

🤝 **Mobilisation**
- Agent **general-purpose** : récupération PDFs officiels + extraction
- Agent reviewer : audit des chunks (pas de découpage cassant le sens, pas
  d'invention)
- Toi (Julien) : validation manuelle d'un échantillon de chunks (10-20)

⚠️ **Risques**
- PDFs REAC scannés ou avec CMap exotique → fallback OCR (Tesseract) prévu
- Si la fiche officielle change de format avant cette étape, recommencer
  l'extraction
- Évolutions réglementaires en parallèle (VAE inversée échue 28/02/2026) → à
  surveiller

---

## Étape 4 — Système prompt ANDREA Formateur (≈ 5 JP)

🎯 **Livrables**

- `packages/prompts/andrea-formateur-v1.md` finalisé (cf. `03-systeme-prompt-...`)
- `llm/prompt.rs` : interpolation Jinja-like des variables, chargement
  `include_str!` à la compilation
- 30 scénarios de test pédagogique :
  - 10 questions référentiel
  - 5 tests garde-fous (« écris mon DP », « garantis le titre »)
  - 5 hors-sujet
  - 5 mises en situation pédagogiques
  - 5 questions sur la posture
- Harness de test automatique : exécute les 30 scénarios, score
  citation/garde-fou/posture (LLM-as-judge avec un 2e modèle pour évaluer)
- Itérations jusqu'à 90 % de réussite

✅ **Validation**

- 90 %+ de réussite sur les 30 scénarios
- Validation manuelle Julien sur 5 sessions réelles longues (≥ 30 min chacune)
- Garde-fous JAMAIS franchis sur 100 essais adversariaux

⏱️ **Estimation** : 5 JP (mais peut s'étirer selon itérations)

🤝 **Mobilisation**
- Toi (Julien) : tu es l'expert pédagogique, ta validation pèse plus que tout
- Agent **general-purpose** : générer des scénarios de test diversifiés
- Skill **simplify** sur le prompt à la fin pour éliminer les redondances

⚠️ **Risques**
- Le LLM hallucine le référentiel malgré le RAG → temperature 0.3, force
  citation, fallback "je ne suis pas certain"
- Le prompt grossit incontrôlablement à force d'itérations → discipline,
  limite 4000 tokens hors RAG
- Difficulté à mesurer la "qualité andragogique" → grille manuelle de revue
  par toi

---

## Étape 5 — Onboarding et personnalisation (≈ 6 JP)

🎯 **Livrables**

- Écran de bienvenue animé (Framer Motion, palette ANDREA)
- Saisie clé licence + lien Gumroad
- Téléchargement progressif des modèles (LLM + Whisper + Piper) avec UI
  rassurante (« Nous installons votre formateur, ~5 Go, ne se fera qu'une
  fois »)
- Onboarding : prénom, comment être appelé, contexte, niveau, objectif,
  cadence, date d'épreuve
- Galerie 12 avatars (8 sourcés + 4 IA) incluant représentation ultramarine
- Sélection voix (homme/femme, écoute samples)
- Choix tutoiement/vouvoiement
- Première session de découverte guidée par ANDREA elle-même
- Sauvegarde profil

✅ **Validation**

- 5 personnes testent l'onboarding sur leurs propres machines (Mac M1 16 Go,
  Mac Air M2 8 Go, PC i5 16 Go, etc.)
- Aucun blocage, aucune confusion sur les étapes
- Téléchargements reprennent après coupure réseau
- Profil sauvegardé est correctement utilisé en session 1

⏱️ **Estimation** : 6 JP

🤝 **Mobilisation**
- Skill **frontend-design** intensivement
- Agent **Explore** : patterns d'onboarding premium 2026 (Linear, Granola)
- Toi (Julien) : sélection finale des avatars et phrase d'échantillon TTS

⚠️ **Risques**
- Les 5-15 Go de download : utilisateur abandonne. Prévoir messages de réassurance
  réguliers + estimation temps restante précise
- Saisie clé erronée (espaces, majuscules, confusion 0/O) → normalisation
  Crockford stricte + feedback temps réel

---

## Étape 6 — Interface utilisateur Découverte (≈ 12 JP)

🎯 **Livrables**

- Layout principal : sidebar 240 px + contenu principal
- Espace **Mes cours** : grille 4 CCP × 13 CP avec progression visuelle
  (anneaux de progression Bloom 0-4)
- Espace **Ma progression** : vue d'ensemble + jalons + dates épreuve
- Espace **Mes paramètres** : profil, avatar, voix, langue, thème, licence,
  export/import backup
- Espace **Mon coffre-fort (basique)** : notes texte + upload simple PDF
- Espaces verrouillés (Pro / Maître) visibles avec cadenas + CTA Gumroad
- Conversation : avatar animé + bulles + indicateurs d'état
- Mode sombre/clair fonctionnel
- Accessibilité WCAG 2.2 AA (Tab order, lecteurs d'écran, focus visible)
- Tests Playwright sur 5 parcours-clés

✅ **Validation**

- Toutes les routes accessibles au clavier
- Test contraste WCAG : aucune erreur AA
- Test lecteur d'écran (VoiceOver Mac + NVDA Windows) sur les 4 espaces clés
- Test densité visuelle : 1080p / 1440p / 4K → pas de débordement
- Validation Julien sur le ressenti général

⏱️ **Estimation** : 12 JP (3 semaines)

🤝 **Mobilisation**
- Skill **frontend-design** sur chaque écran
- Agent **Plan** pour l'architecture des composants partagés
- Agent reviewer : revue UI cohérence

⚠️ **Risques**
- Avatar animé qui consomme trop de CPU → optimisation Lottie / SVG sprite
- Animations Framer Motion qui ralentissent vieux Mac → respect
  `prefers-reduced-motion`
- Densité d'information sur la grille 4×13 = 52 cellules → travail typo et
  spacing

---

## Étape 7 — Packaging et distribution (≈ 8 JP)

🎯 **Livrables**

- Build cross-platform : `andrea-1.0.0-aarch64.dmg`,
  `andrea-1.0.0-x86_64.dmg`, `andrea-1.0.0-x64.msi`
- Signature de code : Apple Developer ID + cert Authenticode Windows
- Notarization macOS automatisée (GitHub Action)
- GitHub Releases configuré, manifeste `latest.json` signé Minisign auto-généré
- Tests sur 4 machines réelles : MacBook Air M1 8 Go, MacBook Pro M3 16 Go,
  PC Windows i5 16 Go, PC Windows AMD Ryzen 32 Go
- Microservice `apps/license-server/` déployé (Vercel/Railway/Cloudflare)
- Webhook Gumroad → génération de clé → email automatique

✅ **Validation**

- Installation par double-clic sans message d'avertissement Gatekeeper
  (Mac) / SmartScreen (Windows)
- Premier lancement complet jusqu'à conversation : <15 min sur fibre,
  <30 min sur ADSL
- Achat test Gumroad → réception clé par email → activation OK
- Auto-updater : v1.0.0 → v1.0.1 sans intervention utilisateur

⏱️ **Estimation** : 8 JP

🤝 **Mobilisation**
- Skill **security-review** sur le pipeline de release
- Agent **claude-code-guide** : signature de code Tauri 2.6
- Agent reviewer : audit final avant publication

⚠️ **Risques**
- Notarization Apple bloquée >16h (constaté en 2026) → buffer dans le planning
- Sidecars non signés individuellement → notarization rejette
- Webhook Gumroad mal configuré → clé non envoyée → 1er bug client critique

---

## Étape 8 — Documentation utilisateur (≈ 4 JP)

🎯 **Livrables**

- Guide de démarrage rapide (`docs/user/quickstart.md` + version PDF distribuée)
- FAQ (10-15 questions) intégrée dans l'app + site
- Vidéo de présentation 2-3 min (script + tournage à organiser)
- Page Gumroad complète (titre, sous-titre, captures, vidéo, descriptions
  des 3 tiers, comparatif)
- CGV / mentions légales / politique RGPD (avec mention obligatoire
  « ANDREA n'est pas un certificateur… »)
- README.md projet final
- CHANGELOG.md initialisé

✅ **Validation**

- Un utilisateur novice (non technique, non FPA) suit le quickstart et
  arrive à une conversation en moins de 30 min
- Page Gumroad relue par Julien et éventuel conseil juridique
- Vidéo en ligne, lien public

⏱️ **Estimation** : 4 JP

🤝 **Mobilisation**
- Toi (Julien) : ton style, ton expertise, ton image
- Skill **simplify** pour la pédagogie de la doc

⚠️ **Risques**
- Documentation qui dérive en encyclopédie → discipline, max 1500 mots par
  page sauf section référence
- CGV trop génériques exposant juridiquement → relecture conseil

---

## Vue d'ensemble Phase 1

```
Étape 1  ─┐
Étape 2  ─┤  Tech foundation                  ≈ 17 JP
Étape 3  ─┘
Étape 4  ─┐  Pédagogie embarquée               ≈ 11 JP
Étape 5  ─┘
Étape 6  ───  UI utilisateur                   ≈ 12 JP
Étape 7  ─┐  Distribution & lancement          ≈ 12 JP
Étape 8  ─┘
            ─────────────────────────────────────────
TOTAL                                          ≈ 52 JP
```

À temps plein : ~10-11 semaines (≈ 2.5 mois). Avec marge pour validation, tests
réels, itérations sur le prompt et imprévus de notarization : **objectif M+3
réaliste**, M+4 confortable.

---

## Méthode de revue à chaque étape

1. Sub-agent **general-purpose** ou **Explore** réalise une **revue croisée**
   du code (cohérence, sécurité, simplicité)
2. Skill **simplify** appliqué sur les changements
3. Skill **security-review** appliqué sur les modules sensibles (licence,
   crypto, sidecar, parsing externe)
4. Tests CI verts (lint + tests unitaires + build)
5. Démo manuelle à Julien (vidéo Loom courte ou session live)
6. **Validation explicite** de Julien avant passage à l'étape suivante

---

## Phase 2 — ANDREA Pro (vue rapide, à détailler en M+3)

- Étape 9 : Module évaluation de documents (PDF/DOCX/ODT, grille jury)
- Étape 10 : Assistant Dossier Professionnel
- Étape 11 : Mises en situation professionnelles (bibliothèque scénarios)
- Étape 12 : Simulateur de jury 3 épreuves
- Étape 13 : Coffre-fort avancé (chiffrement, versions)
- Étape 14 : Tableau de bord progression avancé
- Étape 15 : Migration v1 → v2 sans perte de données

## Phase 3 — ANDREA Maître (vue rapide, M+9 ou M+12)

- Module VAE
- Suivi de cohorte (vue formateur)
- Exports Qualiopi PDF
- Bibliothèque scénarios pédagogiques avancés
