# 05 — Recherche stack technique (mai 2026)

> Document produit le 6 mai 2026, basé sur recherche web active. Tous les
> numéros de version sont à jour à cette date.

---

## 1. Tauri

**Recommandation : Tauri 2.6.0** (branche stable 2.x). Stack mature pour app
desktop offline, framework agnostique côté frontend (React+Tailwind+shadcn OK).

- Plugins officiels confirmés et stables (`plugins-workspace`) :
  `tauri-plugin-updater`, `tauri-plugin-sql` (sqlx), `tauri-plugin-store`,
  `tauri-plugin-shell`, `tauri-plugin-fs`, `tauri-plugin-dialog`.
- Compatibilité multi-cibles : Apple Silicon (`aarch64-apple-darwin`),
  Intel macOS (`x86_64-apple-darwin`), Windows (`x86_64-pc-windows-msvc`).
- **Sidecar v2** : suffixe target-triple obligatoire dans le nom du binaire
  (changement vs v1, piège fréquent).
- Taille typique 2026 : installeur **2-10 Mo** (vs 80-200 Mo Electron),
  RAM ~50 Mo (vs 120 Mo+).

### Alternatives écartées

- **Electron** : taille incompatible avec budget UX (≥80 Mo installeur).
- **Wails** : Go, écosystème plus restreint que Rust+Tauri.

### Pièges

- Breaking change v2.6 : code de manipulation HTML dans `tauri-utils` derrière
  feature flag.
