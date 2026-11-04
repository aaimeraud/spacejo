// Service worker de SpaceJo : rend l'application installable et accélère son ouverture.
// Stratégie « réseau d'abord » : on demande toujours la dernière version au serveur, et on ne se sert de la
// copie gardée en cache que si le réseau ne répond pas. L'API (/api/…) n'est JAMAIS mise en cache : l'état d'une
// partie doit toujours venir du serveur. Les ressources d'autres sites (police Google) ne sont pas touchées.
const CACHE = "spacejo-v1";

self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("activate", (event) => {
  // Supprime les caches d'anciennes versions du service worker.
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== "GET" || url.origin !== self.location.origin) return;
  if (url.pathname.startsWith("/api/") || url.pathname === "/sw.js") return;
  event.respondWith(
    fetch(request)
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          event.waitUntil(caches.open(CACHE).then((cache) => cache.put(request, copy)));
        }
        return response;
      })
      .catch(() => caches.match(request).then((cached) => cached ?? caches.match("/"))),
  );
});
