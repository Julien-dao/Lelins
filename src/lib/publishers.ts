import type { SocialAccount, SocialNetwork } from '../types';

export type PublishStatus = 'success' | 'needs-config' | 'cors-blocked' | 'error';

const PROXY_URL = 'http://127.0.0.1:8088';
let proxyAvailable: boolean | null = null;

export async function checkProxy(): Promise<boolean> {
  if (proxyAvailable !== null) return proxyAvailable;
  try {
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), 800);
    const res = await fetch(`${PROXY_URL}/_health`, { signal: ctrl.signal });
    clearTimeout(t);
    proxyAvailable = res.ok;
  } catch {
    proxyAvailable = false;
  }
  return proxyAvailable;
}

export function resetProxyCache() {
  proxyAvailable = null;
}

export interface PublishResult {
  accountId: string;
  network: SocialNetwork;
  status: PublishStatus;
  message: string;
  url?: string;
}

export interface PublishInput {
  account: SocialAccount;
  caption: string;
  file: File;
}

export const NETWORK_DOCS: Record<SocialNetwork, { label: string; doc: string }> = {
  'instagram-feed': {
    label: 'Instagram',
    doc: 'Graph API · compte Business/Creator, ID de compte IG, média hébergé sur URL publique',
  },
  'instagram-story': {
    label: 'Instagram Story',
    doc: 'Graph API · endpoint stories (vidéo ou image)',
  },
  'instagram-reel': {
    label: 'Instagram Reel',
    doc: 'Graph API · endpoint reels (vidéo uniquement)',
  },
  tiktok: {
    label: 'TikTok',
    doc: 'Content Posting API · app approuvée, scope video.upload',
  },
  'youtube-shorts': {
    label: 'YouTube Shorts',
    doc: 'YouTube Data API v3 · OAuth scope youtube.upload',
  },
  x: {
    label: 'X (Twitter)',
    doc: 'X API v2 · tier payant, proxy local recommandé (CORS)',
  },
  linkedin: {
    label: 'LinkedIn',
    doc: 'Marketing API · proxy local requis (CORS)',
  },
  facebook: {
    label: 'Facebook',
    doc: 'Graph API · ID de Page, endpoint /photos ou /videos',
  },
};

/**
 * Route un compte vers le publisher adapté.
 * Retourne toujours un PublishResult (pas d'exception non capturée).
 */
