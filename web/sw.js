// Offline cache (network-first for HTML, cache-first for assets)
const CACHE = 'ume-v1';
self.addEventListener('install', (e) => { self.skipWaiting(); });
self.addEventListener('activate', (e) => { e.waitUntil(self.clients.claim()); });
self.addEventListener('fetch', (e) => {
  const req = e.request;
  if (req.method !== 'GET') return;
  e.respondWith((async () => {
    const cache = await caches.open(CACHE);
    try {
      const res = await fetch(req);
      if (res.ok && new URL(req.url).origin === location.origin) cache.put(req, res.clone());
      return res;
    } catch {
      const hit = await cache.match(req);
      if (hit) return hit;
      throw new Error('offline');
    }
  })());
});
