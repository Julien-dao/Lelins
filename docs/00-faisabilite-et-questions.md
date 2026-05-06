# 00 — Analyse de faisabilité et questions bloquantes

> Document produit le 6 mai 2026, à valider avant attaque de la phase 1.
> Synthétise les rapports de recherche `05-stack-technique.md` et `04-referentiel-fpa.md`.

---

## TL;DR

**Faisable, ambitieux, à recadrer sur trois points avant de coder :**

1. **Le titre FPA est structuré en 4 CCP / 13 compétences professionnelles**, pas 2 CCP.
   Le brief reflète l'ancienne version (REAC V06, 2018). La version en vigueur est le
   REAC V07 du 21/12/2022, valide jusqu'au 29/04/2028. **Impact** : toute la
   progression pédagogique, le RAG, le système prompt, l'UI « Mes cours » et la
   structure du Dossier Professionnel doivent s'aligner sur 4 CCP / 13 CP.

2. **Le hardware plancher (8 Go de RAM) est une contrainte dure** pour faire tourner
   un LLM en français avec finesse pédagogique. Il faut décider la cible :
   tout-public 8 Go (modèle 3-7B, qualité moyenne) ou 16 Go+ (modèle 12-24B,
   qualité formateur). Ma recommandation : **double profil avec auto-détection**.

3. **La friction d'installation est inévitable** : 5-15 Go à télécharger au premier
   lancement (Ollama + modèle LLM + Whisper + voix Piper + référentiel). Doit être
   absolument scénarisé en onboarding pour ne pas perdre l'utilisateur.

Le reste de la stack (Tauri 2.6, sqlite-vec, Whisper, Piper, Ed25519) est mature et
sans surprise majeure en mai 2026.

---

## 1. Faisabilité technique globale

| Volet | Verdict | Confiance |
|---|---|---|
| Tauri 2.6 + React + Tailwind + shadcn | Mature, sans risque | ★★★★★ |
| Sidecar Ollama (binaire MIT) + ollama-rs | Pattern documenté, viable | ★★★★☆ |
| LLM français pédagogique offline | Faisable mais dépendant du hardware utilisateur | ★★★☆☆ |
| Whisper.cpp + whisper-rs 0.16 (FR small/medium) | Mature | ★★★★★ |
| Piper TTS (fork GPL-3.0 OHF-Voice) | Mature, GPL OK en sidecar | ★★★★☆ |
| sqlite-vec 0.1.9 + bge-m3 pour RAG ~1000 chunks | Largement sous-exploité, sub-100 ms | ★★★★★ |
| Licence Ed25519 + clé courte | Nécessite compromis (MAC tronqué, modèle de menace assumé) | ★★★☆☆ |
| Auto-update Tauri + GitHub Releases | Pattern documenté | ★★★★☆ |
| Notarization macOS + sidecars | Connu, mais pipeline fragile en 2026 | ★★★☆☆ |
| RAG sur référentiel FPA (Etalab 2.0) | Réutilisation libre avec citation | ★★★★☆ |

Aucun show-stopper. Les ★★★ correspondent à des points de friction maîtrisables
avec discipline d'ingénierie.

---

## 2. Faisabilité produit

| Volet | Verdict |
|---|---|
| Trois tiers Découverte / Pro / Maître activés par clé | Faisable proprement avec une seule app |
| Migration sans perte de données entre tiers | Triviale : tables présentes dès la v1, déverrouillage côté UI |
| Migrations BDD entre versions app | Pattern classique (refinery / sqlx_migrate / maison), sans risque |
| Export/import `.andrea-backup` chiffré | Faisable, format à figer (cf. `01-architecture.md`) |
| Multilingue ? | **Non requis pour la v1** (FPA = certification française, public francophone) |

---

## 3. Faisabilité commerciale (Gumroad)

| Volet | Verdict |
|---|---|
| Gumroad sait-il générer une clé Ed25519 par achat ? | **Non nativement.** Il faut un webhook + un microservice maison `andrea-license` |
| Politique d'upgrade payant la différence | Faisable via codes promos Gumroad ou produits dédiés "upgrade" |
| Bundle "Le Titre en main" | Produit séparé sur Gumroad |
| Anti-piratage léger | Email saisi à l'install + comparé au hash dans la clé : OK |

**À budgéter** : un petit serveur webhook Gumroad → générateur de licence (Vercel /
Railway / Cloudflare Workers selon préférence) avec base SQLite des licences
émises pour révocation et support.

---

## 4. Questions bloquantes (à trancher avant la phase 1)

### Référentiel pédagogique (priorité absolue)

1. **Confirmes-tu la bascule du brief sur 4 CCP / 13 CP** (REAC V07 21/12/2022) ?
   Ou as-tu une raison de rester sur l'ancienne structure 2 CCP ?
2. **Acceptes-tu de t'aligner sur la nomenclature officielle exacte** des 13
   compétences (CP1 à CP13), telle qu'elle apparaîtra sur la fiche France
   Compétences ? Je propose de la recopier mot pour mot lors du chargement du
   corpus RAG.
3. **Veux-tu intégrer la dimension VAE dès la v1 Découverte** (a minima en lecture
   du référentiel) ou strictement en v3 Maître ? Note : la VAE inversée
   expérimentale est échue depuis le 28/02/2026, statut de prorogation à
   surveiller.

