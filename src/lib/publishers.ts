import type { SocialAccount, SocialNetwork } from '../types';

export interface PublishResult {
  accountId: string;
  network: SocialNetwork;
  status: 'queued' | 'needs-config' | 'error';
  message: string;
}

export interface PublishInput {
  account: SocialAccount;
  caption: string;
  file: File;
}

/**
 * Itération 1 : les publishers sont des stubs.
 * Chaque réseau nécessite une intégration OAuth + endpoints spécifiques qui
 * seront câblés en itération 2 en utilisant le token fourni par l'utilisateur.
 * Aucune requête réseau n'est effectuée ici.
 */
export async function publish({ account, caption }: PublishInput): Promise<PublishResult> {
  if (!account.token) {
    return {
      accountId: account.id,
      network: account.network,
      status: 'needs-config',
      message: 'Token manquant pour ce compte.',
    };
  }
  return {
    accountId: account.id,
    network: account.network,
    status: 'needs-config',
    message: `Prévu en itération 2. Légende : ${caption.length} car.`,
  };
}

export const NETWORK_DOCS: Record<SocialNetwork, { label: string; doc: string }> = {
  'instagram-feed': {
    label: 'Instagram',
    doc: 'Instagram Graph API · compte Business/Creator requis',
  },
  'instagram-story': {
    label: 'Instagram Story',
    doc: 'Instagram Graph API · endpoint stories',
  },
  'instagram-reel': {
    label: 'Instagram Reel',
    doc: 'Instagram Graph API · endpoint reels',
  },
  tiktok: {
    label: 'TikTok',
    doc: 'TikTok Content Posting API · app approuvée requise',
  },
  'youtube-shorts': {
    label: 'YouTube Shorts',
    doc: 'YouTube Data API v3 · OAuth 2.0',
  },
  x: {
    label: 'X (Twitter)',
    doc: 'X API v2 · tier payant pour le posting',
  },
  linkedin: {
    label: 'LinkedIn',
    doc: 'LinkedIn Marketing API · OAuth 2.0',
  },
  facebook: {
    label: 'Facebook',
    doc: 'Facebook Graph API · Page requise',
  },
};
