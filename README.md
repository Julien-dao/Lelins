# ANDREA

> Formateur virtuel local pour préparer le Titre Professionnel Formateur
> Professionnel d'Adultes (RNCP n°37275).

ANDREA est une application desktop **100 % offline** qui forme, évalue et
guide les apprenants vers la certification du titre FPA. Elle s'incarne en
un formateur virtuel expert en andragogie qui s'adapte au profil et au
rythme de chaque apprenant.

Édité par **Julien PERROT**, La Réunion (974) — produit indépendant non
affilié au Ministère du Travail ni à l'AFPA.

---

## État du projet

🚧 **Phase 0 — Cadrage et architecture (en cours, mai 2026)**

Aucune ligne de code applicatif écrite à ce stade. Les documents de stratégie
sont dans `docs/`.

| Document | Statut |
|---|---|
| [`BRIEF.md`](BRIEF.md) | ✅ Brief produit, source de vérité |
| [`docs/00-faisabilite-et-questions.md`](docs/00-faisabilite-et-questions.md) | ✅ Analyse + questions bloquantes |
| [`docs/01-architecture.md`](docs/01-architecture.md) | ✅ Architecture détaillée |
| [`docs/02-direction-artistique.md`](docs/02-direction-artistique.md) | ✅ DA UI/UX |
| [`docs/03-systeme-prompt-andrea-formateur-v1.md`](docs/03-systeme-prompt-andrea-formateur-v1.md) | ✅ Prompt système v1 |
| [`docs/04-referentiel-fpa.md`](docs/04-referentiel-fpa.md) | ✅ Cartographie FPA |
| [`docs/05-stack-technique.md`](docs/05-stack-technique.md) | ✅ Recherche stack 2026 |
| [`docs/06-plan-etapes.md`](docs/06-plan-etapes.md) | ✅ Plan d'étapes Phase 1 |
| [`docs/07-risques.md`](docs/07-risques.md) | ✅ Risques juridiques & techniques |
| [`docs/08-decisions.md`](docs/08-decisions.md) | ✅ Décisions actées (8 questions tranchées) |

Phase 1 — Développement ANDREA Découverte : sur feu vert.

---

## Stratégie commerciale (3 tiers, une seule app)

| Tier | Prix | Cible |
|---|---|---|
| ANDREA Découverte | 99 € | Conversation FPA + suivi de progression |
| ANDREA Pro | 199 € | + Évaluation documents, simulateur jury, MSP |
| ANDREA Maître | 299 € | + Suivi de cohorte, exports Qualiopi, scénarios avancés |
| Bundle « Le Titre en main » | 449 € (au lieu de 597 €) | Tout |

Politique d'upgrade : on paie la différence entre tiers. Aucune perte de
données entre tiers — la même installation s'enrichit par saisie d'une
nouvelle clé de licence.

---

## Stack technique cible (mai 2026)

- **Desktop** : Tauri 2.6 (Rust + WebView)
- **Frontend** : React + Tailwind + shadcn + Framer Motion
- **LLM local** : Ollama (sidecar MIT) + Mistral Small 3.2 (FR natif)
- **STT** : whisper.cpp + whisper-rs 0.16
- **TTS** : Piper (OHF-Voice, GPL-3.0 en sidecar)
- **RAG** : sqlite-vec 0.1.9 + bge-m3 embeddings
- **Licence** : Ed25519 + Base32 Crockford (cf. `docs/01-architecture.md`)
- **Distribution** : GitHub Releases + auto-updater Tauri

---

## Mention obligatoire

> ANDREA est un outil d'accompagnement à la préparation du Titre Professionnel
> Formateur Professionnel d'Adultes (RNCP n°37275). ANDREA n'est pas un
> organisme certificateur ni un organisme de formation habilité. Le titre est
> délivré par le Ministère du Travail. ANDREA est édité par Julien PERROT,
> indépendamment de l'AFPA et du Ministère du Travail.

---

## Licence

Propriétaire — © Julien PERROT 2026. À détailler avant lancement Gumroad.
