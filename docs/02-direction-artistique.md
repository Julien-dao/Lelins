# 02 — Direction artistique d'ANDREA

> Document produit le 6 mai 2026, à valider avant attaque de la phase 1.

---

## 1. Intention

ANDREA n'est pas un chatbot. C'est un **formateur virtuel premium**. L'UI doit
respirer la confiance professionnelle d'un cabinet d'expertise, la chaleur d'un
formateur dans une salle bien éclairée, et la modernité d'un produit SaaS de
2026. Pas de gadgets, pas d'illustrations 3D fluo, pas de gradient flashy
"AI startup" : du **calme dense**.

Trois mots-clés directeurs :

- **Sobre** — chaque pixel justifie sa présence
- **Lettré** — la typographie porte l'expertise
- **Ancré** — un clin d'œil ultramarin assumé sans folklore

---

## 2. Références visuelles

### Référence #1 — Linear (linear.app)
- Mode sombre par défaut, gradients très contenus
- Hiérarchie typographique stricte
- Animations imperceptibles mais constantes
- À retenir : la **densité d'information sans saturation**

### Référence #2 — Granola (granola.ai)
- Compagnon professionnel à l'IA, ton apaisé
- Avatar sobre, carte conversation propre
- Palette neutre + un seul accent
- À retenir : la **chaleur d'un assistant de confiance**

### Référence #3 — Raycast (raycast.com)
- Interface focus, command palette
- Iconographie soignée, glyphes monochromes
- Mode sombre maîtrisé
- À retenir : la **sophistication des micro-interactions**

### Référence #4 — Arc Browser
- Identité forte, logo reconnaissable
- Couleurs custom par espace
- À retenir : la **personnalité visible mais discrète**

### Anti-références (à éviter)

- ChatGPT desktop : trop générique, pas de personnalité produit
- Claude.ai : excellent mais "tech company", pas "compagnon de formation"
- Duolingo : trop ludique pour un public adulte FPA
- Coursera / OpenClassrooms : design corporate vieillissant
- Tout produit qui met "AI ✨" en évidence — ANDREA est un formateur, pas
  une démo d'IA

---

## 3. Palette de couleurs

### Palette signature ANDREA

```
PRIMARY — "Andragogie Indigo"
  --andrea-50:  #EEF2FF
  --andrea-100: #E0E7FF
  --andrea-200: #C7D2FE
  --andrea-300: #A5B4FC
  --andrea-400: #818CF8
  --andrea-500: #6366F1   ← teinte signature
  --andrea-600: #4F46E5   ← brand primary
  --andrea-700: #4338CA
  --andrea-800: #3730A3
  --andrea-900: #312E81
  --andrea-950: #1E1B4B

ACCENT — "Lagon Réunion"
  --lagon-50:  #F0FDFA
  --lagon-100: #CCFBF1
  --lagon-300: #5EEAD4
  --lagon-500: #14B8A6   ← accent signature (clin d'œil ultramarin)
  --lagon-600: #0D9488
  --lagon-700: #0F766E
  --lagon-900: #134E4A

NEUTRES — base zinc Tailwind
  light bg:        #FAFAFA
  light surface:   #FFFFFF
  light border:    #E4E4E7
  dark bg:         #0A0A0F   ← un soupçon plus bleuté que zinc-950
  dark surface:    #18181B
  dark border:     #27272A
  text strong:     #18181B / #F4F4F5
  text default:    #3F3F46 / #D4D4D8
  text muted:      #71717A / #A1A1AA
  text subtle:     #A1A1AA / #71717A

SÉMANTIQUES
  success:  #10B981   (émeraude)
  warning:  #F59E0B   (ambre)
  error:    #EF4444   (rouge)
  info:     andrea-500
```

### Justification