### Cible matérielle

4. **Quelle config plancher tu garantis sur Gumroad ?** Trois options :
   - **A. Premium uniquement** : 16 Go RAM minimum, modèle Mistral Small 3.2 24B.
     Qualité maximale mais exclut les MacBook Air M1 8 Go et beaucoup de PC
     d'apprenants en reconversion.
   - **B. Universel dégradé** : 8 Go RAM minimum, modèle 3B (Phi-4-mini ou
     Llama 3.2 3B). Tourne partout mais qualité pédagogique limitée.
   - **C. Double profil auto-détecté** *(ma recommandation)* : auto-détection au
     premier lancement, propose le bon modèle, mention claire « performances
     optimales à partir de 16 Go ».

### Stratégie d'embarquement du LLM

5. **Confirmes-tu Ollama en sidecar** (mon choix par défaut) ou préfères-tu une
   intégration pure-Rust via `mistral.rs` (binaire plus lourd, compilation plus
   complexe, mais plus "silencieux" pour l'utilisateur) ?

### Voix et accessibilité

6. **Acceptes-tu Piper GPL-3.0 en sidecar** (le fork rhasspy a été archivé en
   octobre 2025, le mainteneur officiel est désormais OHF-Voice/piper1-gpl) ?
   En sidecar, la GPL n'oblige qu'à fournir le source de Piper, pas d'ANDREA.
   Alternative payante : voix premium SSML cloud (mais tu veux 100 % offline).

### Distribution

7. **Apple Developer ID actif ?** Notarization obligatoire pour macOS. Coût ~99 $/an.
8. **Certificat Windows Authenticode ?** ~250-400 $/an, sinon SmartScreen alerte
   les utilisateurs.
9. **GitHub Releases public ou privé ?** Pour l'auto-updater, privé fonctionne
   avec un token, mais public est plus simple à gérer.

### Marque et juridique

10. **Veux-tu déposer ANDREA à l'INPI** (marque + nom de domaine andrea-formation.fr
    ou similaire) avant le lancement Gumroad ?
11. **Le contenu pédagogique enrichissant le référentiel (anecdotes, études de cas,
    quizz) est-il rédigé par toi en propre** ou consigné dans une banque collective ?
    Important pour la PI et la mention « © Julien PERROT 2026 ».
12. **Mentions légales et CGV : as-tu un modèle existant** ou veux-tu que je
    propose une trame en phase 1 ?

### UX

13. **Tutoiement par défaut ou vouvoiement par défaut ?** (modifiable par
    l'utilisateur dans les paramètres). Je propose **vouvoiement par défaut** —
    plus respectueux du concept de soi de Knowles, plus professionnel ;
    l'utilisateur peut basculer.
14. **Voix par défaut** : femme ou homme ? (Tu peux proposer un échantillon
    pour chaque au premier lancement.)
15. **Prénom du formateur par défaut** : ANDREA, ou peut-on suggérer des prénoms
    français-ultramarins (Andréa, André, Anaëlle, Marie-Andrée…) pour
    l'identification ?

### Données

16. **Format `.andrea-backup` chiffré : passphrase utilisateur** ou clé dérivée
    automatiquement (option par défaut, plus simple) ? Je recommande
    **dérivation automatique** + option avancée passphrase pour qui veut.

---

## 5. Ce qui peut casser (risques opérationnels)

Liste détaillée dans `06-risques.md`. Top 5 :

1. **Notarization macOS instable** en 2026 (forums Apple Dev signalent des délais
   de 16h+ certains jours) — prévoir buffer dans le pipeline release.
2. **Sandbox macOS qui bloque silencieusement le sidecar** Ollama — testé en CI
   continue, sinon découverte tardive.
3. **Modèle LLM hallucine le référentiel** sans RAG strict — temperature basse,
   force-citation, fallback "je ne suis pas sûr, vérifiez sur France Compétences".
4. **Premier lancement long** (5-15 Go) avec utilisateur qui ferme l'app de
   frustration — UX d'attente travaillée, possibilité de fermer/reprendre.
5. **Dépendance externe Ollama** : si Ollama abandonne le projet ou change de
   licence, ANDREA doit pouvoir basculer sur llama.cpp direct ou mistral.rs.
   Architecture en couches abstraites (`llm/` trait Rust) pour permettre la
   bascule.

---

## 6. Pré-requis confirmés avant démarrage code

- [ ] Réponses aux questions 1-16 ci-dessus
- [ ] Validation de l'architecture (`01-architecture.md`)
- [ ] Validation de la direction artistique (`02-direction-artistique.md`)
- [ ] Validation du système prompt (`03-systeme-prompt-andrea-formateur-v1.md`)
- [ ] Récupération des PDFs officiels REAC V07 + RE V07 (téléchargement manuel
      sur travail-emploi.gouv.fr ou banque AFPA — je ne peux pas le faire de mon
      côté, les URLs renvoient 403 en accès direct)
- [ ] Apple Developer ID + certificat Windows si distribution finale visée
- [ ] Création repo Gumroad avec produits Découverte/Pro/Maître/Bundle
- [ ] Dépôt INPI marque "ANDREA" si tu choisis cette voie
