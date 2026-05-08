# FAQ — ANDREA

## Le produit

### ANDREA délivre-t-elle le Titre Professionnel FPA ?
**Non.** ANDREA est un outil d'accompagnement privé. Le Titre Professionnel
est délivré par le Ministère du Travail via un jury habilité par la
DREETS / DDETS. ANDREA vous prépare au passage de l'épreuve, elle ne le
valide pas.

### Mes données sont-elles envoyées sur un serveur ?
**Non.** ANDREA fonctionne 100 % en local sur votre ordinateur. Aucun de
vos messages, documents, notes ou productions ne quitte votre machine.
Seul l'email saisi à l'achat sert à valider votre clé hors-ligne.

### Faut-il une connexion internet pour utiliser ANDREA ?
Internet n'est requis qu'au **premier lancement** (téléchargement du
moteur, ~5-14 Go) et **lors des mises à jour**. Toutes les sessions
quotidiennes fonctionnent hors-ligne.

### Quelle configuration matérielle est nécessaire ?
- **8 Go de RAM** au minimum (ANDREA charge alors le modèle léger
  Phi-4-mini 3.8 milliards de paramètres).
- **16 Go de RAM** recommandés (modèle Mistral Nemo 12 B).
- **24 Go de RAM ou plus** pour la meilleure expérience (Mistral Small
  3.2 24 B, qualité formateur).

ANDREA détecte automatiquement votre matériel et propose le bon modèle.

### macOS Apple Silicon ou Intel ?
Les deux sont supportés. ANDREA est plus rapide sur Apple Silicon (M1+).

## Licence

### J'ai perdu ma clé de licence, comment la retrouver ?
Connectez-vous à votre compte Gumroad : la clé est dans votre historique
d'achat ainsi que dans l'email de confirmation que vous avez reçu.

### Comment passer du tier Découverte au tier Pro (ou Maître) ?
Achetez le **produit d'upgrade** correspondant sur Gumroad. Vous payez la
différence entre les deux tiers (par exemple 100 € pour passer de
Découverte à Pro). Une nouvelle clé vous est envoyée par email. Saisissez-la
dans **Paramètres → Licence** : tous les nouveaux modules se déverrouillent
immédiatement, **sans aucune perte de vos données existantes**.

### J'utilise ANDREA sur deux ordinateurs, est-ce permis ?
Oui, votre licence vaut pour un usage personnel sur vos propres machines.
Pour transférer votre profil et vos documents d'une machine à une autre,
utilisez **Paramètres → Données → Exporter** puis **Importer** côté autre
machine.

### Comment installer ANDREA sur une nouvelle machine ?
1. Téléchargez et installez ANDREA sur la nouvelle machine.
2. Saisissez la même clé de licence + email.
3. Restaurez votre fichier `.andrea-backup` exporté précédemment.

## Pédagogie

### ANDREA peut-elle écrire mon Dossier Professionnel à ma place ?
**Non, ANDREA refuse explicitement** cette demande. Le DP est signé sur
l'honneur par le candidat·e — tout DP rédigé par un tiers est un manquement
à cette déclaration. ANDREA peut **relire**, **conseiller**, **structurer**,
**suggérer des reformulations**, mais le contenu doit venir de votre
expérience réelle.

### ANDREA peut-elle me garantir la réussite à l'épreuve ?
**Non.** ANDREA prépare au mieux, mais l'issue dépend du jury, du jour, et
de votre performance. Aucune garantie de résultat n'est ni n'a vocation à
être donnée.

### Pourquoi ANDREA pose-t-elle autant de questions au lieu d'expliquer
directement ?

C'est volontaire. ANDREA mobilise les principes andragogiques de Malcolm
Knowles : votre expérience est sa première ressource d'apprentissage. La
maïeutique (questionnement socratique) fait émerger votre savoir avant
qu'ANDREA ne complète. Si vous préférez un mode plus direct, demandez-le
explicitement.

### ANDREA peut-elle se tromper sur le référentiel ?
ANDREA est conçue pour **ne jamais inventer une référence** du référentiel.
Si elle n'est pas certaine à 100 %, elle vous le dit et vous oriente vers
[la fiche officielle France Compétences](https://www.francecompetences.fr/recherche/rncp/37275/).
Si vous repérez une erreur, signalez-la au support — la base de
connaissances est mise à jour à chaque release.

## Données et confidentialité

### Comment exporter mes données ?
**Paramètres → Données → Exporter** produit un fichier `.andrea-backup`
chiffré qui contient l'intégralité de vos données : profil, progression,
documents, historique de conversation, notes.

### Comment supprimer définitivement mes données ?
**Paramètres → Données → Tout effacer** supprime le contenu du dossier de
données ANDREA. Vous pouvez aussi désinstaller l'application puis
supprimer manuellement le dossier :

- macOS : `~/Library/Application Support/ANDREA/`
- Windows : `%APPDATA%\ANDREA\`

### ANDREA respecte-t-elle le RGPD ?
Oui. Comme aucune donnée ne quitte votre machine, ANDREA est conforme par
construction. Voir [Politique de confidentialité](./politique-confidentialite.md).

## Technique

### ANDREA est lente sur ma machine, que faire ?
- Vérifiez la **RAM disponible** (au moins 8 Go libres pendant l'usage).
- Branchez l'**alimentation** sur les MacBook : sans secteur, le CPU est
  bridé thermiquement après 20 minutes.
- Dans **Paramètres → Avancé**, vous pouvez forcer un modèle plus léger.

### Comment mettre à jour ANDREA ?
ANDREA vérifie automatiquement les mises à jour au démarrage. Une
notification discrète apparaît quand une nouvelle version est disponible.
Vous pouvez désactiver cette vérification dans Paramètres si vous préférez
gérer les mises à jour manuellement.

### Que faire si l'application refuse de se lancer ?
1. Redémarrez votre ordinateur.
2. Vérifiez que vous avez ≥ 8 Go de RAM disponible.
3. Vérifiez que vous avez ≥ 20 Go d'espace disque libre.
4. En dernier recours, contactez le support en joignant les logs :
   - macOS : `~/Library/Application Support/ANDREA/logs/`
   - Windows : `%APPDATA%\ANDREA\logs\`
