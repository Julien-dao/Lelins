// Mirror of `crates/andrea-referentiel/src/data.rs` for the UI side.
// Kept hand-synchronised in v1; once `ts-rs` is wired up (étape 1.A
// retro-fit), this file becomes generated.

export type Cp = { code: string; ccp: string; order: number; title: string };
export type Ccp = { code: string; title: string; summary: string };

export const ALL_CCP: readonly Ccp[] = [
  {
    code: "CCP1",
    title: "Concevoir et préparer la formation",
    summary:
      "Élaborer la progression pédagogique, concevoir le scénario d'une séquence et concevoir les activités d'apprentissage et d'évaluation, en intégrant la multimodalité.",
  },
  {
    code: "CCP2",
    title: "Animer la formation et évaluer les acquis",
    summary:
      "Animer un temps de formation collectif en présence et à distance, évaluer les acquis des apprenants et inscrire ses actes professionnels dans une démarche de RSE.",
  },
  {
    code: "CCP3",
    title: "Accompagner les apprenants en formation",
    summary:
      "Accueillir et co-construire le parcours, sécuriser les acquis, tutorer à distance et soutenir le développement professionnel des apprenants.",
  },
  {
    code: "CCP4",
    title: "Inscrire sa pratique dans une démarche qualité et RSE",
    summary:
      "Maintenir son expertise, analyser ses pratiques, respecter et promouvoir la réglementation (Qualiopi, RGPD, accessibilité) ainsi que les principes RSE.",
  },
];

export const ALL_CP: readonly Cp[] = [
  { code: "CP1", ccp: "CCP1", order: 1, title: "Élaborer la progression pédagogique d'une action de formation à partir d'une demande." },
  { code: "CP2", ccp: "CCP1", order: 2, title: "Concevoir le scénario pédagogique d'une séquence de formation, en intégrant la multimodalité." },
  { code: "CP3", ccp: "CCP1", order: 3, title: "Concevoir les activités d'apprentissage et d'évaluation des acquis, en intégrant la multimodalité." },
  { code: "CP4", ccp: "CCP2", order: 1, title: "Animer un temps de formation collectif en présence et à distance, en favorisant les interactions." },
  { code: "CP5", ccp: "CCP2", order: 2, title: "Évaluer les acquis d'apprentissage des apprenants." },
  { code: "CP6", ccp: "CCP2", order: 3, title: "Inscrire ses actes professionnels dans une démarche de responsabilité sociale, environnementale et professionnelle." },
  { code: "CP7", ccp: "CCP3", order: 1, title: "Accueillir les apprenants en formation et co-construire leur parcours." },
  { code: "CP8", ccp: "CCP3", order: 2, title: "Accompagner les apprenants dans la construction de leur parcours et la sécurisation de leurs acquis." },
  { code: "CP9", ccp: "CCP3", order: 3, title: "Tutorer les apprenants à distance." },
  { code: "CP10", ccp: "CCP3", order: 4, title: "Accompagner le développement professionnel des apprenants par la remédiation et l'individualisation." },
  { code: "CP11", ccp: "CCP4", order: 1, title: "Maintenir son expertise pédagogique et technique par la veille et le développement professionnel." },
  { code: "CP12", ccp: "CCP4", order: 2, title: "Analyser ses pratiques professionnelles." },
  { code: "CP13", ccp: "CCP4", order: 3, title: "Respecter et promouvoir la réglementation (Qualiopi, RGPD, accessibilité, droit de la formation) ainsi que les principes RSE." },
];

export const RNCP_CODE = "RNCP37275";
export const REFERENTIEL_VERSION = "REAC V07 21/12/2022";
export const VALID_UNTIL = "2028-04-29";
