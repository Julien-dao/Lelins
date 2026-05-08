# 04 — Cartographie du référentiel FPA (mai 2026)

> Document produit le 6 mai 2026, basé sur recherches croisées (France
> Compétences, Légifrance, REAC V07 21/12/2022, sources Qualiopi et organismes
> habilités). À utiliser comme source initiale pour le RAG d'ANDREA et comme
> base de discussion avec Julien pour validation finale.
>
> **Niveau de certitude global : élevé sauf zones marquées « À VÉRIFIER »**.
> Les sources officielles primaires (PDF AFPA / Légifrance / France Compétences)
> ont retourné des erreurs 403 lors de la récupération directe automatisée. Les
> éléments ci-dessous croisent (a) les pages officielles indexées, (b) des
> copies miroirs du REAC/RE V07, (c) des fiches d'organismes Qualiopi.
> Une vérification finale doit être faite manuellement sur la fiche France
> Compétences avant intégration dans le RAG d'ANDREA.

---

## 1. Identifiants officiels

| Champ | Valeur |
|---|---|
| **Numéro RNCP** | **RNCP37275** |
| **Intitulé** | TP — Formateur professionnel d'adultes |
| **Niveau de qualification** | **Niveau 5** (équivalent Bac+2 EQF/CEC) |
| **Date d'enregistrement** | 29 avril 2023 |
| **Date de fin d'enregistrement** | **29 avril 2028** (5 ans) |
| **Code(s) NSF** | **333** Enseignement, formation ; **333t** Éducation et transfert de connaissances |
| **Formacode(s)** | **44586** Formation formateur (principal) ; **44542** Pédagogie ; **44517** Conception action de formation ; **15084** Préparation entrée formation ; **15041** Mise à niveau |
| **Code ROME** | **K2111** Formation professionnelle |
| **Certificateur** | **Ministère du Travail** (DGEFP). L'AFPA n'est PAS certificateur ; elle est concepteur du REAC pour le compte du Ministère |
| **Identifiant France VAE** | 84ca34d2-991e-4037-9f52-30dab61f9d26 |

### Textes réglementaires fondateurs (en vigueur en mai 2026)

- **Arrêté du 7 décembre 2022** relatif au TP-FPA (JORFTEXT000046751421)
- **Arrêté du 24 janvier 2023** modifiant le précédent (JORFTEXT000047074402)
- **Arrêté du 22 décembre 2015** (cadre général TP) (JORFTEXT000031733810)

---

## 2. Structure du titre — 4 CCP, 13 CP

> ⚠️ **Correction par rapport au brief initial** : depuis l'arrêté du 7/12/2022,
> le titre est structuré en **4 CCP / 13 compétences professionnelles**.
> La structure 2 CCP correspondait au REAC 2018 (V06), abrogé. Toute la
> conception d'ANDREA doit s'aligner sur la version V07.

### CCP1 — Concevoir et préparer la formation (3 CP)

- **CP1.** Élaborer la progression pédagogique d'une action de formation à
  partir d'une demande
- **CP2.** Concevoir le scénario pédagogique d'une séquence de formation, en
  intégrant la multimodalité
- **CP3.** Concevoir les activités d'apprentissage et d'évaluation des acquis,
  en intégrant la multimodalité

### CCP2 — Animer la formation et évaluer les acquis (3 CP)

- **CP4.** Animer un temps de formation collectif en présence et à distance,
  en favorisant les interactions
- **CP5.** Évaluer les acquis d'apprentissage des apprenants
- **CP6.** Inscrire ses actes professionnels dans une démarche de
  responsabilité sociale, environnementale et professionnelle (RSE)
  *(À VÉRIFIER : libellé exact et rattachement précis CCP2 vs CCP4)*

### CCP3 — Accompagner les apprenants en formation (4 CP)

- **CP7.** Accueillir les apprenants en formation et co-construire leur
  parcours
- **CP8.** Accompagner les apprenants dans la construction de leur parcours
  et la sécurisation de leurs acquis
- **CP9.** Tutorer les apprenants à distance
- **CP10.** Accompagner le développement professionnel des apprenants
  (remédiation et individualisation)

### CCP4 — Inscrire sa pratique dans une démarche qualité et RSE (3 CP)

