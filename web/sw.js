// Offline cache. Versioned files (?v=<build>, the wasm and its JS glue) are immutable: cache-first, so the 30+ MB wasm is downloaded once per build.
// Everything else (index.html, scripts, manifest) is network-first, so a new build is always picked up with a matching JS/wasm pair.
// Unversioned data (character packs, stage art) is never cached here: it grew the cache past 200 MB. Only the menu packs are kept for offline use.
const CACHE = 'ume-v5';
const SHELL = /(\/|\.html|\.js|\.webmanifest|\.svg|\.menu\.pack)$/;
self.addEventListener('install', () => { self.skipWaiting(); });
self.addEventListener('activate', (e) => { e.waitUntil((async () => { for (const k of await caches.keys()) if (k !== CACHE) await caches.delete(k); await self.clients.claim(); })()); });
self.addEventListener('fetch', (e) => {
  const req = e.request;
  const url = new URL(req.url);
  if (req.method !== 'GET' || url.origin !== location.origin) return;
  e.respondWith((async () => {
    const cache = await caches.open(CACHE).catch(() => null);
    if (!cache) return fetch(req);
    if (url.searchParams.has('v')) {
      const hit = await cache.match(req).catch(() => null);
      if (hit) return hit;
      const res = await fetch(req);
      if (res.ok) {
        await (async () => {
          const old = (await cache.keys()).filter((k) => new URL(k.url).pathname === url.pathname);
          await cache.put(req, res.clone());
          await Promise.all(old.map((k) => cache.delete(k)));
        })().catch(() => {});
      }
      return res;
    }
    try {
      const res = await fetch(req);
      if (res.ok && SHELL.test(url.pathname)) await cache.put(req, res.clone()).catch(() => {});
      return res;
    } catch {
      const hit = await cache.match(req, { ignoreSearch: true });
      if (hit) return hit;
      throw new Error('offline');
    }
  })());
});
