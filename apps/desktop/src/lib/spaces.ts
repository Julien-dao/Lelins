// Registry of every space exposed by ANDREA. The Découverte tier ships
// the four base spaces; Pro and Maître add more, displayed locked until
// the user upgrades (cf. docs/01-architecture.md §7).

export type Tier = "DECO" | "PRO" | "MAIT" | "BNDL";

export type SpaceId =
  | "chat"
  | "courses"
  | "progression"
  | "vault"
  | "settings"
  // Pro
  | "dp"
  | "msp"
  | "jury"
  // Maître
  | "cohort"
  | "qualiopi"
  | "scenarios";

export type Space = {
  id: SpaceId;
  label: string;
  iconName: string; // Lucide icon name
  tier: Tier;
  description: string;
};

export const SPACES: readonly Space[] = [
  {
    id: "chat",
    label: "Conversation",
    iconName: "MessageCircle",
    tier: "DECO",
    description: "Échangez avec ANDREA — texte ou voix.",
  },
  {
    id: "courses",
    label: "Mes cours",
    iconName: "BookOpen",
    tier: "DECO",
    description: "Référentiel FPA structuré en 4 CCP × 13 compétences.",
  },
  {
    id: "progression",
    label: "Ma progression",
    iconName: "LineChart",
    tier: "DECO",
    description: "Vue d'ensemble, jalons et préparation à l'épreuve.",
  },
  {
    id: "vault",
    label: "Mon coffre-fort",
    iconName: "Archive",
    tier: "DECO",
    description: "Vos notes et documents personnels (chiffrés localement).",
  },
  {
    id: "settings",
    label: "Paramètres",
    iconName: "Settings",
    tier: "DECO",
    description: "Profil, voix, avatar, licence, exports.",
  },
  {
    id: "dp",
    label: "Mon DP",
    iconName: "FileText",
    tier: "PRO",
    description: "Assistant à la constitution du Dossier Professionnel.",
  },
  {
    id: "msp",
    label: "Mises en situation",
    iconName: "Presentation",
    tier: "PRO",
    description: "Entraînement aux MSP avec scénarios.",
  },
  {
    id: "jury",
    label: "Simulateur de jury",
    iconName: "Users",
    tier: "PRO",
    description: "Simulations des trois épreuves de certification.",
  },
  {
    id: "cohort",
    label: "Mes apprenants",
    iconName: "UserCheck",
    tier: "MAIT",
    description: "Suivi de cohorte pour formateurs FPA en exercice.",
  },
  {
    id: "qualiopi",
    label: "Exports Qualiopi",
    iconName: "ScrollText",
    tier: "MAIT",
    description: "Génération PDF des évaluations pour archivage Qualiopi.",
  },
  {
    id: "scenarios",
    label: "Scénarios avancés",
    iconName: "Sparkles",
    tier: "MAIT",
    description: "Bibliothèque de scénarios pédagogiques avancés.",
  },
] as const;

const TIER_RANK: Record<Tier, number> = {
  DECO: 0,
  PRO: 1,
  MAIT: 2,
  BNDL: 2,
};

export function isUnlocked(spaceTier: Tier, userTier: Tier): boolean {
  return TIER_RANK[userTier] >= TIER_RANK[spaceTier];
}

export function tierLabel(tier: Tier): string {
  switch (tier) {
    case "DECO":
      return "Découverte";
    case "PRO":
      return "Pro";
    case "MAIT":
      return "Maître";
    case "BNDL":
      return "Le Titre en main";
  }
}

export function gumroadUrlForTier(tier: Tier): string {
  // Placeholder — real product slugs go here once the Gumroad shop is set
  // up (cf. docs/06-plan-etapes.md étape 7).
  switch (tier) {
    case "PRO":
      return "https://julien-perrot.gumroad.com/l/andrea-pro";
    case "MAIT":
      return "https://julien-perrot.gumroad.com/l/andrea-maitre";
    case "BNDL":
      return "https://julien-perrot.gumroad.com/l/andrea-bundle";
    default:
      return "https://julien-perrot.gumroad.com";
  }
}
