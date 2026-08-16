use serde_json::{json, Value};
use std::{
    fmt, fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub const WORKER_PROTOCOL: &str = "elephant-addon-worker-v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 64 * 1024;

const NODE_BOOTSTRAP: &str = r#"
'use strict';
const fs = require('node:fs');
const readline = require('node:readline');
const vm = require('node:vm');
const protocol = 'elephant-addon-worker-v1';
const addonId = String(process.env.ELEPHANT_ADDON_ID || '');
const entryPath = String(process.env.ELEPHANT_ADDON_ENTRY || '');
const manifest = JSON.parse(process.env.ELEPHANT_ADDON_MANIFEST || '{}');
const serializeError = (error) => ({
  name: error?.name || 'Error',
  message: error?.message || String(error),
  stack: error?.stack || ''
});
const safeValue = (value) => {
  if (value instanceof Error) return serializeError(value);
  if (value === undefined) return null;
  if (typeof value === 'function') return '[Function]';
  try { return structuredClone(value); }
  catch (_) { try { return JSON.parse(JSON.stringify(value)); } catch (_) { return String(value); } }
};
const emit = (message) => process.stdout.write(JSON.stringify(message) + '\n');
const main = async () => {
  if (!addonId || !entryPath) throw new Error('Addon worker requires an id and entry path');
  const log = (level, ...args) => emit({ type: 'log', level, args: args.map(safeValue) });
  const sandbox = {
    console: {
      debug: (...args) => log('debug', ...args),
      info: (...args) => log('info', ...args),
      log: (...args) => log('info', ...args),
      warn: (...args) => log('warn', ...args),
      error: (...args) => log('error', ...args)
    },
    setTimeout, clearTimeout, setInterval, clearInterval,
    AbortController, structuredClone, TextEncoder, TextDecoder
  };
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;
  for (const name of ['fetch', 'WebSocket', 'EventSource', 'XMLHttpRequest', 'importScripts', 'Worker', 'SharedWorker', 'BroadcastChannel', 'indexedDB', 'caches', '__TAURI__']) sandbox[name] = undefined;
  sandbox.postMessage = emit;
  const context = vm.createContext(sandbox, { name: 'elephant-addon:' + addonId });
  const source = fs.readFileSync(entryPath, 'utf8');
  vm.runInContext(source, context, { filename: entryPath });
  sandbox.__workerAddonId = addonId;
  sandbox.__workerManifest = manifest;
  sandbox.__serializeError = serializeError;
  sandbox.__safeValue = safeValue;
  sandbox.__nodeProcess = process;
  const runtime = `
  (() => {
    const addonId = self.__workerAddonId;
    const manifest = self.__workerManifest || {};
    const serializeError = self.__serializeError;
    const safeValue = self.__safeValue;
    const nodeProcess = self.__nodeProcess;
    const deepFreeze = (value) => {
      if (!value || typeof value !== 'object' || Object.isFrozen(value)) return value;
      for (const child of Object.values(value)) deepFreeze(child);
      return Object.freeze(value);
    };
    const capabilities = deepFreeze({ apiVersion: 1, runtime: 'isolated-worker', permissions: manifest.permissions || {}, contributes: manifest.contributes || {} });
    const definition = self.elephantAddon;
    const pendingRpc = new Map(), commands = new Map(), views = new Map(), listeners = new Map(), disposables = [];
    const lifecycle = new AbortController();
    let nextRpcId = 1, activationDispose = null, activated = false, stopped = false;
    const post = (message) => self.postMessage(message);
    const log = (level, ...args) => post({ type: 'log', level, args: args.map(safeValue) });
    const once = (dispose) => { let active = true; return () => { if (!active) return; active = false; return dispose(); }; };
    const toDisposable = (value) => {
      if (typeof value === 'function') return value;
      if (value && typeof value.dispose === 'function') return () => value.dispose();
      if (value && typeof value.unsubscribe === 'function') return () => value.unsubscribe();
      if (value && typeof value.abort === 'function') return () => value.abort();
      throw new TypeError('Disposable must be a function or expose dispose(), unsubscribe() or abort()');
    };
    const track = (value) => { const dispose = once(toDisposable(value)); disposables.push(dispose); return dispose; };
    const cleanup = async () => {
      while (disposables.length) {
        const dispose = disposables.pop();
        try { await dispose(); } catch (error) { post({ type: 'log', level: 'warn', args: ['cleanup failed', serializeError(error)] }); }
      }
      commands.clear(); views.clear(); listeners.clear();
      for (const pending of pendingRpc.values()) pending.reject(new Error('Addon worker stopped'));
      pendingRpc.clear();
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
    const ownsId = (value) => { const id = String(value || '').trim(); return id === addonId || id.startsWith(addonId + '.'); };
    const ownedId = (value, label) => { const id = String(value || '').trim(); if (!ownsId(id) || id === addonId) throw new Error(label + ' ids must start with ' + addonId + '.'); return id; };
    const registerCommand = (command) => {
      if (!command || typeof command !== 'object') throw new TypeError('Command definition is required');
      const id = ownedId(command.id, 'External addon command');
      if (typeof command.run !== 'function') throw new TypeError('Command run handler is required');
      const registration = { run: command.run };
      commands.set(id, registration);
      post({ type: 'register-action', action: { id, title: String(command.title || id), description: String(command.description || ''), order: Number.isFinite(command.order) ? command.order : 0 } });
      return track(() => { if (commands.get(id) === registration) { commands.delete(id); post({ type: 'unregister-action', id }); } });
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
      return track(() => { if (views.get(id) === registration) { views.delete(id); post({ type: 'unregister-view', id }); } });
    };
    const registerMany = (definitions, register) => {
      if (!Array.isArray(definitions)) throw new TypeError('Definitions must be an array');
      const registered = [];
      try { for (const definition of definitions) registered.push(register(definition)); }
      catch (error) { while (registered.length) registered.pop()(); throw error; }
      return once(() => { while (registered.length) registered.pop()(); });
    };
    const onEvent = (name, listener, single = false) => {
      const eventName = String(name || '').trim();
      if (!eventName || typeof listener !== 'function') throw new TypeError('Event name and listener are required');
      if (!listeners.has(eventName)) listeners.set(eventName, new Set());
      let dispose = () => {};
      const wrapped = single ? (payload) => { dispose(); listener(payload); } : listener;
      listeners.get(eventName).add(wrapped);
      dispose = track(() => listeners.get(eventName)?.delete(wrapped));
      return dispose;
    };
    const scheduler = Object.freeze({
      timeout(callback, delay = 0) {
        if (typeof callback !== 'function') throw new TypeError('Timeout callback must be a function');
        const id = setTimeout(callback, Math.max(0, Number(delay) || 0));
        return Object.freeze({ id, dispose: track(() => clearTimeout(id)) });
      },
      interval(callback, delay = 1) {
        if (typeof callback !== 'function') throw new TypeError('Interval callback must be a function');
        const id = setInterval(callback, Math.max(1, Number(delay) || 1));
        return Object.freeze({ id, dispose: track(() => clearInterval(id)) });
      }
    });
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
    const readNote = (path) => rpc('notes.read', { path });
    const writeNote = (path, content, options = {}) => rpc('notes.write', {
      path,
      content: String(content ?? ''),
      markdown: String(content ?? ''),
      overwrite: options.overwrite !== false
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
    const api = Object.freeze({
      app: Object.freeze({ info: () => rpc('app.info'), capabilities: () => capabilities }),
      ids: Object.freeze({ qualify: qualifyId, owns: ownsId }),
      capabilities,
      log: Object.freeze({
        debug: (...args) => log('debug', ...args),
        info: (...args) => log('info', ...args),
        warn: (...args) => log('warn', ...args),
        error: (...args) => log('error', ...args)
      }),
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
      events: Object.freeze({
        on: (name, listener) => onEvent(name, listener),
        once: (name, listener) => onEvent(name, listener, true),
        emit: (name, payload) => { for (const listener of listeners.get(String(name || '').trim()) || []) listener(payload); }
      }),
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
          if (stopped || activated) throw new Error('Addon worker is already active or stopped');
          if (!definition || typeof definition.activate !== 'function') throw new Error('Addon entry must assign self.elephantAddon = { activate(api) { ... } }');
          const dispose = await definition.activate(api);
          if (dispose != null) activationDispose = track(dispose);
          activated = true;
          post({ type: 'activation-result', id: message.id, ok: true });
        } catch (error) {
          await cleanup();
          post({ type: 'activation-result', id: message.id, ok: false, error: serializeError(error) });
        }
        return;
      }
      if (message.type === 'run-command') {
        try {
          const command = commands.get(message.commandId);
          if (!command) throw new Error('Unknown addon command: ' + message.commandId);
          post({ type: 'command-result', id: message.id, ok: true, result: await command.run(message.payload) });
        } catch (error) {
          post({ type: 'command-result', id: message.id, ok: false, error: serializeError(error) });
        }
        return;
      }
      if (message.type === 'view-state' || message.type === 'view-action') {
        try {
          const view = views.get(message.viewId);
          if (!view) throw new Error('Unknown addon view: ' + message.viewId);
          const result = message.type === 'view-state' ? await view.getState(message.params || {}) : await view.dispatch(String(message.action || ''), message.params || {});
          post({ type: message.type + '-result', id: message.id, ok: true, result });
        } catch (error) {
          post({ type: message.type + '-result', id: message.id, ok: false, error: serializeError(error) });
        }
        return;
      }
      if (message.type === 'deactivate') {
        let failure = null;
        stopped = true;
        try { lifecycle.abort(); } catch (error) { failure = error; }
        try { if (typeof definition?.deactivate === 'function') await definition.deactivate(api); } catch (error) { failure ||= error; }
        try { if (activationDispose) await activationDispose(); } catch (error) { failure ||= error; }
        await cleanup();
        post(failure ? { type: 'deactivation-result', id: message.id, ok: false, error: serializeError(failure) } : { type: 'deactivation-result', id: message.id, ok: true });
        nodeProcess.stdin.pause();
        setImmediate(() => nodeProcess.exit(failure ? 1 : 0));
        return;
      }
      post({ type: 'protocol-error', ok: false, error: { name: 'ProtocolError', message: 'Unsupported worker message type' } });
    };
  })();`;
  vm.runInContext(runtime, context, { filename: '<elephant-addon-runtime>' });
  delete sandbox.__workerAddonId;
  delete sandbox.__workerManifest;
  delete sandbox.__serializeError;
  delete sandbox.__safeValue;
  delete sandbox.__nodeProcess;
  emit({ type: 'ready', protocol, runtime: 'node', addonId });
  const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  let queue = Promise.resolve();
  rl.on('line', (line) => {
    let message;
    try { message = JSON.parse(line); } catch (error) { emit({ type: 'protocol-error', ok: false, error: serializeError(error) }); return; }
    if (!message || typeof message !== 'object' || Array.isArray(message) || typeof message.type !== 'string') {
      emit({ type: 'protocol-error', ok: false, error: { name: 'ProtocolError', message: 'Worker message must be an object with a type' } });
      return;
    }
    const dispatch = async () => {
      if (typeof sandbox.onmessage !== 'function') throw new Error('Addon worker message handler is unavailable');
      await sandbox.onmessage({ data: message });
    };
    if (message.type === 'rpc-result') {
      dispatch().catch((error) => emit({ type: 'worker-error', fatal: false, error: serializeError(error) }));
      return;
    }
    queue = queue.then(dispatch).catch((error) => emit({ type: 'worker-error', fatal: false, error: serializeError(error) }));
  });
  await new Promise((resolve) => rl.once('close', resolve));
};
main().catch((error) => { emit({ type: 'worker-error', fatal: true, error: serializeError(error) }); process.exitCode = 1; });
"#;

#[derive(Debug, Clone)]
pub struct AddonWorkerConfig {
    pub node_executable: PathBuf,
    pub entry: PathBuf,
    pub addon_id: String,
    pub manifest: Value,
    pub timeout: Duration,
}

impl AddonWorkerConfig {
    pub fn new(
        node_executable: impl Into<PathBuf>,
        entry: impl Into<PathBuf>,
        addon_id: impl Into<String>,
        manifest: Value,
    ) -> Self {
        Self {
            node_executable: node_executable.into(),
            entry: entry.into(),
            addon_id: addon_id.into(),
            manifest,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[derive(Debug)]
pub enum AddonWorkerError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidConfig(String),
    Protocol(String),
    Remote {
        operation: String,
        error: Value,
    },
    ProcessExited {
        status: Option<i32>,
        stderr: String,
    },
    Timeout {
        operation: String,
        timeout: Duration,
    },
}

impl fmt::Display for AddonWorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "addon worker I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "addon worker JSON failed: {error}"),
            Self::InvalidConfig(error) => {
                write!(formatter, "invalid addon worker configuration: {error}")
            }
            Self::Protocol(error) => write!(formatter, "addon worker protocol error: {error}"),
            Self::Remote { operation, error } => write!(
                formatter,
                "addon worker {operation} failed: {}",
                error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error")
            ),
            Self::ProcessExited { status, stderr } => {
                write!(formatter, "addon worker exited with {status:?}: {stderr}")
            }
            Self::Timeout { operation, timeout } => write!(
                formatter,
                "addon worker timed out during {operation} after {} ms",
                timeout.as_millis()
            ),
        }
    }
}