- **CP11.** Maintenir son expertise pédagogique et technique (veille,
  développement professionnel)
- **CP12.** Analyser ses pratiques professionnelles
- **CP13.** Respecter et promouvoir la réglementation en vigueur (Qualiopi,
  RGPD, accessibilité, droit de la formation) et les principes RSE

### Compétences transversales (annexe REAC)

- Mobiliser un environnement numérique professionnel
- Communiquer à l'écrit comme à l'oral en s'adaptant aux interlocuteurs
- Agir dans une logique inclusive (handicap, illettrisme, mixité)

---

## 3. REAC — Référentiel Emploi Activités Compétences

**Téléchargement officiel** :
- `https://travail-emploi.gouv.fr` (rubrique « Titres professionnels »)
- `https://www.banque.di.afpa.fr/EspaceEmployeursCandidatsActeurs/`
- Copies miroir publiques (non officielles, utiles pour vérification) :
  - `https://formanov.com/wp-content/uploads/REAC_FPA_V07_21122022.pdf`

**Sections principales du REAC** :

1. Présentation de l'emploi-type
2. Fiche emploi-type (mission, environnement, relations professionnelles)
3. Fiches Activités-types (1 par CCP, soit 4 fiches)
4. Fiches Compétences professionnelles (13 fiches détaillant chaque CP)
5. Glossaire technique du métier
6. Glossaire du REAC

**Pour chaque CP**, le REAC liste :

- Savoir-faire professionnels (gestes attendus)
- Savoirs associés (théoriques)
- Savoir-être (posture éthique)
- Critères de performance
- Modalités d'évaluation associées (renvoi vers le RE)

---

## 4. RE — Référentiel d'Évaluation

**Source pérenne** : publié sur travail-emploi.gouv.fr en annexe de l'arrêté ;
copies miroir vérifiables sur :
- `https://formanov.com/wp-content/uploads/REV2_FPA_V07_21122022.pdf`
- `https://schola-ingenierie.fr/wp-content/uploads/2023/01/REV2_FPA_V07_21122022.pdf`

### Architecture des épreuves (session de validation totale du titre)

**Durée totale candidat ≈ 3 h** (concordance multi-sources).

#### A. Mise en Situation Professionnelle (MSP) reconstituée

- **Format** : présentation orale d'une analyse de pratique professionnelle, à
  partir d'un sujet tiré au sort parmi 6 sujets possibles
- **Préparation hors présence du jury** : **45 minutes**
- **Présentation orale devant le jury** : **10 minutes**
- **Objet** : démontrer la maîtrise des gestes professionnels du métier sur
  les 4 activités-types

#### B. Entretien technique

- **Durée** : **20 minutes** de questions du jury
- **Objet** : approfondir les compétences techniques liées au sujet de la MSP
  et investiguer les zones non couvertes

#### C. Questionnement à partir de productions / Entretien sur le DP

- **Durée indicative** : **1 h 45** (cumulé avec présentation des
  activités-types et discussion DP)
- **Objet** : interroger le candidat à partir de ses productions intégrées au
  DP

#### D. Entretien final

- **Durée** : **20 minutes**
- **Objet** : positionnement professionnel, projection, représentation de
  l'emploi et de son environnement (notamment éthique et RSE)

> *À VÉRIFIER (chiffres précis)* : les durées concordent entre sources mais le
> découpage exact entre B/C/D peut varier d'une publication à l'autre.
> La règle : se référer au RE V07 du 21/12/2022 publié.

### Composition du jury

- **2 professionnels du métier** (formateurs d'adultes en exercice, ou ayant
  exercé depuis moins de 5 ans)
- Habilités par le **Préfet via la DREETS/DDETS**, après vérification de
  **3 ans d'expérience minimum** dans le métier visé
- Habilitation valable au plus pour la durée de validité du titre
- Cadre juridique : Arrêté du 22/12/2015, articles 5 à 8

### Documents dont dispose le jury pour décider

1. Résultats de la MSP
2. Le **Dossier Professionnel (DP)**
3. Le livret des **Évaluations Passées en Cours de Formation (ECF)** —
   uniquement si le candidat vient d'un parcours de formation
4. Résultats de l'entretien final

---

## 5. Dossier Professionnel (DP)

