import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('external addon runtime security contracts', () => {
  it('routes note access through dedicated permission-scoped Tauri commands', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js')
    const worker = read('Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js')
    const notes = read('Elephant/backend/tauri/src/addon_note_access.rs')
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')

    expect(worker).toContain("list: (prefix) => rpc('notes.list', { prefix })")
    expect(worker).toContain("const readNote = (path) => rpc('notes.read', { path })")
    expect(worker).toContain("const writeNote = (path, content, options = {}) => rpc('notes.write'")
    expect(worker).toContain('async update(path, updater, options = {})')
    expect(runtime).toContain("invoke('tauri_addons_notes_list', { addonId, prefix })")
    expect(runtime).toContain("invoke('tauri_addons_notes_read', { addonId, path })")
    expect(runtime).toContain("invoke('tauri_addons_notes_write'")
    expect(runtime).toContain("if (method === 'notes.list')")
    expect(runtime).toContain("if (method === 'notes.read')")
    expect(runtime).toContain("if (method === 'notes.write')")
    expect(notes).toContain('MAX_LISTED_NOTES: usize = 1_000')
    expect(notes).toContain('MAX_DIRECTORY_DEPTH: usize = 64')
    expect(notes).toContain('MAX_NOTE_BYTES: u64 = 5 * 1024 * 1024')
    expect(notes).toContain('is_hidden_component(relative)')
    expect(notes).toContain('file_type.is_symlink()')
    expect(notes).toContain('scope_matches(scope, relative_path)')
    expect(notes).toContain('Addon is not permitted to read')
    expect(notes).toContain('Addons cannot access notes in hidden directories')
    expect(notes).toContain('record.manifest.permissions.notes.write')
    expect(notes).toContain('write_markdown_atomic')
    expect(lib).toContain('addon_note_access::tauri_addons_notes_list')
    expect(lib).toContain('addon_note_access::tauri_addons_notes_read')
    expect(lib).toContain('addon_note_access::tauri_addons_notes_write')
  })

  it('routes Worker network requests through the hardened broker', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js')
    const worker = read('Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js')
    const http = read('Elephant/backend/tauri/src/addon_http_access.rs')
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')

    expect(worker).toContain("const httpRequest = (request) => rpc('http.request', request || {})")
    expect(worker).toContain('async requestJson(request)')
    expect(runtime).toContain("invoke('tauri_addons_http_request', { addonId, params })")
    expect(runtime).toContain("if (method === 'http.request')")
    expect(http).toContain('Network access to a local or private address')
    expect(http).toContain('External addon HTTPS requests are restricted to port 443')
    expect(http).toContain('redirect(reqwest::redirect::Policy::none())')
    expect(http).toContain('resolve(host, address)')
    expect(http).toContain('MAX_HTTP_RESPONSE_BYTES: u64 = 5 * 1024 * 1024')
    expect(lib).toContain('addon_http_access::tauri_addons_http_request')
  })

  it('keeps unsafe browser capabilities disabled inside isolated Workers', () => {
    const worker = read('Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js')

    expect(worker).toContain("['fetch','WebSocket','EventSource','XMLHttpRequest','importScripts','Worker','SharedWorker','BroadcastChannel','indexedDB','caches'].forEach(__disable)")
    expect(worker).toContain("Object.defineProperty(self, '__TAURI__'")
    expect(worker).toContain("runtime: 'isolated-worker'")
  })

  it('shares registry and permission logic instead of duplicating it', () => {
    const shared = read('Elephant/backend/tauri/src/addon_runtime_access.rs')
    const notes = read('Elephant/backend/tauri/src/addon_note_access.rs')
    const http = read('Elephant/backend/tauri/src/addon_http_access.rs')

    expect(shared).toContain('pub fn read_enabled_addon')
    expect(shared).toContain('pub fn scope_matches')
    expect(shared).toContain('pub fn host_matches')
    expect(notes).toContain('addon_runtime_access')
    expect(http).toContain('addon_runtime_access')
    expect(notes).not.toContain('struct AddonRegistry')
    expect(http).not.toContain('struct AddonRegistry')
  })
})