impl std::error::Error for AddonWorkerError {}

impl From<io::Error> for AddonWorkerError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for AddonWorkerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub type RpcHandler<'a> = dyn FnMut(&str, &Value) -> Result<Value, String> + 'a;

pub struct AddonWorker {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    output: mpsc::Receiver<io::Result<String>>,
    stderr: Arc<Mutex<Vec<u8>>>,
    timeout: Duration,
    next_id: u64,
    events: Vec<Value>,
    ready: Value,
    closed: bool,
}

impl AddonWorker {
    pub fn spawn(config: AddonWorkerConfig) -> Result<Self, AddonWorkerError> {
        if config.addon_id.trim().is_empty() {
            return Err(AddonWorkerError::InvalidConfig(
                "addon id is required".to_string(),
            ));
        }
        if !config.manifest.is_object() {
            return Err(AddonWorkerError::InvalidConfig(
                "manifest must be a JSON object".to_string(),
            ));
        }
        let entry = fs::canonicalize(&config.entry).map_err(|error| {
            AddonWorkerError::InvalidConfig(format!("addon entry is unavailable: {error}"))
        })?;
        if !entry.is_file() {
            return Err(AddonWorkerError::InvalidConfig(
                "addon entry must be a regular file".to_string(),
            ));
        }
        let manifest = serde_json::to_string(&config.manifest)?;
        let mut child = Command::new(&config.node_executable)
            .arg("--eval")
            .arg(NODE_BOOTSTRAP)
            .env("ELEPHANT_ADDON_ID", &config.addon_id)
            .env("ELEPHANT_ADDON_ENTRY", &entry)
            .env("ELEPHANT_ADDON_MANIFEST", manifest)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                AddonWorkerError::InvalidConfig(format!("failed to start node: {error}"))
            })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AddonWorkerError::Protocol("node stdin is unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AddonWorkerError::Protocol("node stdout is unavailable".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AddonWorkerError::Protocol("node stderr is unavailable".to_string()))?;
        let (sender, output) = mpsc::channel();
        thread::spawn(move || read_output(stdout, sender));
        let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
        let stderr_target = Arc::clone(&stderr_buffer);
        thread::spawn(move || capture_stderr(stderr, stderr_target));
        let mut worker = Self {
            child,
            stdin: BufWriter::new(stdin),
            output,
            stderr: stderr_buffer,
            timeout: config.timeout,
            next_id: 1,
            events: Vec::new(),
            ready: Value::Null,
            closed: false,
        };
        worker.ready = worker.wait_for_ready()?;
        Ok(worker)
    }

    pub fn activate(&mut self) -> Result<Value, AddonWorkerError> {
        let mut no_broker = |_method: &str, _params: &Value| {
            Err("no addon host RPC broker is attached".to_string())
        };
        self.request_with_broker(
            "activate",
            Value::Object(Default::default()),
            &mut no_broker,
        )
    }

    pub fn activate_with_broker(
        &mut self,
        broker: &mut RpcHandler<'_>,
    ) -> Result<Value, AddonWorkerError> {
        self.request_with_broker("activate", Value::Object(Default::default()), broker)
    }

    pub fn deactivate(&mut self) -> Result<Value, AddonWorkerError> {
        let mut no_broker = |_method: &str, _params: &Value| {
            Err("no addon host RPC broker is attached".to_string())
        };
        let result = self.request_with_broker(
            "deactivate",
            Value::Object(Default::default()),
            &mut no_broker,
        )?;
        self.wait_for_exit()?;
        self.closed = true;
        Ok(result)
    }

    pub fn deactivate_with_broker(
        &mut self,
        broker: &mut RpcHandler<'_>,
    ) -> Result<Value, AddonWorkerError> {
        let result =
            self.request_with_broker("deactivate", Value::Object(Default::default()), broker)?;
        self.wait_for_exit()?;
        self.closed = true;
        Ok(result)
    }

    pub fn request(
        &mut self,
        message_type: &str,
        payload: Value,
    ) -> Result<Value, AddonWorkerError> {
        let mut no_broker = |_method: &str, _params: &Value| {
            Err("no addon host RPC broker is attached".to_string())
        };
        self.request_with_broker(message_type, payload, &mut no_broker)
    }

    pub fn request_with_broker(
        &mut self,
        message_type: &str,
        payload: Value,
        broker: &mut RpcHandler<'_>,
    ) -> Result<Value, AddonWorkerError> {
        if self.closed {
            return Err(AddonWorkerError::Protocol(
                "addon worker is closed".to_string(),
            ));
        }
        let expected = response_type(message_type).ok_or_else(|| {
            AddonWorkerError::Protocol(format!("unsupported request type: {message_type}"))
        })?;
        let mut request = payload.as_object().cloned().ok_or_else(|| {
            AddonWorkerError::Protocol("worker request payload must be an object".to_string())
        })?;
        let id = self.next_id;
        self.next_id += 1;
        request.insert("type".to_string(), Value::String(message_type.to_string()));
        request.insert("id".to_string(), Value::Number(id.into()));
        self.write_json(Value::Object(request))?;
        loop {
            let message = self.read_json(message_type)?;
            let kind = message.get("type").and_then(Value::as_str).unwrap_or("");
            if kind == "worker-error" {
                return Err(AddonWorkerError::Remote {
                    operation: message_type.to_string(),
                    error: message
                        .get("error")
                        .cloned()
                        .unwrap_or_else(|| json!({ "message": "worker failed" })),
                });
            }
            if kind == "protocol-error" {
                return Err(AddonWorkerError::Remote {
                    operation: message_type.to_string(),
                    error: message
                        .get("error")
                        .cloned()
                        .unwrap_or_else(|| json!({ "message": "worker protocol error" })),
                });
            }
            if kind == "rpc" {
                self.events.push(message.clone());
                self.respond_to_rpc(&message, broker)?;
                continue;
            }
            if kind == expected && message.get("id").and_then(Value::as_u64) == Some(id) {
                if message.get("ok").and_then(Value::as_bool) == Some(true) {
                    return Ok(message);
                }
                return Err(AddonWorkerError::Remote {
                    operation: message_type.to_string(),
                    error: message
                        .get("error")
                        .cloned()
                        .unwrap_or_else(|| json!({ "message": "unknown worker error" })),
                });
            }
            self.events.push(message);
        }
    }

    pub fn ready_message(&self) -> &Value {
        &self.ready
    }
    pub fn events(&self) -> &[Value] {
        &self.events
    }
    pub fn take_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
    pub fn is_running(&mut self) -> Result<bool, AddonWorkerError> {
        Ok(self.child.try_wait()?.is_none())
    }
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(
            &self
                .stderr
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
        .into_owned()
    }

    fn wait_for_ready(&mut self) -> Result<Value, AddonWorkerError> {
        loop {
            let message = self.read_json("startup")?;
            match message.get("type").and_then(Value::as_str) {
                Some("ready")
                    if message.get("protocol").and_then(Value::as_str) == Some(WORKER_PROTOCOL) =>
                {
                    return Ok(message)
                }
                Some("worker-error") => {
                    return Err(AddonWorkerError::Remote {
                        operation: "startup".to_string(),
                        error: message.get("error").cloned().unwrap_or_else(
                            || json!({ "message": "worker failed during startup" }),
                        ),
                    })
                }
                _ => self.events.push(message),
            }
        }
    }

    fn read_json(&mut self, operation: &str) -> Result<Value, AddonWorkerError> {
        let line = self
            .output
            .recv_timeout(self.timeout)
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => AddonWorkerError::Timeout {
                    operation: operation.to_string(),
                    timeout: self.timeout,
                },
                mpsc::RecvTimeoutError::Disconnected => AddonWorkerError::ProcessExited {
                    status: self
                        .child
                        .try_wait()
                        .ok()
                        .flatten()
                        .and_then(|status| status.code()),
                    stderr: self.stderr(),
                },
            })??;
        if line.len() > MAX_LINE_BYTES {
            return Err(AddonWorkerError::Protocol(
                "worker message exceeds 16 MiB".to_string(),
            ));
        }
        let message: Value = serde_json::from_str(line.trim_end())?;
        if !message.is_object() {
            return Err(AddonWorkerError::Protocol(
                "worker message must be a JSON object".to_string(),
            ));
        }
        Ok(message)
    }

    fn write_json(&mut self, message: Value) -> Result<(), AddonWorkerError> {
        let mut bytes = serde_json::to_vec(&message)?;
        if bytes.len() > MAX_LINE_BYTES {
            return Err(AddonWorkerError::Protocol(
                "worker request exceeds 16 MiB".to_string(),
            ));
        }
        bytes.push(b'\n');
        self.stdin.write_all(&bytes)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn respond_to_rpc(
        &mut self,
        message: &Value,
        broker: &mut RpcHandler<'_>,
    ) -> Result<(), AddonWorkerError> {
        let id = message.get("id").and_then(Value::as_u64).ok_or_else(|| {
            AddonWorkerError::Protocol("worker RPC has no numeric id".to_string())
        })?;
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| AddonWorkerError::Protocol("worker RPC has no method".to_string()))?;
        let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
        let response = match broker(method, &params) {
            Ok(result) => json!({ "type": "rpc-result", "id": id, "ok": true, "result": result }),
            Err(error) => {
                json!({ "type": "rpc-result", "id": id, "ok": false, "error": { "name": "HostRpcError", "message": error } })
            }
        };
        self.write_json(response)
    }

    fn wait_for_exit(&mut self) -> Result<(), AddonWorkerError> {
        let deadline = Instant::now() + self.timeout;
        loop {
            if let Some(status) = self.child.try_wait()? {
                if status.success() {
                    return Ok(());
                }
                return Err(AddonWorkerError::ProcessExited {
                    status: status.code(),
                    stderr: self.stderr(),
                });
            }
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                let _ = self.child.wait();
                return Err(AddonWorkerError::Timeout {
                    operation: "deactivate/exit".to_string(),
                    timeout: self.timeout,
                });
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for AddonWorker {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn response_type(message_type: &str) -> Option<&'static str> {
    match message_type {
        "activate" => Some("activation-result"),
        "deactivate" => Some("deactivation-result"),
        "run-command" => Some("command-result"),
        "view-state" => Some("view-state-result"),
        "view-action" => Some("view-action-result"),
        _ => None,
    }
}

fn read_output(stdout: ChildStdout, sender: mpsc::Sender<io::Result<String>>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if sender.send(Ok(line)).is_err() {
                    break;
                }
            }
            Err(error) => {
                let _ = sender.send(Err(error));
                break;
            }
        }
    }
}

fn capture_stderr(mut stderr: impl Read, target: Arc<Mutex<Vec<u8>>>) {
    let mut bytes = Vec::new();
    let _ = stderr
        .by_ref()
        .take(MAX_STDERR_BYTES as u64)
        .read_to_end(&mut bytes);
    if let Ok(mut output) = target.lock() {
        *output = bytes;
    }
}