**Cadre normatif.** Le DP suit le format générique du « Guide du formateur DP »
du Ministère du Travail (version de référence du 1er juin 2016, toujours en
vigueur dans sa structure générique).

### Structure officielle attendue

1. Page de garde + identification du candidat
2. **Sommaire** facilitant la lecture du jury
3. **Description de la pratique professionnelle**, organisée par
   activité-type (donc 4 sections, une par CCP)
4. Tableau des titres/diplômes possédés (annexe optionnelle)
5. **Déclaration sur l'honneur signée**

### Règle des exemples par activité-type

- **Maximum 3 exemples** de pratique professionnelle par activité-type
- **Maximum 3 pages par exemple**
- Soit, dans le cas FPA à 4 CCP : **jusqu'à 12 exemples ≈ 36 pages utiles**
  + annexes

### Critères du jury sur le DP

- Précision et concrétude des situations
- Capacité d'analyse réflexive (et non simple description)
- Cohérence avec les compétences visées
- Authenticité (le DP est le support de l'entretien final, le jury croise les
  déclarations)

---

## 6. Évolutions réglementaires récentes (2022-2026)

### Révision majeure de fin 2022

- L'arrêté du **7/12/2022** rebascule le titre FPA d'une structure 2 CCP
  (REAC 2018 V06) à **4 CCP / 13 CP**, intègre explicitement la multimodalité
  (présentiel/distanciel), le tutorat à distance, et la RSE.
- Arrêté correctif du **24/01/2023**.
- Enregistrement RNCP au **29/04/2023** pour 5 ans.

### VAE — réforme de la loi du 21 décembre 2022 (loi n°2022-1598)

- Création du portail unique **France VAE** (`vae.gouv.fr`).
- **VAE inversée** : expérimentation de 3 ans (1er mars 2023 → 28 février 2026)
  permettant un contrat de professionnalisation aboutissant à une VAE.
  Cadre : décret n°2023-408 du 26/05/2023, arrêté du 26/06/2023.
- **À VÉRIFIER au 6/5/2026** : l'expérimentation de la VAE inversée est
  techniquement échue depuis le 28/02/2026. Vérifier si elle a été prorogée,
  pérennisée ou abandonnée — c'est un point chaud à intégrer dans la veille
  ANDREA, particulièrement pour le tier Maître.
- Suppression de la condition d'1 an d'expérience minimum pour accéder à la
  VAE (décret VAE 2023).

### Aucune révision du REAC FPA détectée pour 2024-2025-2026

Le millésime en vigueur reste **V07 du 21/12/2022**.

---

## 7. Modalités d'accès

### Voies d'accès officielles

1. Formation continue (parcours majoritaire)
2. Formation initiale (rare)
3. Apprentissage / contrat de professionnalisation
4. VAE (via France VAE)
5. Validation par blocs / CCP (chaque CCP capitalisable 5 ans indépendamment)

### Public cible

- Personnes en reconversion vers les métiers de la formation
- Formateurs occasionnels souhaitant professionnaliser leur pratique
- Demandeurs d'emploi orientés vers le secteur formation/insertion
- Travailleurs indépendants (pré-requis de fait pour Qualiopi de qualité)

### Prérequis (pratique commune ; non opposable réglementairement)

- Niveau Bac (niveau 4) recommandé OU expérience professionnelle significative
- Maîtrise du français écrit/oral (B2 minimum de fait)
- Maîtrise des outils bureautiques et numériques de base

### Durée moyenne de formation

- **Parcours complet** : 600 à 1 000 heures (typiquement **815 h** :
  500 h centre + 315 h entreprise)
- **Stage en entreprise obligatoire** : minimum 210 h pour CCP1+CCP2,
  généralement 315 h sur le parcours complet
- **Calendrier** : 6 à 12 mois selon le rythme

---

## 8. Documents à embarquer dans le RAG d'ANDREA

