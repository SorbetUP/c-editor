const CACHE_NAME = 'elephantnote-pwa-v1';
const APP_ASSETS = [
  './',
  './index.html',
  './manifest.webmanifest',
  './app/bootstrap.js',
  './app/styles/app.css',
  './components/hybrid-note-editor.js',
  './editor.js',
  './editor.wasm',
  './cursor_wasm.js',
  './cursor_wasm.wasm',
  './cursor_c_interface.js'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(APP_ASSETS))
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => Promise.all(
      keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
    ))
  );
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') {
    return;
  }

  event.respondWith(
    caches.match(event.request).then((cached) => (
      cached || fetch(event.request).then((response) => {
        const clone = response.clone();
        caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
        return response;
      })
    ))
  );
});