- Breaking change v2.3 : update wry 0.50 + objc2 0.6 casse `with_webview` API.
- macOS notarization : **hardened runtime obligatoire**
  (`"hardenedRuntime": true` dans `tauri.conf.json`). Le sidecar via
  `externalBin` casse la notarization si pas signé individuellement
  (issue tauri#11992). Service Apple parfois bloqué >16h en 2026
  (forums Apple Developer fév. 2026).
- WebView2 sur Windows : présent par défaut Win10 1903+ via Edge, à vérifier
  sur installs custom.

### Sources

- https://v2.tauri.app/release/tauri/v2.6.0/
- https://github.com/tauri-apps/plugins-workspace
- https://v2.tauri.app/distribute/sign/macos/
- https://github.com/tauri-apps/tauri/issues/11992

---

## 2. Ollama et modèles français

**Recommandation : Ollama 0.23.1** (testé largement en mai 2026).

### Modèles français privilégiés (pédagogie + conversation longue)

| Hardware | Modèle conseillé | Quantisation | Performance attendue |
|---|---|---|---|
| MacBook Air M1 8 Go | Phi-4-mini 3.8B / Llama 3.2 3B / Ministral-3B | Q4_K_M | 15-20 tok/s, marge mémoire serrée |
| MacBook M2 16 Go | Mistral Nemo 12B / Llama 3.1 8B / Qwen2.5 7B | Q4_K_M | confortable |
| Mac 32 Go / PC RTX | **Mistral Small 3.2 24B** (français natif Mistral AI) | Q4_K_M | optimal pour ANDREA |
| PC i5 16 Go (CPU only) | Mistral 7B / Llama 3.1 8B / Qwen 2.5 7B | Q4_K_M | 5-10 tok/s acceptable |

**Mistral Small 3.2** est le modèle d'élection : améliore function-calling,
instruction-following, réduit les répétitions. Très bon français natif.

**Llama 3.3 70B** : performances proches de Llama 3.1 405B mais réclame ≥48 Go
VRAM — hors cible.

### Function-calling et structured outputs

Pris en charge nativement par Ollama (`format: "json"` ou JSON Schema complet).
Modèles compatibles : llama3.1, llama3.2, llama4, mistral-nemo, qwen2.5,
mistral-small3.2.

### Embeddings

- **`bge-m3`** (568M, 1.2 Go disque, 100+ langues, contexte 8192,
  dense+sparse+ColBERT) : recommandé pour RAG multilingue/français.
- Alternatives : `nomic-embed-text-v2-moe`, `mxbai-embed-large`
  (principalement anglais).

### Pièges

- 8 Go RAM = impossible de faire tourner 24B confortablement.
- Q4_K_M reste le sweet-spot qualité/taille. Q5/Q8 réservés aux machines
  confortables.
- **Ollama distribue son binaire sous MIT** (commercial bundling OK), mais
  l'app GUI Ollama est sous licence séparée — pour ANDREA, n'embarquer **que
  le binaire CLI/serve**.

### Sources

- https://computingforgeeks.com/ollama-models-cheat-sheet/
- https://ollama.com/library/mistral-small3.2:24b
- https://docs.ollama.com/capabilities/structured-outputs
- https://ollama.com/library/bge-m3
- https://github.com/ollama/ollama/blob/main/LICENSE

---

## 3. Embarquer Ollama dans une app utilisateur

**Recommandation** : binaire `ollama` (CLI/serve) en **sidecar Tauri** via
`externalBin`, lancé en `Command::spawn()`, communiquant en HTTP localhost
(port 11434). Côté Rust, utiliser **`ollama-rs` 0.3.4** (publié 12 février
2026, activement maintenu). Gestion du cycle de vie via `tauri-plugin-shell` +
identifier `shell:allow-spawn`.

### Alternatives sérieuses

1. **`mistral.rs`** (EricLBuehler) — moteur Rust natif basé sur Candle,
   multimodal (texte/vision/audio/embeddings). Pure Rust = pas de binaire
   externe, mais binaire final plus lourd et compilation plus délicate
   (CUDA/Metal).
2. **`candle`** (HuggingFace) — framework ML Rust minimaliste, multi-backend
   (CPU/CUDA/Metal). Bonne perf sur Apple Silicon, légèrement derrière
   llama.cpp en pure inference speed.
3. **llama.cpp** directement avec bindings Rust (`llama-cpp-2`) — meilleure
   perf brute, mais compilation native par cible.
4. **llamafile** (Mozilla) — exécutable unique multi-plateforme via
   Cosmopolitan Libc, intéressant pour distribution mais moins flexible pour
   app GUI.

### Pièges

- Tauri qui spawn Ollama peut **échouer silencieusement** sans message
  d'erreur (problème connu, vérifier permissions sandbox macOS).
- Sidecar nommage v2 : `ollama-aarch64-apple-darwin`,
  `ollama-x86_64-apple-darwin`, `ollama-x86_64-pc-windows-msvc.exe`.
- Le binaire Ollama pèse ~30-50 Mo seul ; les modèles (4-15 Go) doivent être
  téléchargés au premier lancement (gros UX point).
- Pour candle/mistral.rs : compile-time long, dépendances système (Metal/CUDA
  SDK) côté dev.

### Sources

- https://v2.tauri.app/develop/sidecar/
- https://crates.io/crates/ollama-rs
- https://github.com/EricLBuehler/mistral.rs
- https://github.com/huggingface/candle

---

## 4. Whisper.cpp

**Recommandation** : `whisper.cpp` (ggml-org) + bindings Rust **`whisper-rs`
0.16.0** (release 12 mars 2026, repo migré sur Codeberg).

### Modèles français recommandés (.bin GGML)

- **`small`** (244M, ~466 Mo) : bon compromis pour pédagogie
- **`medium`** (769M, ~1.5 Go) : si machine confortable
- `tiny`/`base` : insuffisants pour le français nuancé
- `large-v3` : >6 Go RAM, hors cible MacBook Air

### Performance

- M1 + `medium` : transcription d'un fichier 10 min ≈ 2-3 min (RTF ~0.25)
- M3/M4 : quasi temps-réel
- Windows CPU i5 : `small` viable, `medium` lent (RTF ~1)

### Pièges

- Repo `whisper-rs` migré sur **Codeberg** (lien GitHub historique en miroir).
- Model files .bin volumineux : prévoir téléchargement post-install plutôt
  que bundle.

### Sources

- https://codeberg.org/tazz4843/whisper-rs
- https://crates.io/crates/whisper-rs
- https://github.com/ggml-org/whisper.cpp

---

## 5. Piper TTS et alternatives

**Recommandation** : **`OHF-Voice/piper1-gpl`** (fork officiel Open Home
Foundation, **GPL-3.0**, releases avril 2026). Le repo `rhasspy/piper`
original a été **archivé le 6 octobre 2025**.

### Voix françaises libres disponibles (modèles ONNX)

- `fr_FR-siwis-medium` : référence courante, neutre, légèrement robotique
  mais intelligible
- `fr_FR-tom-medium` : voix masculine, qualité correcte
- `fr_FR-gilles-low` : qualité basse
- `fr_FR-upmc-medium` (jessica/pierre) : **bug connu de débit trop rapide**
  (issue HA #105819)
- `fr_FR-mls`/`mls_1840` : medium/low

### Latence

Très faible (<200 ms par phrase courante sur CPU moderne).

### Alternatives commerciales et locales

- **MeloTTS** (myshell-ai, MIT, multi-langue dont français) : utilisable
  commercialement et localement. Recommandé en 2e choix si Piper insuffisant.
- **OpenVoice V2** (myshell-ai, MIT depuis avril 2024, FR natif) : cloning
  de voix, MIT = commercial OK.
- **F5-TTS** : code MIT mais **poids modèle CC-BY-NC** → exclu pour usage
  commercial.
- **XTTS v2 / Coqui** : licence CPML non-commerciale uniquement.
  **Coqui AI a fermé en décembre 2025** ; impossible d'acheter une licence
  commerciale. **À éviter pour ANDREA commercial.**

### Pièges

- Piper est passé en GPL-3.0 dans le fork OHF — incompatible avec
  distribution propriétaire si lié statiquement. **Vérifier juridiquement** :
  Piper s'utilise en CLI/sidecar (process séparé) → la GPL n'oblige pas à
  ouvrir l'app, mais à fournir le source du binaire Piper. Acceptable dans
  la plupart des cas.
- Voix ONNX : ~60 Mo par voix, à packager.

### Sources

- https://github.com/OHF-Voice/piper1-gpl
- https://github.com/rhasspy/piper/blob/master/VOICES.md
- https://github.com/myshell-ai/MeloTTS
- https://huggingface.co/myshell-ai/OpenVoiceV2

---

## 6. sqlite-vec et RAG

**Recommandation** : **`sqlite-vec` 0.1.9** (release 31 mars 2026,
alpha 0.1.10 en cours). Production-ready dans le contexte de "fast enough"
embarqué. Pure C, MIT/Apache-2.0, fonctionne sur Linux/macOS/Windows
(et WASM/mobile).

- Pour ~1000 chunks de référentiel FPA (TP FPA RNCP37275, ~200 pages, chunks
  ~500 tokens) : largement sous-exploité. Index brute-force ou IVF, latence
  sub-100 ms triviale.
- Embarquement Tauri : **`tauri-plugin-rusqlite2` 2.2.4** qui supporte le
  chargement d'extensions dynamiques. Le plugin officiel `tauri-plugin-sql`
  (sqlx) ne supporte pas nativement `sqlite3_auto_extension` — limitation
  connue (issue plugins-workspace#2622). Avec rusqlite, charger via
  `db.load_extension("path/to/sqlite-vec.dylib")`.

### Alternatives

- **`sqlite-vss`** : **déprécié officiellement**, dépendances C++ (Faiss)
  lourdes, ne tourne pas sur Windows. À ne pas utiliser.
- **libSQL / Turso embedded** : extension du noyau SQLite, vector natif intégré
  récemment, mais écosystème Rust moins stable.
- **LanceDB embedded** : pure Rust, plus orienté gros volumes.

### Pièges

- Extension `.dylib`/`.so`/`.dll` à packager pour chaque cible (3 binaires).
  Permissions exécutables (chmod 755) requises.
- API encore en 0.1.x : breaking changes possibles sur la syntaxe `vec0`.
- macOS : sandbox/Gatekeeper peut bloquer le chargement d'une extension non
  signée — signer le `.dylib` avec le même Developer ID que l'app.

### Sources

- https://github.com/asg017/sqlite-vec
- https://github.com/tauri-apps/plugins-workspace/issues/2622
- https://crates.io/crates/tauri-plugin-rusqlite2

---

## 7. Licences Ed25519

**Recommandation** : **`ed25519-dalek` 2.2.0** (stable production), avec
option d'évolution vers `ed25519-dalek` 3.0.0-pre.6 (publié 4 février 2026).

### Format clé compact ANDREA

La signature Ed25519 brute fait **64 octets = 512 bits**, soit 103 caractères
en Crockford Base32 — beaucoup trop pour une clé saisissable.

**Solution adoptée** : payload 9 octets (4+4+1) signé puis tronqué en MAC sur
~28 bits. Sécurité réelle ≈ 28 bits (forge théorique en heures), mais :

1. La clé secrète serveur n'est jamais distribuée.
2. Le hash email lie la clé à un acheteur spécifique : un crack n'aide pas au
   piratage de masse.
3. Modèle de menace : **dissuasion contre partage occasionnel**, pas
   inviolabilité.

Format final : `ANDREA-DECO-XXXXX-XXXXX-XXXXX-XXXXX` (24 chars de payload
encodé en Base32 Crockford).

### Outils

- `data-encoding` (Base32) ou `base32` ; pour Crockford strict :
  `base32_clockwork`. Crockford exclut I/L/O/U pour lisibilité humaine.

### Sources

- https://crates.io/crates/ed25519-dalek
- https://www.crockford.com/base32.html
- https://en.wikipedia.org/wiki/EdDSA

---

## 8. Tauri Updater 2026

**Recommandation** : `tauri-plugin-updater` officiel + GitHub Releases comme
backend statique. Manifeste `latest.json` :

```json
{
  "version": "1.2.3",
  "notes": "...",
  "pub_date": "2026-05-06T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {"signature": "...", "url": "https://.../app-aarch64.app.tar.gz"},
    "darwin-x86_64":  {"signature": "...", "url": "https://.../app-x86_64.app.tar.gz"},
    "windows-x86_64": {"signature": "...", "url": "https://.../app_x64-setup.exe"}
  }
}
```

- Signature **obligatoire et non désactivable** : Minisign (Ed25519). Clé
  publique en clair dans `tauri.conf.json`, clé privée dans secret CI.
- Check fréquence recommandée : au démarrage + check passif toutes les 24h.
- UX : ne pas forcer l'update, proposer "Installer plus tard" sauf updates
  de sécurité.

### Pièges

- Cycle de release : prévoir GitHub Action pour générer et signer
  `latest.json` automatiquement.
- macOS : binaire de mise à jour doit être notarized aussi sinon Gatekeeper
  bloque.
- Pas de rollback intégré : prévoir conservation N-1 côté serveur.

### Sources

- https://v2.tauri.app/plugin/updater/
- https://thatgurjot.com/til/tauri-auto-updater/

---

## 9. Parsing PDF/DOCX/ODT en Rust

| Format | Choix recommandé | Notes |
|---|---|---|
| PDF texte natif | `pdf-extract` (basé lopdf) | Suffit pour 90 % des PDFs FPA |
| PDF inspection | `pdf-inspector` (firecrawl) | Pure Rust, lopdf, <200 ms |
| PDF scanné OCR | `tesseract-rs` | ~30 Mo binaire + 5 Mo `fra.traineddata`, sidecar |
| DOCX | `docx-rs` (PoiScript) | Lecture ; alternative récente `rdocx` (fév. 2026) |
| ODT | `litchi` ou parsing manuel ZIP+XML | Support Rust parent pauvre |
| XLSX/ODS | `calamine` | Pure Rust |

### Pièges

- `pdf-extract` échoue silencieusement sur PDFs avec CMap exotiques (typique
  PDFs scannés institutionnels français).
- Tesseract en français demande pack `fra.traineddata` (~5 Mo).
- ODT : robustesse XSD non stricte.

### Sources

- https://crates.io/crates/pdf-extract
- https://github.com/firecrawl/pdf-inspector
- https://github.com/PoiScript/docx-rs
- https://docs.rs/calamine/

---

## 10. Alternatives Ollama embarqué

**Recommandation** : **garder Ollama** en sidecar (MIT, redistribution OK,
écosystème modèles le plus large).

### Alternatives évaluées pour silent-embed commercial

- **GPT4All** : MIT (GPL pour le binaire complet à vérifier), bundle moteur
  llama.cpp + UI. Moins agile qu'Ollama.
- **LM Studio CLI** : **propriétaire**. Free for personal use, **commercial
  license requise** pour bundling commercial. À écarter sauf accord négocié.
- **Jan.ai** : AGPL-3.0 → contagion virale incompatible avec ANDREA propriétaire.
  À écarter.
- **llamafile** (Mozilla, Apache 2.0) : binaire unique multi-OS, idéal pour
  distribution silencieuse. Modèles préemballés. Moins flexible qu'Ollama.
- **mistral.rs** (MIT) : pure Rust, pas de binaire externe — option la plus
  "silencieuse" possible mais coût d'ingénierie supérieur.

### Verdict

Ollama reste le choix pragmatique. Architecture en couches (`LlmProvider`
trait Rust) pour permettre une bascule future si nécessaire.

---

## 11. Synthèse opérationnelle pour ANDREA

| Volet | Choix | Version |
|---|---|---|
| Framework desktop | Tauri | 2.6.x |
| Frontend | React + Tailwind + shadcn + Framer Motion | latest |
| LLM moteur | Ollama (sidecar binaire MIT) | 0.23.1 |
| LLM modèle FR (16+ Go RAM) | Mistral Small 3.2 24B | Q4_K_M |
| LLM modèle FR fallback (8-16 Go) | Mistral Nemo 12B / Llama 3.1 8B | Q4_K_M |
| LLM modèle FR mode léger (8 Go) | Phi-4-mini / Llama 3.2 3B | Q4_K_M |
| Embeddings | bge-m3 | latest |
| STT | whisper.cpp + whisper-rs | 0.16.0 |
| STT modèle FR | Whisper small | latest |
| TTS | Piper (OHF-Voice/piper1-gpl, sidecar GPL-3.0) | latest |
| TTS voix défaut | fr_FR-siwis-medium / fr_FR-tom-medium | — |
| RAG vector store | sqlite-vec | 0.1.9 |
| SQLite plugin Tauri | tauri-plugin-rusqlite2 | 2.2.4 |
| Licence | ed25519-dalek + Base32 Crockford | 2.2.0 |
| Updater | tauri-plugin-updater + GitHub Releases + Minisign | — |
| Parsing PDF | pdf-extract / lopdf | — |
| Parsing DOCX | docx-rs | — |
| Parsing ODT | litchi (avec fallback ZIP+XML) | — |
| Crypto | libsodium (xchacha20poly1305, argon2id) | — |
| Frontend state | Zustand | latest |

---

## 12. Risques majeurs identifiés

1. **8 Go RAM (M1 base)** = limitation dure : mode dégradé avec modèle 7B/3B.
2. **Notarization macOS** instable en 2026 + sidecars = buffer dans pipeline
   release.
3. **Piper GPL-3.0** : OK en sidecar, à valider juridiquement par toi.
4. **XTTS v2 inutilisable** commercialement post-fermeture Coqui.
5. **Téléchargement modèle au premier lancement** : 5-15 Go = friction UX
   inévitable.
