export const createIsolatedAddonWorkerSource = (entrySource, addonId, manifest = {}) => {
  const workerCapabilities = {
    apiVersion: 1,
    runtime: 'isolated-worker',
    permissions: manifest.permissions || {},
    contributes: manifest.contributes || {}
  }
  return `
'use strict';
const __nativePost = self.postMessage.bind(self);
const __disable = (name) => {
  try { Object.defineProperty(self, name, { value: undefined, writable: false, configurable: false }); }
  catch (_) { try { self[name] = undefined; } catch (_) {} }
};
['fetch','WebSocket','EventSource','XMLHttpRequest','importScripts','Worker','SharedWorker','BroadcastChannel','indexedDB','caches'].forEach(__disable);
try {
  Object.defineProperty(self, '__TAURI__', { value: undefined, writable: false, configurable: false });
  Object.defineProperty(self, 'postMessage', { value: __nativePost, writable: false, configurable: false });
} catch (_) {}
${entrySource}
;
(() => {
  const addonId = ${JSON.stringify(addonId)};
  const deepFreeze = (value) => {
    if (!value || typeof value !== 'object' || Object.isFrozen(value)) return value;
    for (const child of Object.values(value)) deepFreeze(child);
    return Object.freeze(value);
  };
  const capabilities = deepFreeze(${JSON.stringify(workerCapabilities)});
  const definition = self.elephantAddon;
  const pendingRpc = new Map();
  const commands = new Map();
  const views = new Map();
  const eventListeners = new Map();
  const disposables = [];
  const lifecycle = typeof AbortController === 'function' ? new AbortController() : (() => {
    const listeners = new Set();
    const signal = {
      aborted: false,
      addEventListener(name, listener) { if (name === 'abort' && typeof listener === 'function') listeners.add(listener); },
      removeEventListener(name, listener) { if (name === 'abort') listeners.delete(listener); }
    };
    return { signal, abort() {
      if (signal.aborted) return;
      signal.aborted = true;
      for (const listener of listeners) listener();
      listeners.clear();
    } };
  })();
  let nextRpcId = 1;
  let activationDispose = null;
  const post = (message) => __nativePost(message);
  const serializeError = (error) => ({ name: error?.name || 'Error', message: error?.message || String(error), stack: error?.stack || '' });
  const serializeLogValue = (value) => {
    if (value instanceof Error) return serializeError(value);
    if (value === undefined) return null;
    if (typeof value === 'function') return '[Function]';
    try { return structuredClone(value); }
    catch (_) { try { return JSON.parse(JSON.stringify(value)); } catch (_) { return String(value); } }
  };
  const once = (dispose) => {
    let active = true;
    return () => {
      if (!active) return;
      active = false;
      return dispose();
    };
  };
  const toDisposable = (value) => {
    if (typeof value === 'function') return value;
    if (value && typeof value.dispose === 'function') return () => value.dispose();
    if (value && typeof value.unsubscribe === 'function') return () => value.unsubscribe();
    if (value && typeof value.abort === 'function') return () => value.abort();
    throw new TypeError('Disposable must be a function or expose dispose(), unsubscribe() or abort()');
  };
  const track = (value) => {
    const dispose = once(toDisposable(value));
    disposables.push(dispose);
    return dispose;
  };
  const cleanup = async () => {
    while (disposables.length) {
      const dispose = disposables.pop();
      try { await dispose(); }
      catch (error) { post({ type: 'log', level: 'warn', args: ['cleanup failed', serializeError(error)] }); }
    }
    commands.clear();
    views.clear();
    eventListeners.clear();
  };
  const rpc = (method, params = {}) => new Promise((resolve, reject) => {
    const id = nextRpcId++;
    pendingRpc.set(id, { resolve, reject });
    post({ type: 'rpc', id, method, params });
  });
  const qualifyId = (value) => {
    const id = String(value || '').trim();
    if (!id) throw new TypeError('Addon id suffix is required');
    return id.startsWith(addonId + '.') ? id : addonId + '.' + id;
  };
  const ownsId = (value) => {
    const id = String(value || '').trim();
    return id === addonId || id.startsWith(addonId + '.');
  };
  const ownedId = (value, label) => {
    const id = String(value || '').trim();
    if (!ownsId(id) || id === addonId) throw new Error(label + ' ids must start with ' + addonId + '.');
    return id;
  };
  const registerCommand = (command) => {
    if (!command || typeof command !== 'object') throw new TypeError('Command definition is required');
    const id = ownedId(command.id, 'External addon command');
    if (typeof command.run !== 'function') throw new TypeError('Command run handler is required');
    const registration = { run: command.run };
    commands.set(id, registration);
    post({ type: 'register-action', action: { id, title: String(command.title || id), description: String(command.description || ''), order: Number.isFinite(command.order) ? command.order : 0 } });
    return track(() => {
      if (commands.get(id) !== registration) return;
      commands.delete(id);
      post({ type: 'unregister-action', id });
    });
  };
  const registerView = (view) => {
    if (!view || typeof view !== 'object') throw new TypeError('View definition is required');
    const id = ownedId(view.id, 'External addon view');
    if (typeof view.getState !== 'function' || typeof view.dispatch !== 'function') throw new TypeError('View getState and dispatch handlers are required');
    const kind = String(view.kind || '').trim();
    if (!kind) throw new TypeError('View kind is required');
    const registration = { getState: view.getState, dispatch: view.dispatch };
    views.set(id, registration);
    post({ type: 'register-view', view: { id, title: String(view.title || id), description: String(view.description || ''), icon: String(view.icon || 'list-todo'), kind, order: Number.isFinite(view.order) ? view.order : 0 } });
    return track(() => {
      if (views.get(id) !== registration) return;
      views.delete(id);
      post({ type: 'unregister-view', id });
    });
  };
  const registerMany = (definitions, register) => {
    if (!Array.isArray(definitions)) throw new TypeError('Definitions must be an array');
    const registered = [];
    try { for (const definition of definitions) registered.push(register(definition)); }
    catch (error) { while (registered.length) registered.pop()(); throw error; }
    return once(() => { while (registered.length) registered.pop()(); });
  };
  const storageEntries = () => rpc('storage.entries');
  const storage = Object.freeze({
    get: (key) => rpc('storage.get', { key }),
    set: (key, value) => rpc('storage.set', { key, value }),
    remove: (key) => rpc('storage.remove', { key }),
    entries: storageEntries,
    async has(key) { return Object.prototype.hasOwnProperty.call(await storageEntries(), key); },
    async keys() { return Object.keys(await storageEntries()).sort(); },
    async clear() {
      const entries = await storageEntries();
      await Promise.all(Object.keys(entries).map((key) => rpc('storage.remove', { key })));
    },
    async update(key, updater, fallback) {
      if (typeof updater !== 'function') throw new TypeError('Storage updater must be a function');
      const current = await rpc('storage.get', { key });
      const next = await updater(current === undefined || current === null ? fallback : current);
      await rpc('storage.set', { key, value: next });
      return next;
    }
  });
  const httpRequest = (request) => rpc('http.request', request || {});
  const http = Object.freeze({
    request: httpRequest,
    get: (url, options = {}) => httpRequest({ ...options, url, method: 'GET' }),
    post: (url, body, options = {}) => {
      const headers = { ...(options.headers || {}) };
      const payload = typeof body === 'string' ? body : JSON.stringify(body);
      if (typeof body !== 'string' && !Object.keys(headers).some((name) => name.toLowerCase() === 'content-type')) headers['content-type'] = 'application/json';
      return httpRequest({ ...options, url, method: 'POST', headers, body: payload });
    },
    async requestJson(request) {
      const response = await httpRequest(request);
      return Object.freeze({ ...response, json: JSON.parse(response.body || 'null') });
    }
  });
  const localEventName = (name) => {
    const eventName = String(name || '').trim();
    if (!eventName) throw new TypeError('Event name is required');
    return eventName;
  };
  const onEvent = (name, listener, single = false) => {
    const eventName = localEventName(name);
    if (typeof listener !== 'function') throw new TypeError('Event listener must be a function');
    if (!eventListeners.has(eventName)) eventListeners.set(eventName, new Set());
    let dispose = () => {};
    const wrapped = single ? (payload) => { dispose(); listener(payload); } : listener;
    eventListeners.get(eventName).add(wrapped);
    dispose = track(() => eventListeners.get(eventName)?.delete(wrapped));
    return dispose;
  };
  const emitEvent = (name, payload) => {
    for (const listener of eventListeners.get(localEventName(name)) || []) listener(payload);
  };
  const scheduler = Object.freeze({
    timeout(callback, delay = 0) {
      if (typeof callback !== 'function') throw new TypeError('Timeout callback must be a function');
      const id = setTimeout(callback, Math.max(0, Number(delay) || 0));
      const dispose = track(() => clearTimeout(id));
      return Object.freeze({ id, dispose });
    },
    interval(callback, delay = 0) {
      if (typeof callback !== 'function') throw new TypeError('Interval callback must be a function');
      const id = setInterval(callback, Math.max(1, Number(delay) || 1));
      const dispose = track(() => clearInterval(id));
      return Object.freeze({ id, dispose });
    }
  });
  const log = Object.freeze(Object.fromEntries(['debug','info','warn','error'].map((level) => [level, (...args) => post({ type: 'log', level, args: args.map(serializeLogValue) })])));
  const readNote = (path) => rpc('notes.read', { path });
  const writeNote = (path, content, options = {}) => rpc('notes.write', {
    path,
    content: String(content ?? ''),
    markdown: String(content ?? ''),
    overwrite: options?.overwrite !== false
  });
  const api = Object.freeze({
    app: Object.freeze({ info: () => rpc('app.info'), capabilities: () => capabilities }),
    ids: Object.freeze({ qualify: qualifyId, owns: ownsId }),
    capabilities,
    log,
    notes: Object.freeze({
      list: (prefix) => rpc('notes.list', { prefix }),
      read: readNote,
      write: writeNote,
      async update(path, updater, options = {}) {
        if (typeof updater !== 'function') throw new TypeError('Note updater must be a function');
        const document = await readNote(path);
        const current = typeof document === 'string' ? document : String(document?.markdown ?? document?.content ?? '');
        const next = await updater(current, document);
        if (typeof next !== 'string') throw new TypeError('Note updater must return Markdown text');
        const result = await writeNote(path, next, { ...options, overwrite: true });
        return Object.freeze({ ...result, markdown: next, content: next });
      }
    }),
    http,
    storage,
    commands: Object.freeze({ register: registerCommand, registerMany: (definitions) => registerMany(definitions, registerCommand) }),
    views: Object.freeze({ register: registerView, registerMany: (definitions) => registerMany(definitions, registerView) }),
    events: Object.freeze({ on: (name, listener) => onEvent(name, listener), once: (name, listener) => onEvent(name, listener, true), emit: emitEvent }),
    scheduler,
    lifecycle: Object.freeze({
      signal: lifecycle.signal,
      addDisposable: track,
      onAbort(listener) {
        if (typeof listener !== 'function') throw new TypeError('Abort listener must be a function');
        lifecycle.signal.addEventListener('abort', listener, { once: true });
        return track(() => lifecycle.signal.removeEventListener('abort', listener));
      }
    })
  });
  self.onmessage = async (event) => {
    const message = event?.data || {};
    if (message.type === 'rpc-result') {
      const pending = pendingRpc.get(message.id);
      if (!pending) return;
      pendingRpc.delete(message.id);
      if (message.ok) pending.resolve(message.result);
      else pending.reject(new Error(message.error?.message || message.error || 'Addon API call failed'));
      return;
    }
    if (message.type === 'activate') {
      try {
        if (!definition || typeof definition.activate !== 'function') throw new Error('Addon entry must assign self.elephantAddon = { activate(api) { ... } }');
        const dispose = await definition.activate(api);
        if (dispose != null) activationDispose = track(dispose);
        post({ type: 'activation-result', id: message.id, ok: true });
      } catch (error) { post({ type: 'activation-result', id: message.id, ok: false, error: serializeError(error) }); }
      return;
    }
    if (message.type === 'run-command') {
      try {
        const command = commands.get(message.commandId);
        if (!command) throw new Error('Unknown addon command: ' + message.commandId);
        post({ type: 'command-result', id: message.id, ok: true, result: await command.run(message.payload) });
      } catch (error) { post({ type: 'command-result', id: message.id, ok: false, error: serializeError(error) }); }
      return;
    }
    if (message.type === 'view-state' || message.type === 'view-action') {
      try {
        const view = views.get(message.viewId);
        if (!view) throw new Error('Unknown addon view: ' + message.viewId);
        const result = message.type === 'view-state' ? await view.getState(message.params || {}) : await view.dispatch(String(message.action || ''), message.params || {});
        post({ type: message.type + '-result', id: message.id, ok: true, result });
      } catch (error) { post({ type: message.type + '-result', id: message.id, ok: false, error: serializeError(error) }); }
      return;
    }
    if (message.type === 'deactivate') {
      try {
        lifecycle.abort();
        if (typeof definition?.deactivate === 'function') await definition.deactivate(api);
        if (activationDispose) await activationDispose();
        await cleanup();
        post({ type: 'deactivation-result', id: message.id, ok: true });
      } catch (error) {
        await cleanup();
        post({ type: 'deactivation-result', id: message.id, ok: false, error: serializeError(error) });
      }
    }
  };
})();
`
}
