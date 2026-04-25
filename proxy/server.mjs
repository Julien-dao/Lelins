#!/usr/bin/env node
/**
 * Lelins Studio — proxy local pour les API à CORS strict (X, LinkedIn).
 *
 * Usage :
 *   node proxy/server.mjs            # écoute sur 127.0.0.1:8088
 *   PORT=9000 node proxy/server.mjs  # autre port
 *
 * Sécurité : écoute uniquement sur 127.0.0.1, autorise uniquement http://localhost:5173
 * et http://127.0.0.1:5173 (dev Vite) à appeler. Modifiez ALLOWED_ORIGINS si vous servez
 * l'app sur un autre port.
 *
 * Le proxy se contente de relayer Authorization + body. Aucune persistance, aucun log
 * du token, aucun stockage. Tournez-le côté machine de l'utilisateur uniquement.
 */
import { createServer } from 'node:http';
import { request as httpsRequest } from 'node:https';

const PORT = Number(process.env.PORT) || 8088;
const HOST = '127.0.0.1';

const ALLOWED_ORIGINS = new Set([
  'http://localhost:5173',
  'http://127.0.0.1:5173',
  'http://localhost:4173',
  'http://127.0.0.1:4173',
]);

/** Map: prefix in our URL → upstream host */
const ROUTES = {
  '/x/': 'api.twitter.com',
  '/x-upload/': 'upload.twitter.com',
  '/linkedin/': 'api.linkedin.com',
};

function pickRoute(pathname) {
  for (const [prefix, host] of Object.entries(ROUTES)) {
    if (pathname.startsWith(prefix)) {
      return { host, upstreamPath: pathname.slice(prefix.length - 1) };
    }
  }
  return null;
}

const server = createServer((req, res) => {
  const origin = req.headers.origin;
  const allowOrigin = origin && ALLOWED_ORIGINS.has(origin) ? origin : '*';

  // CORS headers
  res.setHeader('Access-Control-Allow-Origin', allowOrigin);
  res.setHeader('Access-Control-Allow-Credentials', 'false');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, PATCH, DELETE, OPTIONS');
  res.setHeader(
    'Access-Control-Allow-Headers',
    'Authorization, Content-Type, Content-Length, Content-Range, X-Restli-Protocol-Version, X-Upload-Content-Type, X-Upload-Content-Length',
  );

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  if (req.url === '/_health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ ok: true, name: 'lelins-proxy', version: 1 }));
    return;
  }

  const url = new URL(req.url, `http://${HOST}:${PORT}`);
  const route = pickRoute(url.pathname);
  if (!route) {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Route inconnue. Utilisez /x/, /x-upload/ ou /linkedin/.' }));
    return;
  }

  const upstreamUrl = `https://${route.host}${route.upstreamPath}${url.search}`;
  const headers = { ...req.headers };
  delete headers.host;
  delete headers.origin;
  delete headers.referer;
  delete headers['accept-encoding'];

  const upstream = httpsRequest(
    upstreamUrl,
    { method: req.method, headers },
    (upRes) => {
      // Strip any upstream CORS headers (we set our own).
      const responseHeaders = { ...upRes.headers };
      delete responseHeaders['access-control-allow-origin'];
      delete responseHeaders['access-control-allow-credentials'];
      delete responseHeaders['access-control-allow-methods'];
      delete responseHeaders['access-control-allow-headers'];
      res.writeHead(upRes.statusCode || 502, responseHeaders);
      upRes.pipe(res);
    },
  );

  upstream.on('error', (err) => {
    console.error('[proxy] upstream error', err.message);
    if (!res.headersSent) {
      res.writeHead(502, { 'Content-Type': 'application/json' });
    }
    res.end(JSON.stringify({ error: 'upstream_failed', detail: err.message }));
  });

  req.pipe(upstream);
});

server.listen(PORT, HOST, () => {
  console.log(`[lelins-proxy] http://${HOST}:${PORT}`);
  console.log('  → /x/* → api.twitter.com');
  console.log('  → /x-upload/* → upload.twitter.com');
  console.log('  → /linkedin/* → api.linkedin.com');
});