| Document | URL pérenne | Format | Réutilisabilité |
|---|---|---|---|
| Fiche RNCP 37275 | https://www.francecompetences.fr/recherche/rncp/37275/ | HTML | Donnée publique réutilisable, citation obligatoire |
| Arrêté 7/12/2022 | https://www.legifrance.gouv.fr/jorf/id/JORFTEXT000046751421 | HTML | **Etalab 2.0** (réutilisation libre, y compris commerciale) |
| Arrêté 24/1/2023 modificatif | https://www.legifrance.gouv.fr/jorf/id/JORFTEXT000047074402 | HTML | Etalab 2.0 |
| Arrêté 22/12/2015 (cadre général TP) | https://www.legifrance.gouv.fr/loda/id/JORFTEXT000031733810 | HTML | Etalab 2.0 |
| REAC FPA V07 | travail-emploi.gouv.fr / banque AFPA | PDF | Annexe d'arrêté → public, citation requise. **PROBABLE Etalab 2.0** |
| RE FPA V07 | Idem | PDF | Idem |
| Fiche France VAE | https://vae.gouv.fr/certifications/84ca34d2-991e-4037-9f52-30dab61f9d26/ | HTML | Données publiques |
| Guide DP du Ministère (juin 2016) | travail-emploi.gouv.fr | PDF | Etalab 2.0 (à confirmer) |
| Pages "Jury du titre professionnel" | https://www.jurytitreprofessionnel.fr/ | HTML | Site officiel DGEFP |

### Recommandations licence pour ANDREA (produit commercial)

- **Arrêtés et codes** (Légifrance) : réutilisation libre sous Licence Ouverte
  Etalab 2.0, citation requise.
- **REAC/RE** (Ministère du Travail) : présomption Etalab mais **mention
  « Source : Ministère du Travail – DGEFP, REAC FPA V07 du 21/12/2022 »
  obligatoire** par sécurité.
- **Ne pas reproduire intégralement** un référentiel dans un PDF marketing
  public sans mention claire ; l'usage RAG (embeddings + extraits cités à
  l'utilisateur) est le mode le plus sûr juridiquement.

---

## 9. Stratégie de chunking pour le RAG

Cible : ~800-1500 chunks pour ~200 pages de référentiel.

### Règles de chunking proposées

- **Granularité primaire** : 1 chunk = 1 compétence professionnelle (CP), avec
  contexte CCP de rattachement
- **Granularité secondaire** : sous-sections (savoirs, savoir-faire,
  savoir-être, critères de performance, modalités d'évaluation) chunked
  séparément avec lien vers le CP parent
- **Granularité tertiaire** : épreuves d'évaluation (MSP, Entretien Tech,
  Entretien Final, DP) chunked individuellement
- **Métadonnées par chunk** :
  ```json
  {
    "chunk_id": "reac-v07-cp3-savoirs",
    "source_doc": "REAC V07 21/12/2022",
    "ccp": "CCP1",
    "cp": "CP3",
    "section": "Savoirs associés",
    "citation_human": "REAC V07 (21/12/2022) — CCP1 — CP3 — Savoirs associés",
    "url": "https://www.francecompetences.fr/recherche/rncp/37275/"
  }
  ```
- **Taille cible** : 200-500 tokens par chunk
- **Overlap** : minimal (50 tokens) car la structure est sémantique, pas
  narrative
- **Embedding** : bge-m3 (1024 dim, multilingue, contexte 8192 OK)

### Pipeline d'ingestion

```
1. Téléchargement PDFs officiels (manuel ou via script si URL stables)
2. Extraction texte (pdf-extract, fallback OCR si scanné)
3. Parsing structuré (regex sur intitulés "CCP", "CP", "Savoirs", etc.)
4. Génération JSONL `chunked/referentiel-fpa-v07.jsonl`
5. Embarquement dans `apps/desktop/src-tauri/resources/`
6. Au premier lancement : ingestion en sqlite-vec local
```

---

## 10. Risques juridiques — nom, marque, mentions

### « Titre Professionnel » / « FPA »

- **« Titre professionnel »** est une **catégorie juridique** définie aux
  articles **L.6113-1 et suivants du Code du travail**, et non une marque INPI
  déposée par l'État. Son usage est encadré par la qualité de l'objet : on ne
  peut affirmer délivrer un titre professionnel que si l'on est certificateur
  ou habilité par le certificateur.
- **« Formateur Professionnel d'Adultes »** est l'intitulé officiel d'une
  certification d'État. Aucune marque INPI privée connue ne s'oppose à son
  usage descriptif. **À VÉRIFIER** par recherche d'antériorité INPI avant
  exploitation commerciale du nom dans une marque ou nom de domaine.
