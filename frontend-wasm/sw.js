// Service Worker para EcoAlerta Pasco (PWA, Offline Fallback & Web Push Serenazgo)
const CACHE_NAME = 'ecoalerta-cache-v2.4';
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

self.addEventListener('push', event => {
  let data = {};
  if (event.data) {
    try {
      data = event.data.json();
    } catch (e) {
      data = { title: 'EcoAlerta Pasco', body: event.data.text() || 'Nueva alerta registrada.' };
    }
  }

  const title = data.title || '🚨 EcoAlerta Serenazgo Pasco';
  const body = data.body || 'Se ha registrado una nueva denuncia que requiere atención.';
  const alertId = data.alert_id ? String(data.alert_id) : '';
  const domain = data.domain || 'ciudadana';
  const severity = data.severity || 'medio';
  const targetUrl = data.url || (alertId ? `/?vista=${domain}&alert_id=${alertId}` : `/?vista=${domain}`);

  const options = {
    body: body,
    icon: 'https://img.icons8.com/color/192/leaf.png',
    badge: 'https://img.icons8.com/color/192/leaf.png',
    vibrate: [300, 100, 300, 100, 400],
    tag: alertId ? `ecoalerta-alert-${alertId}` : 'ecoalerta-push',
    renotify: true,
    requireInteraction: severity === 'critico' || domain === 'ciudadana',
    data: {
      url: targetUrl,
      alert_id: alertId,
      domain: domain,
      severity: severity
    },
    actions: [
      { action: 'open', title: 'Ver en EcoAlerta' }
    ]
  };

  event.waitUntil(
    self.registration.showNotification(title, options)
  );
});

self.addEventListener('notificationclick', event => {
  event.notification.close();
  const data = event.notification.data || {};
  const targetUrl = data.url || '/';
  const alertId = data.alert_id || '';
  const domain = data.domain || '';

  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then(clientList => {
      for (const client of clientList) {
        if (client.url && client.url.startsWith(self.location.origin) && 'focus' in client) {
          client.postMessage({
            type: 'OPEN_ALERT',
            alert_id: alertId,
            domain: domain,
            url: targetUrl
          });
          return client.focus();
        }
      }
      if (self.clients.openWindow) {
        return self.clients.openWindow(targetUrl);
      }
    })
  );
});
