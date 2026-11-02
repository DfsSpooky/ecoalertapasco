// Service Worker para EcoAlerta Pasco (PWA & Offline Fallback)
const CACHE_NAME = 'ecoalerta-cache-v2.3';
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/manifest.json'
];

self.addEventListener('install', event => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => {
      return cache.addAll(STATIC_ASSETS).catch(err => console.warn('Cache install partial:', err));
    })
  );
});

self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(keys => {
      return Promise.all(
        keys.map(key => {
          if (key !== CACHE_NAME) {
            return caches.delete(key);
          }
        })
      );
    }).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);

  // Solo interceptar peticiones del mismo origen (archivos estáticos de EcoAlerta).
  // Ignorar completamente llamadas a APIs externas (Nominatim, OpenStreetMap tiles, CDNs) y a la API Django (/api/)
  if (url.origin !== self.location.origin || url.pathname.startsWith('/api/') || event.request.method !== 'GET') {
    return;
  }

  // Network-first con fallback seguro a caché para assets estáticos locales
  event.respondWith(
    fetch(event.request)
      .then(response => {
        if (response && response.status === 200) {
          const respClone = response.clone();
          caches.open(CACHE_NAME).then(cache => cache.put(event.request, respClone));
        }
        return response;
      })
      .catch(async () => {
        const cached = await caches.match(event.request);
        if (cached) return cached;
        if (event.request.mode === 'navigate') {
          const indexCached = await caches.match('/index.html');
          if (indexCached) return indexCached;
        }
        return new Response('Offline', { status: 503, statusText: 'Service Unavailable' });
      })
  );
});
