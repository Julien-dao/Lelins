# Politique de confidentialité

ANDREA est conçue pour fonctionner **intégralement en local** sur votre
ordinateur. La présente politique détaille les très rares données que
l'éditeur conserve, les durées de conservation, et vos droits.

## 1. Responsable du traitement

**Julien PERROT**, éditeur d'ANDREA, La Réunion (974), France.
Contact : `support@andrea-formation.fr`.

## 2. Données traitées par l'éditeur

L'éditeur ne traite **que** les données suivantes, dans le cadre du
service commercial :

| Donnée | Source | Finalité | Base légale | Durée de conservation |
|---|---|---|---|---|
| Email d'achat | Webhook Gumroad | Émission et envoi de la clé de licence | Exécution contractuelle | 5 ans après le dernier achat |
| Clé de licence émise | Generation côté serveur | Gestion de la révocation et du support | Exécution contractuelle | 5 ans après le dernier achat |
| Email de contact support | Email entrant | Réponse aux demandes d'assistance | Exécution contractuelle | 3 ans après la dernière interaction |

L'éditeur **ne traite aucune autre donnée personnelle** de l'utilisateur :
ni les messages échangés avec ANDREA, ni les documents soumis, ni les
notes, ni la progression, ni l'historique de session. Ces données restent
**strictement** sur la machine de l'utilisateur, dans son dossier
système (`~/Library/Application Support/ANDREA/` sur macOS,
`%APPDATA%\ANDREA\` sur Windows).

## 3. Sous-traitants

| Sous-traitant | Donnée transmise | Finalité |
|---|---|---|
| **Gumroad, Inc.** (USA) | Email d'achat, données de paiement | Encaissement, facturation. Voir [politique Gumroad](https://gumroad.com/privacy) |
| **GitHub, Inc.** (USA, Microsoft) | Email d'achat optionnel pour les téléchargements de releases | Hébergement des binaires d'installation. Voir [politique GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) |

Aucun autre sous-traitant n'a accès à des données utilisateur.

## 4. Aucun envoi vers un service tiers depuis l'application

L'application **ne contacte aucun serveur** pendant l'utilisation
courante (conversation, évaluation, retrieval). Elle ne contacte un
serveur **qu'aux moments suivants** :

- Téléchargement initial des modèles d'IA depuis `ollama.com` et
  `huggingface.co` (au premier lancement uniquement).
- Vérification des mises à jour depuis `github.com/Julien-dao/Lelins`
  (au démarrage et toutes les 24h, désactivable dans Paramètres).

Ces requêtes ne transmettent **aucune donnée utilisateur**.

## 5. Vos droits

Conformément au Règlement Général sur la Protection des Données (RGPD)
et à la loi Informatique et Libertés modifiée, vous disposez des droits
suivants concernant les données traitées par l'éditeur :

- **Droit d'accès** aux données vous concernant.
- **Droit de rectification** des données inexactes.
- **Droit à l'effacement** ("droit à l'oubli"), sous réserve des
  obligations légales de conservation (facturation : 10 ans).
- **Droit d'opposition** au traitement.
- **Droit à la portabilité** des données.

Pour exercer ces droits, écrivez à `support@andrea-formation.fr`.

Pour les **données stockées localement** sur votre machine (la quasi-
totalité de votre usage d'ANDREA), vous disposez de :

- **L'export complet** via Paramètres → Données → Exporter
  (`.andrea-backup` chiffré).
- **L'effacement immédiat et complet** via Paramètres → Données → Tout
  effacer.

## 6. Sécurité

- Les documents personnels sensibles (Dossier Professionnel en cours)
  sont **chiffrés sur disque** (XChaCha20-Poly1305) avec une clé dérivée
  de votre profil.
- L'export `.andrea-backup` est **chiffré** (Argon2id + XChaCha20-Poly1305).
- Aucun mot de passe n'est requis par défaut, mais une **passphrase
  optionnelle** peut être définie pour les exports portables.

## 7. Réclamation

Si vous estimez vos droits non respectés, vous pouvez introduire une
réclamation auprès de la Commission Nationale de l'Informatique et des
Libertés ([cnil.fr](https://www.cnil.fr)).

## 8. Évolution

La présente politique peut être mise à jour pour refléter des évolutions
légales ou produit. La date d'effet figure ci-dessous ; en cas de
modification substantielle, vous serez notifié·e par email à l'adresse
d'achat.

**Date d'effet : `{date à compléter}`**.
