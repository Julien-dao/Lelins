# 08 — Décisions actées (mai 2026)

> Décisions prises par Julien le 6 mai 2026 en réponse aux questions bloquantes
> de `docs/00-faisabilite-et-questions.md`. Ces décisions sont **figées** sauf
> révision explicite ultérieure.

---

## Décision 1 — Structure pédagogique : 4 CCP / 13 CP ✓

ANDREA s'aligne intégralement sur le **REAC V07 du 21/12/2022** (RNCP37275),
soit **4 CCP et 13 CP**. Le brief initial mentionnait 2 CCP — c'était l'ancienne
structure 2018 (V06), abrogée.

**Conséquences** :
- Système prompt déjà rédigé sur cette base (`docs/03`)
- Cartographie référentiel déjà alignée (`docs/04`)
- Espace « Mes cours » : grille 4 CCP × 13 CP
- Pipeline RAG : chunks structurés par CCP/CP
- Pipeline DP (en v2 Pro) : 4 sections, 1 par activité-type

---

## Décision 2 — Double profil hardware auto-détecté ✓

Au premier lancement, ANDREA détecte la RAM disponible et propose
automatiquement le bon modèle :

| RAM détectée | Modèle proposé | Quantisation |
|---|---|---|
| ≥ 24 Go | Mistral Small 3.2 24B (FR natif) | Q4_K_M |
| 12 - 24 Go | Mistral Nemo 12B ou Llama 3.1 8B | Q4_K_M |
| 8 - 12 Go | Phi-4-mini 3.8B ou Llama 3.2 3B | Q4_K_M |
| < 8 Go | Refus avec message clair | — |

Mention systématique : « ANDREA fonctionne au mieux à partir de 16 Go de RAM. »
L'utilisateur peut forcer un autre modèle dans Paramètres → Avancé.

---

## Décision 3 — Ollama en sidecar ✓

Choix : binaire `ollama` (MIT) embarqué en sidecar Tauri via `externalBin`,
piloté en HTTP localhost via `ollama-rs 0.3.4`.

Architecture en couches : un trait Rust `LlmProvider` permet une bascule
ultérieure vers `mistral.rs` (pure Rust) ou `llama.cpp` direct si Ollama
disparaît ou change de licence.

---

## Décision 4 — Piper GPL-3.0 en sidecar ✓

Choix : `OHF-Voice/piper1-gpl` (fork officiel mainteneu actif). La GPL-3.0
n'oblige qu'à fournir le source de **Piper** sur demande, pas le source
d'ANDREA, car Piper tourne en process séparé (sidecar).

**Action requise avant lancement** :
- Page « Crédits » dans l'app listant Piper avec lien vers le source
- Mention dans CGV : « ANDREA utilise Piper TTS (OHF-Voice/piper1-gpl, GPL-3.0).
  Le source de Piper est disponible sur demande via [contact]. »

Voix par défaut : `fr_FR-siwis-medium` (femme) + `fr_FR-tom-medium` (homme).

---

## Décision 5 — Signature de code active ✓

Apple Developer ID + certificat Windows Authenticode actifs côté Julien.
Pas de blocage côté distribution.

**Conséquences** :
- Notarization macOS dès l'étape 1 (build CI signé)
- SmartScreen Windows : avec certificat standard, alerte au début ; avec
  certificat EV (à envisager si budget), alerte évitée

---

## Décision 6 — Forme d'adresse : choix explicite à l'onboarding ✓

**Pas de défaut imposé.** L'écran d'onboarding pose explicitement la question :

> « Comment souhaitez-vous que je vous parle ? »
>
> [ Tutoiement (« tu ») ]   [ Vouvoiement (« vous ») ]

Les deux options sont présentées avec un poids visuel équivalent. La réponse
est sauvegardée dans `profile.tutoiement` et utilisée par le système prompt à
chaque tour. Modifiable à tout moment dans Paramètres.

**Note d'implémentation** : le système prompt accorde aussi le **genre du
formateur** (« votre formatrice » / « votre formateur ») selon l'avatar choisi.

---

## Décision 7 — Pas de dépôt INPI immédiat ✓

Lancement de la v1 en **usage descriptif** uniquement, qui est juridiquement
sûr (pratique commerciale très répandue pour les outils de préparation aux
titres professionnels — Dunod, ESF, GRETA, etc.).

**À envisager après le lancement v1** :
- Recherche d'antériorité INPI sur « ANDREA » dans les classes 9 (logiciels),
  41 (formation) et 42 (services informatiques)
- Si libre : dépôt (~250 € + 50 € par classe supplémentaire au-delà de la
  première)

---

## Décision 8 — Pas de module VAE dans ANDREA ✓

**Le module VAE est retiré du périmètre.** Le tier Maître est révisé en
conséquence.

### Tier Maître révisé (299 €)

- Tout le contenu Pro
- Suivi de cohorte (pour formateurs FPA accompagnant leurs apprenants)
- Export des évaluations en PDF pour archivage Qualiopi
- Bibliothèque de scénarios pédagogiques avancés

(Auparavant prévu : module VAE — supprimé.)

### Conséquences sur les autres docs

- README : retirer « Module VAE » et l'espace « Mon parcours VAE »
- Plan d'étapes Phase 3 : retirer le module VAE
- Espaces dans l'app (`docs/01`) : retirer route `/vae`
- Document risques : retirer le risque « VAE inversée échue 28/02/2026 »
  (devient sans objet)

Le référentiel FPA mentionne toujours la VAE comme voie d'accès, c'est une
information factuelle qu'ANDREA peut citer si l'utilisateur l'évoque, mais
**aucun module ni accompagnement VAE** n'est embarqué dans le produit.

---

## Questions encore ouvertes (à trancher en cours d'étape 1)

Réponses non bloquantes pour démarrer la phase 1, mais à confirmer avant les
étapes concernées :

| # | Question | Échéance |
|---|---|---|
| 9 | GitHub Releases public ou privé ? | Étape 7 (packaging) |
| 11 | Contenu pédagogique enrichissant : tu rédiges en propre, ou banque collective ? | Étape 4 (système prompt) puis Phase 2 |
| 12 | Mentions légales / CGV : modèle existant ou trame à proposer ? | Étape 8 (documentation) |
| 14 | Voix par défaut : femme ou homme ? | Étape 5 (onboarding) |
| 15 | Prénom du formateur par défaut : « ANDREA » uniquement, ou suggestions multiples ? | Étape 5 |
| 16 | Format `.andrea-backup` : dérivation auto par défaut + option passphrase avancée ? | Étape 1 |

Recommandations par défaut si pas de réponse explicite :

- 9. **Public** (simplicité, gratuit, standard open-source)
- 11. **Tu rédiges en propre** (PI Julien PERROT, valeur ajoutée différenciante)
- 12. **Trame à proposer** par moi en étape 8, validation par toi
- 14. **Femme par défaut** (cohérence avec « ANDREA » prénom plutôt féminin
  perçu en France ; modifiable)
- 15. **« ANDREA » + saisie libre** (pas de suggestions multiples — clarté)
- 16. **Dérivation auto par défaut + option passphrase avancée**