- **« FPA »** est un sigle d'usage courant non protégé en tant que tel.

### Mentions obligatoires pour ANDREA (produit privé qui « prépare » au TP-FPA)

ANDREA est un **outil d'accompagnement, pas un organisme certificateur**.
Mention recommandée à intégrer dans :

- L'écran de bienvenue
- Le footer permanent de l'application
- Les CGV
- La page Gumroad

> *« ANDREA est un outil d'accompagnement à la préparation du Titre
> Professionnel Formateur Professionnel d'Adultes (RNCP n°37275). ANDREA n'est
> pas un organisme certificateur ni un organisme de formation habilité.
> Le titre est délivré par le Ministère du Travail. ANDREA est édité par
> Julien PERROT, indépendamment de l'AFPA et du Ministère du Travail. »*

### Précédents commerciaux

Très nombreux : Dunod et ESF Sciences Humaines publient des ouvrages
préparatoires ; des dizaines d'organismes privés (CCI, GRETA-CFA, EI Groupe,
IDEV, Skill and You, etc.) commercialisent des préparations. L'usage descriptif
(« préparation au titre FPA », « réussir son FPA ») est **bien établi et non
litigieux**, à condition de respecter le point ci-dessus.

---

## 11. Zones d'incertitude résiduelles à lever

1. **Libellé exact des 13 CP** — la formulation officielle doit être recopiée
   mot pour mot depuis la fiche France Compétences (les WebFetch automatisés
   ont renvoyé 403 dans cette session).
2. **Découpage exact des durées d'épreuves** B/C/D — vérifier sur le RE V07
   officiel.
3. **Statut Etalab effectif des PDF REAC/RE** — la pratique du Ministère est
   Etalab par défaut, mais une confirmation sur la page de téléchargement est
   requise.
4. **Statut au 6/5/2026 de la VAE inversée** (échue le 28/2/2026 sauf
   prorogation) — point réglementaire chaud.
5. **Existence d'un CCS attaché au TP-FPA** — non détecté dans les sources,
   *probable absence*, à confirmer sur la fiche RNCP.

---

## 12. Sources

- [France Compétences — RNCP37275](https://www.francecompetences.fr/recherche/rncp/37275/)
- [France VAE — TP FPA](https://vae.gouv.fr/certifications/84ca34d2-991e-4037-9f52-30dab61f9d26/)
- [Légifrance — Arrêté du 7 décembre 2022](https://www.legifrance.gouv.fr/jorf/id/JORFTEXT000046751421)
- [Légifrance — Arrêté du 24 janvier 2023](https://www.legifrance.gouv.fr/jorf/id/JORFTEXT000047074402)
- [Légifrance — Arrêté du 22 décembre 2015](https://www.legifrance.gouv.fr/loda/id/JORFTEXT000031733810)
- [REAC FPA V07 (copie miroir)](https://formanov.com/wp-content/uploads/REAC_FPA_V07_21122022.pdf)
- [RE FPA V07 (copie miroir)](https://formanov.com/wp-content/uploads/REV2_FPA_V07_21122022.pdf)
- [Site officiel Jury du titre professionnel](https://www.jurytitreprofessionnel.fr/)
- [Centre Inffo — VAE inversée](https://www.centre-inffo.fr/site-droit-formation/actualites-droit/mise-en-place-de-la-vae-inversee)
- [Schola Ingénierie — Évolution TP FPA 2023](https://schola-ingenierie.fr/evolution-du-tp-formateur-professionnel-dadultes/)
- [Educentre — REAC/RC/RE FPA](https://educentre.fr/articles/reac-rc-re-RNCP37275-formateur-professionnel-dadultes-hrw)
- [Patricia Barrett — Épreuves FPA](https://www.patriciabarrett.fr/certification-fpa-comprendre-enfin-les-preparations-et-epreuves-du-titre/)
- [Formanov — Évolutions 2023 référentiels FPA](https://formanov.com/evolution-2023-des-referentiels-du-titre-professionnel-fpa/)
- [Guide DP officiel du formateur (2016)](https://media.espace-competences.org/Actu/etat-orientation-vae-guide_formateur_dossier_professionnel.pdf)