export async function publish(input: PublishInput): Promise<PublishResult> {
  const { account } = input;
  if (!account.token) {
    return result(account, 'needs-config', 'Token manquant.');
  }
  try {
    switch (account.network) {
      case 'youtube-shorts':
        return await publishYouTube(input);
      case 'instagram-feed':
      case 'instagram-story':
      case 'instagram-reel':
        return await publishInstagram(input);
      case 'facebook':
        return await publishFacebook(input);
      case 'tiktok':
        return await publishTikTok(input);
      case 'x':
        return await publishX(input);
      case 'linkedin':
        return await publishLinkedIn(input);
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    if (/Failed to fetch|NetworkError|CORS/i.test(msg)) {
      return result(
        account,
        'cors-blocked',
        `Requête bloquée par CORS. ${msg}. Un proxy local est nécessaire.`,
      );
    }
    return result(account, 'error', msg);
  }
}

function result(
  account: SocialAccount,
  status: PublishStatus,
  message: string,
  url?: string,
): PublishResult {
  return { accountId: account.id, network: account.network, status, message, url };
}

// ============================================================================
// YouTube : Data API v3, resumable upload. Fonctionne depuis le navigateur.
// Le token doit avoir le scope https://www.googleapis.com/auth/youtube.upload
// ============================================================================
async function publishYouTube({ account, caption, file }: PublishInput): Promise<PublishResult> {
  const metadata = {
    snippet: {
      title: caption.split('\n')[0].slice(0, 100) || 'Lelins Studio',
      description: caption,
      tags: Array.from(caption.matchAll(/#([\p{L}\p{N}_]+)/gu)).map((m) => m[1]),
      categoryId: '22',
    },
    status: { privacyStatus: 'private', selfDeclaredMadeForKids: false },
  };

  // Step 1: initiate resumable session.
  const init = await fetch(
    'https://www.googleapis.com/upload/youtube/v3/videos?uploadType=resumable&part=snippet,status',
    {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${account.token}`,
        'Content-Type': 'application/json; charset=UTF-8',
        'X-Upload-Content-Type': file.type || 'video/mp4',
        'X-Upload-Content-Length': String(file.size),
      },
      body: JSON.stringify(metadata),
    },
  );
  if (!init.ok) {
    const body = await init.text();
    return result(account, 'error', `Init YouTube échoué (${init.status}): ${body.slice(0, 200)}`);
  }
  const uploadUrl = init.headers.get('Location');
  if (!uploadUrl) {
    return result(account, 'error', 'Location header manquant de la réponse YouTube.');
  }

  // Step 2: upload bytes.
  const upload = await fetch(uploadUrl, {
    method: 'PUT',
    headers: { 'Content-Type': file.type || 'video/mp4' },
    body: file,
  });
  if (!upload.ok) {
    const body = await upload.text();
    return result(account, 'error', `Upload YouTube échoué (${upload.status}): ${body.slice(0, 200)}`);
  }
  const parsed = (await upload.json()) as { id?: string };
  if (!parsed.id) {
    return result(account, 'error', 'Réponse YouTube sans ID.');
  }
  return result(
    account,
    'success',
    `YouTube : brouillon privé créé. Ouvrez YouTube Studio pour publier.`,
    `https://studio.youtube.com/video/${parsed.id}/edit`,
  );
}

// ============================================================================
// Instagram (Graph API) — nécessite un compte Business/Creator lié à une Page FB.
// L'API n'accepte pas les octets directement : il faut une URL publique pour le média.
// On détecte si l'utilisateur a renseigné `mediaHost` (p.ex. un bucket qu'il gère)
// et on tente ; sinon on retourne 'needs-config' avec un message clair.
// ============================================================================
async function publishInstagram({ account, caption }: PublishInput): Promise<PublishResult> {
  if (!account.externalId) {
    return result(
      account,
      'needs-config',
      'IG Business Account ID manquant (ajoutez-le dans le compte).',
    );
  }
  if (!account.mediaHost) {
    return result(
      account,
      'needs-config',
      "Instagram Graph API exige une URL publique pour le média. Renseignez 'mediaHost' et hébergez le fichier avant publication.",
    );
  }
  // Ici, on suppose que l'utilisateur a uploadé le fichier à mediaHost/<filename>
  // via son propre système hors de cette app (workflow manuel ou script externe).
  const mediaUrl = account.mediaHost;
  const isVideo = /\.(mp4|mov|webm)$/i.test(mediaUrl);
  const createParams = new URLSearchParams();
  createParams.set('access_token', account.token);
  createParams.set('caption', caption);
  if (isVideo) {
    createParams.set('media_type', account.network === 'instagram-reel' ? 'REELS' : 'VIDEO');
    createParams.set('video_url', mediaUrl);
  } else {
    createParams.set('image_url', mediaUrl);
  }

  const createRes = await fetch(
    `https://graph.facebook.com/v19.0/${encodeURIComponent(account.externalId)}/media`,
    { method: 'POST', body: createParams },
  );
  if (!createRes.ok) {
    const body = await createRes.text();
    return result(
      account,
      'error',
      `IG création média échouée (${createRes.status}): ${body.slice(0, 200)}`,
    );
  }
  const { id: creationId } = (await createRes.json()) as { id: string };

  const publishParams = new URLSearchParams();
  publishParams.set('access_token', account.token);
  publishParams.set('creation_id', creationId);
  const publishRes = await fetch(
    `https://graph.facebook.com/v19.0/${encodeURIComponent(account.externalId)}/media_publish`,
    { method: 'POST', body: publishParams },
  );
  if (!publishRes.ok) {
    const body = await publishRes.text();
    return result(
      account,
      'error',
      `IG publication échouée (${publishRes.status}): ${body.slice(0, 200)}`,
    );
  }
  return result(account, 'success', 'Publication Instagram envoyée.');
}

// ============================================================================
// Facebook Graph API — photo directe supportée depuis le navigateur via multipart.
// ============================================================================
async function publishFacebook({ account, caption, file }: PublishInput): Promise<PublishResult> {
  if (!account.externalId) {
    return result(account, 'needs-config', 'Page ID Facebook manquant.');
  }
  const isVideo = file.type.startsWith('video/');
  const endpoint = isVideo
    ? `https://graph-video.facebook.com/v19.0/${encodeURIComponent(account.externalId)}/videos`
    : `https://graph.facebook.com/v19.0/${encodeURIComponent(account.externalId)}/photos`;

  const form = new FormData();
  form.append('access_token', account.token);
  if (isVideo) {
    form.append('source', file);
    form.append('description', caption);
  } else {
    form.append('source', file);
    form.append('caption', caption);
  }

  const res = await fetch(endpoint, { method: 'POST', body: form });
  if (!res.ok) {
    const body = await res.text();
    return result(account, 'error', `FB échec (${res.status}): ${body.slice(0, 200)}`);
  }
  const parsed = (await res.json()) as { id?: string; post_id?: string };
  return result(
    account,
    'success',
    `Facebook : publié (id ${parsed.post_id ?? parsed.id}).`,
  );
}

// ============================================================================
// TikTok Content Posting API — direct-post nécessite video.publish,
// ou bien inbox upload (FILE_UPLOAD) qui est moins restrictif. On tente inbox.
// ============================================================================
async function publishTikTok({ account, file }: PublishInput): Promise<PublishResult> {
  if (!file.type.startsWith('video/')) {
    return result(account, 'error', 'TikTok accepte uniquement la vidéo.');
  }

  // Step 1: init upload (inbox).
  const initRes = await fetch('https://open.tiktokapis.com/v2/post/publish/inbox/video/init/', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${account.token}`,
      'Content-Type': 'application/json; charset=UTF-8',
    },
    body: JSON.stringify({
      source_info: {
        source: 'FILE_UPLOAD',
        video_size: file.size,
        chunk_size: file.size,
        total_chunk_count: 1,
      },
    }),
  });
  if (!initRes.ok) {
    const body = await initRes.text();
    return result(account, 'error', `TikTok init échec (${initRes.status}): ${body.slice(0, 200)}`);
  }
  const initData = (await initRes.json()) as {
    data?: { upload_url?: string; publish_id?: string };
  };
  const uploadUrl = initData.data?.upload_url;
  if (!uploadUrl) {
    return result(account, 'error', 'TikTok : upload_url manquant.');
  }

  // Step 2: upload bytes.
  const up = await fetch(uploadUrl, {
    method: 'PUT',
    headers: {
      'Content-Type': file.type,
      'Content-Range': `bytes 0-${file.size - 1}/${file.size}`,
    },
    body: file,
  });
  if (!up.ok) {
    const body = await up.text();
    return result(account, 'error', `TikTok upload échec (${up.status}): ${body.slice(0, 200)}`);
  }

  return result(
    account,
    'success',
    `TikTok : vidéo dans l'Inbox. Terminez la publication dans l'app TikTok.`,
  );
}

// ============================================================================
// X (Twitter) — exige un proxy local (CORS strict). Routes via /x/ et /x-upload/.
// Endpoint: POST /2/tweets pour le tweet (texte), upload v1.1 pour le média.
// ============================================================================
async function publishX({ account, caption, file }: PublishInput): Promise<PublishResult> {
  const ok = await checkProxy();
  if (!ok) {
    return result(
      account,
      'cors-blocked',
      "Proxy local non détecté. Lancez `node proxy/server.mjs` puis réessayez.",
    );
  }

  // 1. Upload média (v1.1, INIT/APPEND/FINALIZE)
  const isVideo = file.type.startsWith('video/');
  const initParams = new URLSearchParams();
  initParams.set('command', 'INIT');
  initParams.set('total_bytes', String(file.size));
  initParams.set('media_type', file.type);
  if (isVideo) initParams.set('media_category', 'tweet_video');

  const initRes = await fetch(`${PROXY_URL}/x-upload/1.1/media/upload.json`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${account.token}`,
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: initParams,
  });
  if (!initRes.ok) {
    const body = await initRes.text();
    return result(account, 'error', `X INIT échec (${initRes.status}): ${body.slice(0, 200)}`);
  }
  const { media_id_string } = (await initRes.json()) as { media_id_string: string };

  // APPEND chunks (5 MB max each).
  const chunkSize = 5 * 1024 * 1024;
  let segment = 0;
  for (let off = 0; off < file.size; off += chunkSize, segment++) {
    const slice = file.slice(off, off + chunkSize);
    const form = new FormData();
    form.set('command', 'APPEND');
    form.set('media_id', media_id_string);
    form.set('segment_index', String(segment));
    form.set('media', slice);
    const ap = await fetch(`${PROXY_URL}/x-upload/1.1/media/upload.json`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${account.token}` },
      body: form,
    });
    if (!ap.ok) {
      const body = await ap.text();
      return result(account, 'error', `X APPEND échec (${ap.status}): ${body.slice(0, 200)}`);
    }
  }

  const finParams = new URLSearchParams();
  finParams.set('command', 'FINALIZE');
  finParams.set('media_id', media_id_string);
  const fin = await fetch(`${PROXY_URL}/x-upload/1.1/media/upload.json`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${account.token}`,
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: finParams,
  });
  if (!fin.ok) {
    const body = await fin.text();
    return result(account, 'error', `X FINALIZE échec (${fin.status}): ${body.slice(0, 200)}`);
  }

  // 2. Création du tweet
  const tweet = await fetch(`${PROXY_URL}/x/2/tweets`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${account.token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      text: caption.slice(0, 280),
      media: { media_ids: [media_id_string] },
    }),
  });
  if (!tweet.ok) {
    const body = await tweet.text();
    return result(account, 'error', `X tweet échec (${tweet.status}): ${body.slice(0, 200)}`);
  }
  const tweetData = (await tweet.json()) as { data?: { id?: string } };
  const id = tweetData.data?.id;
  return result(
    account,
    'success',
    `X : tweet publié.`,
    id ? `https://x.com/${account.handle}/status/${id}` : undefined,
  );
}

// ============================================================================
// LinkedIn — exige un proxy local. UGC posts API + media upload.
// account.externalId = URN de la personne ("urn:li:person:XXXX") ou de l'organisation.
// ============================================================================
async function publishLinkedIn({
  account,
  caption,
  file,
}: PublishInput): Promise<PublishResult> {
  if (!account.externalId) {
    return result(
      account,
      'needs-config',
      "URN auteur LinkedIn manquant (ex. 'urn:li:person:XXXX').",
    );
  }
  const ok = await checkProxy();
  if (!ok) {
    return result(
      account,
      'cors-blocked',
      "Proxy local non détecté. Lancez `node proxy/server.mjs` puis réessayez.",
    );
  }

  const isVideo = file.type.startsWith('video/');
  const recipe = isVideo
    ? 'urn:li:digitalmediaRecipe:feedshare-video'
    : 'urn:li:digitalmediaRecipe:feedshare-image';

  // 1. Register upload
  const reg = await fetch(`${PROXY_URL}/linkedin/v2/assets?action=registerUpload`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${account.token}`,
      'Content-Type': 'application/json',
      'X-Restli-Protocol-Version': '2.0.0',
    },
    body: JSON.stringify({
      registerUploadRequest: {
        owner: account.externalId,
        recipes: [recipe],
        serviceRelationships: [
          { identifier: 'urn:li:userGeneratedContent', relationshipType: 'OWNER' },
        ],
      },
    }),
  });
  if (!reg.ok) {
    const body = await reg.text();
    return result(account, 'error', `LinkedIn register échec (${reg.status}): ${body.slice(0, 200)}`);
  }
  const regData = (await reg.json()) as {
    value: {
      asset: string;
      uploadMechanism: {
        'com.linkedin.digitalmedia.uploading.MediaUploadHttpRequest': { uploadUrl: string };
      };
    };
  };
  const uploadUrl = regData.value.uploadMechanism[
    'com.linkedin.digitalmedia.uploading.MediaUploadHttpRequest'
  ].uploadUrl;
  const asset = regData.value.asset;

  // 2. Upload bytes (LinkedIn returns a direct CDN URL — proxy not needed here,
  //    but it may still be CORS-blocked. Try direct first, fall back via proxy.)
  let up = await fetch(uploadUrl, {
    method: 'POST',
    headers: { Authorization: `Bearer ${account.token}` },
    body: file,
  }).catch(() => null);
  if (!up || !up.ok) {
    return result(
      account,
      'error',
      'Upload LinkedIn échoué (CDN bloqué ?). Vérifiez les logs réseau.',
    );
  }

  // 3. Create UGC post
  const post = await fetch(`${PROXY_URL}/linkedin/v2/ugcPosts`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${account.token}`,
      'Content-Type': 'application/json',
      'X-Restli-Protocol-Version': '2.0.0',
    },
    body: JSON.stringify({
      author: account.externalId,
      lifecycleState: 'PUBLISHED',
      specificContent: {
        'com.linkedin.ugc.ShareContent': {
          shareCommentary: { text: caption },
          shareMediaCategory: isVideo ? 'VIDEO' : 'IMAGE',
          media: [
            {
              status: 'READY',
              media: asset,
            },
          ],
        },
      },
      visibility: { 'com.linkedin.ugc.MemberNetworkVisibility': 'PUBLIC' },
    }),
  });
  if (!post.ok) {
    const body = await post.text();
    return result(account, 'error', `LinkedIn post échec (${post.status}): ${body.slice(0, 200)}`);
  }
  return result(account, 'success', 'LinkedIn : publication envoyée.');
}
