# 07 — Risques juridiques et techniques d'ANDREA

> Document produit le 6 mai 2026, à valider avant attaque de la phase 1.

---

## 1. Risques juridiques

### 1.1 Usage des appellations « Titre Professionnel » et « FPA »

| Risque | Mitigation |
|---|---|
| Confusion avec un statut de certificateur | Mention obligatoire en footer permanent, écran d'accueil, CGV, page Gumroad : « ANDREA est un outil d'accompagnement à la préparation du Titre Professionnel Formateur Professionnel d'Adultes (RNCP n°37275). ANDREA n'est pas un organisme certificateur ni un organisme de formation habilité. Le titre est délivré par le Ministère du Travail. ANDREA est édité par Julien PERROT, indépendamment de l'AFPA et du Ministère du Travail. » |
| Marque « ANDREA » antériorité INPI | Recherche INPI avant lancement Gumroad. Coût : 0 € (recherche en ligne) → 250 € (dépôt classes 9, 41, 42 si absence d'antériorité) |
| Communication trompeuse sur la réussite à l'épreuve | Clause CGV explicite : « ANDREA prépare au passage du titre. Aucune garantie de réussite ne peut être donnée. Le résultat dépend du jury, du dossier, de l'épreuve. » |

### 1.2 Propriété intellectuelle du référentiel

| Risque | Mitigation |
|---|---|
| Reproduction du REAC/RE en violation des droits d'auteur de la DGEFP | (a) Citation systématique « Source : Ministère du Travail – DGEFP, REAC FPA V07 du 21/12/2022 » à chaque extrait montré à l'utilisateur. (b) Pas de PDF ANDREA reproduisant le référentiel intégralement. (c) Usage RAG : extraits cités, pas distribués brut. (d) Documenter formellement l'usage Etalab 2.0 dans le README et les CGV |
| Évolution réglementaire qui rend le référentiel embarqué obsolète | Dans CGV : « Le référentiel embarqué est mis à jour à chaque release. ANDREA s'engage à publier une mise à jour dans les 60 jours suivant une évolution réglementaire majeure du titre. » |

### 1.3 PI du contenu pédagogique enrichissant

- Tout contenu créé par toi (anecdotes, études de cas, quiz, scénarios MSP) :
  copyright Julien PERROT 2026.
- Mention dans CGV : « Les contenus pédagogiques originaux d'ANDREA sont la
  propriété intellectuelle de Julien PERROT. Toute reproduction sans
  autorisation est interdite. »
- Données personnelles de l'utilisateur (DP en cours de rédaction, notes) :
  propriété de l'utilisateur, ANDREA n'a aucun droit dessus, aucune
  exploitation possible (100 % local).

### 1.4 RGPD

| Élément | Statut |
|---|---|
| Stockage 100 % local | ✓ Conforme par construction |
| Pas d'envoi de données à un tiers | ✓ Conforme |
| Email saisi à l'install | À justifier : strictement pour validation licence + support |
| Microservice license-server (côté Julien) | Stocke email + clé licence — registre des traitements à tenir |
| Politique de confidentialité | À rédiger (modèle disponible CNIL, ~1 page) |
| Droit à l'export et à l'effacement | ✓ Bouton « Exporter mes données » + « Tout effacer » dans Paramètres |
| DPO obligatoire ? | Non (pas de traitement à grande échelle de données sensibles) |

### 1.5 Licence des composants tiers

| Composant | Licence | Compatibilité produit propriétaire |
|---|---|---|
| Tauri 2.x | MIT/Apache-2.0 | ✓ OK |
| React, Tailwind, shadcn | MIT | ✓ OK |
| Ollama binaire | MIT | ✓ OK (sidecar) |
| Modèles LLM (Mistral) | Mistral Research / Apache-2.0 selon modèle | À vérifier modèle par modèle ; Mistral Small 3.2 = Apache-2.0 = OK |
| Whisper.cpp / whisper-rs | MIT | ✓ OK |
| **Piper (OHF-Voice)** | **GPL-3.0** | ⚠️ OK uniquement en sidecar process séparé. Obligation : fournir le code source de Piper aux utilisateurs sur demande |
| sqlite-vec | Apache-2.0 / MIT | ✓ OK |
| ed25519-dalek | BSD-3-Clause | ✓ OK |
| libsodium | ISC | ✓ OK |
| bge-m3 (modèle) | MIT | ✓ OK |
| Voix Piper françaises | CC-BY 4.0 ou similaire (à vérifier voix par voix) | ✓ OK avec attribution |
| Inter / Source Serif / JetBrains Mono | OFL 1.1 | ✓ OK |

**Action requise** :
- Documenter Piper GPL-3.0 dans `LICENSES/` du projet
- Page « Crédits » dans l'app listant tous les composants tiers
- Conseil juridique léger pour valider le sidecar GPL avant le lancement
  Gumroad

### 1.6 Distribution et signature

| Risque | Mitigation |
|---|---|
| Pas de Apple Developer ID → Gatekeeper bloque l'app | Souscription Apple Developer (~99 $/an) |
| Pas de cert Windows → SmartScreen alerte | Cert Authenticode (~250-400 $/an) ou EV (mieux mais ~600 $/an) |
| Notarization Apple aléatoire | Buffer 48h dans le pipeline release ; CI capable de relancer |

### 1.7 Conditions générales de vente

À rédiger avant lancement Gumroad. Points-clés :

- Identité de l'éditeur (Julien PERROT, RCS, SIRET)
- Description du produit (3 tiers, fonctionnalités)
- **Politique de remboursement** (Gumroad permet 7-30 jours selon réglage —
  obligation 14 jours en France)
- Limite de responsabilité (pas de garantie réussite, pas de garantie de
  performance sur tout matériel)
- Politique d'upgrade (paie la différence)
- Politique de mises à jour (engagement minimum 12 mois sur la version
  achetée)
- Juridiction (TGI Saint-Denis 974 ou TJ Paris)

---

## 2. Risques techniques

### 2.1 Plateforme / distribution

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Notarization Apple bloquée >16h (constaté 2026) | Moyenne | Élevé sur planning | Buffer 48h, CI capable de relance |
| Sidecar Ollama bloqué par sandbox macOS | Moyenne | Critique | Test CI macOS notarized dès l'étape 1, pas seulement en dev |
| `sqlite-vec.dylib` non signé bloqué par Gatekeeper | Moyenne | Critique | Signer l'extension avec le même Developer ID que l'app |
| WebView2 absent sur Windows custom | Faible | Élevé | Détection au premier lancement + redirection vers installeur Microsoft |
| Auto-updater casse à une release | Faible | Élevé | Tests CI sur upgrade v(N-1) → v(N) avant chaque release publique |

### 2.2 Performance / hardware

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| 8 Go RAM insuffisante pour modèle 24B | Élevée (config courante MacBook Air) | Élevé | Détection RAM → propose modèle 3B ou 7B avec mention claire « performances optimales à partir de 16 Go » |
| Latence STT/TTS trop élevée sur Windows CPU | Moyenne | Moyen | Modèle Whisper `tiny` sur fallback ; piper sur CPU déjà rapide |
| Premier lancement = 5-15 Go download | Certaine | Élevé sur conversion | UX d'attente travaillée, reprise sur coupure, message rassurant |
| Disque utilisateur saturé après installation | Moyenne | Moyen | Vérification espace disque préalable au téléchargement (≥ 20 Go libre minimum) |
| Throttling thermique sur MacBook Air en sessions longues | Moyenne | Moyen | Pas de mitigation parfaite. Documentation : « pour sessions >30 min, brancher l'alimentation » |

### 2.3 Sécurité

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Crack du système de licence | Élevée à terme | Faible (modèle de menace assumé : dissuasion, pas inviolabilité) | Email lié à la clé ; blocklist embarquée mise à jour via auto-updater ; pas de protection inviolable |
| Injection dans subprocess Ollama/Piper | Faible si sanitisation propre | Critique | Sanitiser tous les inputs ; pas d'eval ; arguments passés via array, jamais shell |
| Path traversal dans documents/ chiffrés | Faible | Moyen | Normalisation des paths, filename sanitization |
| Vol des données utilisateur (machine partagée) | Moyenne | Moyen | Chiffrement local des documents sensibles ; option passphrase backup |
| Fuite d'info dans logs | Faible si discipline | Moyen | Aucun log d'email, aucun log de contenu DP ; logs rotatifs avec niveau configurable |

### 2.4 Pédagogie / qualité du LLM

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Hallucination du référentiel | Élevée si sans RAG strict | Critique (atteinte à la valeur produit) | Temperature 0.3, RAG systématique, force-citation, garde-fou « je ne suis pas certain·e » |
| Posture trop scolaire / pas andragogique | Moyenne | Élevé sur ressenti utilisateur | Validation Julien sur 30 scénarios + 5 sessions longues avant gel v1 |
| Modèle qui dérive sur sessions très longues | Moyenne | Moyen | Gestion de contexte (résumé intermédiaire toutes les N tours), `num_ctx` 8192 |
| Modèle français qui glisse vers l'anglais | Moyenne sur petits modèles | Moyen | Prompt système strict en FR, exemples FR, modèles à dominante FR (Mistral) |
| Garde-fous franchis (rédige le DP, garantit le titre) | Moyenne sans tests | Critique | 100 essais adversariaux avant release ; système prompt avec règles ABSOLUES |

### 2.5 Dépendances et écosystème

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Ollama abandonné ou licence change | Faible mais réel | Critique | Architecture en couches (`LlmProvider` trait Rust) : bascule mistral.rs ou llama.cpp possible en 2-3 JP |
| Piper plus maintenu (rhasspy déjà archivé en 2025) | Moyenne | Moyen | Le fork OHF-Voice est actif ; alternative MeloTTS prête en plan B |
| sqlite-vec API change (encore en 0.1.x) | Moyenne | Faible | Adapter wrapper isolé ; bump majeur = effort circonscrit |
| Modèle Mistral Small 3.x retiré ou non maintenu | Faible | Moyen | Plusieurs modèles alternatifs équivalents ; le RAG compense largement |
| Modèles téléchargés sur ollama.com indisponibles | Faible | Élevé | Mirroir possible en self-hosting plus tard ; pour v1, accepté |

### 2.6 Données et migrations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Migration BDD échoue à mi-parcours | Moyenne | Critique | Backup automatique pré-migration + rollback transactionnel + UI de récupération |
| `.andrea-backup` corrompu à l'export | Faible | Moyen | Checksum SHA256 inclus dans le format ; vérification à l'import |
| Import backup d'une version antérieure | Probable | Faible | Migrations exécutées sur le snapshot avant import |
| Saisie utilisateur perdue au crash app | Moyenne | Moyen | Auto-save toutes les 30 s sur les éditeurs longs (notes, DP) |

---

## 3. Risques produit / commerciaux

| Risque | Mitigation |
|---|---|
| Faible adoption tier Découverte (99 €) | Page Gumroad soignée, vidéo de présentation, témoignage formateurs FPA en place |
| Demandes de remboursement | Période d'essai 14 jours obligatoire en FR ; produit fini et démontré |
| Support client non scalable | FAQ embarquée, guide PDF, vidéo onboarding ; canal email unique au début |
| Concurrence d'OF qui copient l'idée | Avantage de l'avance + de la PI Julien sur les contenus pédagogiques originaux |
| Évolution du titre FPA en 2027-2028 | Veille active + mise à jour ANDREA dans les 60 jours (clause CGV) |

---

## 4. Risques opérationnels (côté Julien)

| Risque | Mitigation |
|---|---|
| Charge de support utilisateur dépassante | Limiter le périmètre support en CGV (« réponse sous 5 jours ouvrés »), FAQ riche, doc soignée |
| Webhook Gumroad qui plante un dimanche soir | Monitoring (UptimeRobot gratuit), email d'alerte, fallback manuel sur 24h |
| Perte des clés privées Ed25519 / Minisign | Stockage chiffré offline + backup chez un tiers de confiance + rotation possible (la clé publique est versionnée dans le binaire mais peut être complétée par une seconde) |
| Perte de l'Apple Developer ID (mort, indisponibilité) | Compte familial Apple Developer + procuration au développeur backend |

---

## 5. Tableau récapitulatif — Top 10 des risques à surveiller

| # | Risque | Catégorie | Sévérité | Action immédiate |
|---|---|---|---|---|
| 1 | Notarization macOS instable + sidecars | Plateforme | Élevée | Tester pipeline complet dès étape 1 |
| 2 | Hallucination référentiel par le LLM | Pédagogie | Critique | RAG strict + temperature basse + garde-fou |
| 3 | Cible 8 Go RAM trop juste pour 24B | Hardware | Élevée | Auto-détection + double profil de modèle |
| 4 | Premier lancement (5-15 Go) abandonné | UX | Élevée | UX d'attente travaillée |
| 5 | Sidecar Piper GPL-3.0 et propriété ANDREA | Juridique | Moyen | Conseil juridique avant lancement |
| 6 | Crack du système de licence | Sécurité | Faible (assumé) | Modèle de menace clarifié |
| 7 | Migration BDD qui casse | Données | Critique | Backup auto + rollback transactionnel |
| 8 | ~~Évolution VAE inversée~~ — sans objet, module VAE retiré du périmètre | — | — | — |
| 9 | Prompt qui dérive en sessions longues | Pédagogie | Moyen | Résumé intermédiaire + tests longs |
| 10 | Webhook Gumroad qui plante | Opérationnel | Moyen | Monitoring + fallback manuel |
