// Offline cache. Versioned files (?v=<build>, the wasm and its JS glue) are immutable: cache-first, so the 30+ MB wasm is downloaded once per build.
// Everything else (index.html, scripts, manifest) is network-first, so a new build is always picked up with a matching JS/wasm pair.
const CACHE = 'ume-v4';
self.addEventListener('install', () => { self.skipWaiting(); });
self.addEventListener('activate', (e) => { e.waitUntil((async () => { for (const k of await caches.keys()) if (k !== CACHE) await caches.delete(k); await self.clients.claim(); })()); });
self.addEventListener('fetch', (e) => {
  const req = e.request;
  const url = new URL(req.url);
  if (req.method !== 'GET' || url.origin !== location.origin) return;
  e.respondWith((async () => {
    const cache = await caches.open(CACHE);
    if (url.searchParams.has('v')) {
      const hit = await cache.match(req);
      if (hit) return hit;
      const res = await fetch(req);
      if (res.ok) {
        const old = (await cache.keys()).filter((k) => new URL(k.url).pathname === url.pathname);
        await cache.put(req, res.clone());
        await Promise.all(old.map((k) => cache.delete(k)));
      }
      return res;
    }
    try {
      const res = await fetch(req);
      if (res.ok) cache.put(req, res.clone());
      return res;
    } catch {
      const hit = await cache.match(req, { ignoreSearch: true });
      if (hit) return hit;
      throw new Error('offline');
    }
  })());
});