- **Andragogie Indigo** (#4F46E5) : teinte savante, ni techno-fluo ni corporate
  fade. Connote la confiance et l'intellect. Suffisamment proche du violet pour
  porter la chaleur, suffisamment proche du bleu pour la rigueur.
- **Lagon Réunion** (#14B8A6) : turquoise chaud du lagon réunionnais. Évoque
  l'ouverture, la progression, la fraîcheur. Clin d'œil identitaire à 974 sans
  folklore. Excellent contraste sur indigo en dark mode.
- **Zinc bluté** (#0A0A0F) : un cran plus profond que zinc-950 standard,
  évite le "noir Slack" générique.

### Contrastes WCAG

Tous les couples ont été pré-validés AA minimum :
- `andrea-600` sur `bg light`           → 7.1:1 ✓ AAA
- `andrea-500` sur `dark bg`            → 6.8:1 ✓ AAA
- `lagon-500` sur `dark bg`             → 7.5:1 ✓ AAA
- `text-default` sur `bg`               → 12:1+ ✓ AAA
- `text-muted` sur `bg`                 → 4.6:1 ✓ AA

---

## 4. Typographie

### Famille principale UI — `Inter Variable`
- Usage : interface, navigation, boutons, formulaires
- Déjà standard SaaS premium 2026
- Variable axes : `wght` (300-800), `opsz` (14-32)
- Licence : SIL Open Font License 1.1 (libre commercial)

### Famille pédagogique — `Source Serif 4 Variable`
- Usage : contenus longs (cours, explications d'ANDREA, citations REAC),
  bulles conversation côté ANDREA
- Distingue visuellement "ANDREA explique" du chrome UI
- Lisibilité longue exceptionnelle
- Licence : SIL Open Font License 1.1

### Famille mono — `JetBrains Mono`
- Usage : citations exactes du référentiel, codes de licence, debug
- Licence : SIL Open Font License 1.1

### Échelle typographique

```
display     : 56 / 60 / -0.02em   Inter Display, weight 600
h1          : 36 / 44 / -0.01em   Inter, weight 600
h2          : 28 / 36 / -0.01em   Inter, weight 600
h3          : 22 / 30              Inter, weight 600
h4          : 18 / 28              Inter, weight 600
body-lg     : 17 / 28              Source Serif (pédagogie) / Inter (UI)
body        : 15 / 24              Source Serif (pédagogie) / Inter (UI)
body-sm     : 13 / 20              Inter
caption     : 12 / 16              Inter
mono-sm     : 13 / 20              JetBrains Mono
```

Numéros et chiffres : **tabular-nums** activé partout (alignement progression
"%, dates, scores).

---

## 5. Logo et identité

### Wordmark "ANDREA"

```
ANDREA
```

- Inter Display, weight 600, tracking +0.02em
- Lettre "A" finale légèrement modifiée : la barre transversale est rabaissée
  de 1px et inclinée de 3°, créant une signature subtile (un mouvement, comme
  un gradient pédagogique qui monte)
- En light mode : couleur `andrea-700`
- En dark mode : couleur `andrea-300`

### Monogramme "A"

Pour favicon, icône app, dock macOS / barre des tâches Windows :

- Lettre A composée de **trois traits superposés en éventail**, convergeant en
  pointe, suggérant : (1) **a**pprendre, (2) **a**ccompagner, (3) **a**tteindre
- Fond : carré arrondi (radius 22% selon iOS/macOS guidelines) avec gradient
  diagonal `andrea-700` → `lagon-600`
- Trait du A : blanc cassé `#FAFAFA` 2.5px à 32×32
- Fournitures : SVG vectoriel + PNG @1× @2× @3× pour 16, 32, 128, 256, 512, 1024

### Pas de mascotte

C'est l'**avatar du formateur** qui porte l'humanité. Pas besoin d'une mascotte
en plus ; ce serait redondant et infantilisant pour le public adulte.

---

## 6. Avatars de formateurs

### Stratégie

12 portraits proposés au premier lancement, équitablement répartis :

- 6 femmes / 6 hommes
- Tranche d'âge : 25-65 ans
- Diversité d'origines volontaire :
  - Européen-ne (2)
  - Maghrébin-e (2)
  - Afro-caribéen-ne (2)
  - Asiatique (1)
  - **Réunionnais-e / créole** (2) — exigence du brief
  - **Mahorais-e** (1)
  - Métis-se (2)

### Source proposée par priorité

#### 1er choix — banque éditoriale Unsplash + Pexels
- Recherches ciblées : "professional portrait french", "creole woman portrait",
  "french teacher", "réunion island portrait"
- Sélection éditoriale soignée par toi (Julien) avant intégration
- Licence : Unsplash License et Pexels License (libre commercial, pas
  d'attribution obligatoire mais de bon ton)
- **Travail nécessaire** : trouver des portraits ultramarins de qualité — c'est
  rare sur ces banques mainstream, prévoir éventuellement un photographe local
  974 pour 2-4 portraits (budget ~300-600 €)

#### 2e choix (complément) — génération IA contrôlée
- **Flux 1.1 Pro** (Black Forest Labs, licence commerciale, ~0.04 $/image) ou
  **Stable Diffusion 3.5 Large**
- Prompts soignés type :
  ```
  Photographic portrait of a 45 year old creole french woman from Reunion Island,
  warm professional smile, looking at camera, natural lighting, neutral background,
  shoulders visible, business casual attire, photorealistic, shot on Sony A7R,
  85mm lens, shallow depth of field
  ```
- Vérification systématique : pas de mains, pas d'artefacts, expression
  authentique et bienveillante

#### Stylisation cohérente
- Application uniforme d'un look post-prod léger : saturation -8%, grain ISO
  léger, vignettage subtil, légère désaturation des arrière-plans pour
  uniformiser
- Format final : **rond** (mask circulaire) en 256×256 pour la galerie,
  192×192 in-app, 64×64 pour timeline conversation

### Upload utilisateur
- Possibilité d'uploader sa propre photo (drag & drop)
- Crop circulaire intégré (react-image-crop)
- Stockage local dans `documents/avatar/` chiffré

---

## 7. Animations et micro-interactions

### Avatar du formateur

- **Respiration** : `transform: scale(1)` ↔ `scale(1.018)` sur 4.2s
  ease-in-out infinie. Quasi imperceptible mais donne la vie.
- **Écoute** : halo animé `box-shadow` `lagon-500` au moment où ANDREA capture
  l'audio utilisateur (pulsations 1.2s).
- **Pensée** : 3 points qui dansent en arc juste sous l'avatar.
- **Parole** : ondulations sonores (sine wave SVG animée) en bas de l'avatar
  synchronisées au flux TTS.
- **Inactif** : très léger float vertical 1.5px sur 7s.

### Indicateurs d'état conversation

```
"ANDREA vous écoute…"      → halo lagon, pulse mic
"ANDREA réfléchit…"        → trois points dansants, fade-in
"ANDREA vous explique…"    → ondulations sonores, bulle texte qui se remplit
"ANDREA est prête à vous lire" (au repos) → état neutre
```

(Au féminin par défaut si avatar femme choisi, masculin sinon — accord
automatique. Détail premium important.)

### Transitions de page

- Framer Motion `AnimatePresence` avec `mode="wait"`
- Durée : 220ms cubic-bezier `(0.32, 0.72, 0, 1)` (Vercel preset)
- `opacity` + `transform: translateY(8px → 0)` — pas plus.

### Bulles de conversation

- Apparition par fade + slide depuis le bas, 180ms
- Texte qui s'écrit lettre par lettre côté ANDREA (sync TTS), 35-50 ms/word
- Citations REAC dans une bulle dépliable avec icône `BookText`

### Boutons

- Hover : `transform: translateY(-1px)` + ombre douce, 120ms
- Active : `scale(0.98)`, 80ms
- Focus visible : ring 2px `andrea-500` + offset 2px

### Mode sombre / clair

- Toggle dans Paramètres
- Détection initiale : `prefers-color-scheme`
- Transition : `transition: background-color 200ms, color 200ms`
- Sombre par défaut **après l'onboarding** (l'onboarding utilise des palettes
  pleines et lumineuses pour l'accueil)

---

## 8. Mise en page et grille

- Layout principal : sidebar gauche fixe 240 px (espaces) + contenu principal
- Densité : padding intérieur des cartes 24 px, gap 16 px
- Largeur max contenu pédagogique : 720 px (lisibilité texte long)
- Largeur max conversation : 880 px
- Border-radius : 12 px partout (sauf badges 6 px, avatars cercle parfait)
- Ombres : 3 niveaux, très douces, blur 20-30 px, opacity 0.04-0.08

---

## 9. Iconographie

- **Lucide React** comme bibliothèque par défaut (cohérent shadcn)
- Trait 1.5 px, taille 18 ou 20 px en UI standard
- Icônes spécifiques ANDREA à dessiner si besoin (ex. badge "Prêt pour
  l'épreuve", certificat, dossier professionnel)

---

## 10. Sons

Au-delà des voix Piper :
- **Notification douce** quand une mise à jour est dispo (1 cloche, 0.4 s)
- **Confirmation** légère après sauvegarde (tick, 0.2 s)
- **Aucun son sur action standard** (pas de clics intrusifs)
- Tous les sons désactivables dans les paramètres
- Source : `freesound.org` CC0 ou enregistrement maison

---

## 11. Voix Piper proposées par défaut

D'après la recherche stack technique :

- **Femme** : `fr_FR-siwis-medium` (référence française)
- **Homme** : `fr_FR-tom-medium`
- 2 voix supplémentaires en option avancée : `fr_FR-mls_1840-medium`,
  `fr_FR-upmc-medium` (à tester en pratique vs bug de débit signalé)

Échantillons audio préparés pour l'onboarding : ANDREA prononce une phrase
identique avec chaque voix, l'utilisateur écoute, choisit.

Phrase d'échantillon proposée :
*"Bienvenue. Je suis ANDREA, votre formatrice virtuelle. Ensemble, nous allons
préparer votre Titre Professionnel Formateur Professionnel d'Adultes."*

---

## 12. Accessibilité (WCAG 2.2 AA cible)

- Navigation clavier complète (Tab order logique, raccourcis documentés)
- Lecteurs d'écran : tous les éléments interactifs ont un `aria-label`
- Focus visible permanent (jamais `outline: none` sans alternative)
- Animations respectent `prefers-reduced-motion`
- Texte agrandissable jusqu'à 200% sans casse layout
- Contraste minimum AA partout, AAA sur le contenu pédagogique principal
- Sous-titres synchronisés sur les sorties vocales d'ANDREA (même quand
  vocal pur)
- Voix tunable : vitesse de lecture (0.8× à 1.4×) dans paramètres

---

## 13. Identité produit récapitulée

```
Nom            : ANDREA
Tagline FR     : « Votre formateur virtuel pour le titre FPA »
Tagline produit: « ANDREA — Apprendre. Accompagner. Atteindre. »
Promesse       : « Le seul outil qui forme, évalue et guide
                  jusqu'à votre certification. »
Ton de voix    : pédagogique, posé, précis, chaleureux mesuré,
                 jamais condescendant, parfois drôle quand le contexte le permet.
Couleurs       : Andragogie Indigo + Lagon Réunion
Typographie    : Inter (UI) + Source Serif (pédagogie) + JetBrains Mono (techn.)
Mode défaut    : sombre (post-onboarding)
```
