import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const absolute = (file) => path.join(root, file)
const read = (file) => fs.readFileSync(absolute(file), 'utf8')

const REMOVED_CORE_SYNC_PATHS = [
  'Elephant/backend/tauri/src/sync_commands.rs',
  'Elephant/backend/tauri/src/sync_contract_tests.rs',
  'Elephant/backend/tauri/src/sync/mod.rs',
  'Elephant/backend/tauri/src/sync/logging.rs',
  'Elephant/backend/tauri/src/sync/manifest.rs',
  'Elephant/backend/tauri/src/sync/plan.rs',
  'Elephant/backend/tauri/src/sync/protocol.rs',
  'Elephant/backend/tauri/src/sync/transfer.rs',
  'Elephant/backend/tauri/src/vault/sync.rs',
  'Elephant/backend/tauri/src/vault/sync_iroh/conflict_archive.rs',
  'Elephant/backend/tauri/src/vault/sync_iroh/conflict_actions.rs',
  'Elephant/backend/tauri/src/vault/sync_iroh/base.rs',
  'Elephant/backend/tauri/src/vault/sync_iroh/network.rs',
  'Elephant/backend/tauri/src/vault/sync_iroh/commands.rs',
  'Elephant/backend/tauri/src/vault/sync_iroh/e2e_tests.rs'
]

describe('Sync physical migration boundary', () => {
  it('owns the persistent Iroh endpoint and stable identity in a package service', () => {
    const manifest = JSON.parse(read('addons/official/sync/manifest.json'))
    const build = JSON.parse(read('addons/official/sync/addon.build.json'))
    const native = read('addons/official/sync/native/src/main.rs')

    expect(manifest.native.runner).toBe('service')
    expect(manifest.native.protocol).toBe('elephant-addon-service-v1')
    expect(build.runner).toBe('service')
    expect(native).toContain('Endpoint::builder(presets::Minimal)')
    expect(native).toContain('.secret_key(secret_key)')
    expect(native).toContain('.address_lookup(MdnsAddressLookup::builder().service_name(MDNS_SERVICE))')
    expect(native).toContain('load_or_create_secret_key')
    expect(native).toContain('wait_for_endpoint_addr')
    expect(native).toContain('sync.endpoint')
  })

  it('runs pairing and synchronization entirely through the physical package', () => {
    const manifest = JSON.parse(read('addons/official/sync/manifest.json'))
    const entry = read('addons/official/sync/main.service.js')
    const native = read('addons/official/sync/native/src/main.rs')
    const nativeLibrary = read('addons/official/sync/native/src/lib.rs')
    const invite = read('addons/official/sync/native/src/invite.rs')
    const pairing = read('addons/official/sync/native/src/pairing.rs')
    const protocol = read('addons/official/sync/native/src/protocol.rs')
    const session = read('addons/official/sync/native/src/session.rs')
    const transfer = read('addons/official/sync/native/src/transfer.rs')

    expect(manifest.description).toMatch(/persistent Iroh pairing/i)
    expect(manifest.description).toMatch(/bidirectional vault synchronization/i)
    expect(manifest.description).toMatch(/baselines and conflict management/i)
    expect(entry).toContain("from './main.js'")
    expect(entry).toContain("this.callNativeService('sync.create-invite'")
    expect(entry).toContain("this.callNativeService('sync.accept-invite'")
    expect(entry).toContain("this.callNativeService('sync.run'")
    expect(entry).toContain("if (command === 'iroh_sync_run') return await this.runNativeSync()")
    expect(entry).toContain("if (command === 'iroh_sync_create_invite')")
    expect(entry).toContain("if (command === 'iroh_sync_accept_invite')")
    expect(native).toContain('use elephant_sync_service::{')
    expect(native).toContain('Router::builder(endpoint.clone())')
    expect(native).toContain('SyncProtocol')
    expect(native).toContain('handle_incoming_connection')
    expect(native).toContain('ControlMessage::SyncOpen(open)')
    expect(native).toContain('serve_sync_session(')
    expect(native).toContain('run_all_sessions(&endpoint, &self.vault_dir)')
    expect(native).toContain('"sync.run" => service.run_sync().await')
    expect(native).toContain('"networkTransfersReady": true')
    expect(nativeLibrary).toContain('pub mod invite;')
    expect(nativeLibrary).toContain('pub mod pairing;')
    expect(nativeLibrary).toContain('pub mod protocol;')
    expect(nativeLibrary).toContain('pub mod session;')
    expect(nativeLibrary).toContain('pub mod transfer;')
    expect(invite).toContain('pub struct PairingInvite')
    expect(invite).toContain('pub fn create(')
    expect(invite).toContain('pub fn verify_pending_invite')
    expect(pairing).toContain('pub struct SyncConfig')
    expect(pairing).toContain('pub fn create_pending_invite')
    expect(pairing).toContain('PairingInvite::create(')
    expect(pairing).toContain('pub fn consume_pair_request')
    expect(pairing).toContain('pub fn register_accepted_peer')
    expect(protocol).toContain('pub const ALPN: &[u8] = b"elephantnote/vault-sync/1"')
    expect(protocol).toContain('pub enum ControlMessage')
    expect(protocol).toContain('PairRequest(PairRequest)')
    expect(protocol).toContain('SyncOpen(SyncOpen)')
    expect(session).toContain('pub async fn run_all_sessions')
    expect(session).toContain('pub async fn serve_sync_session')
    expect(transfer).toContain('pub async fn send_file')
    expect(transfer).toContain('pub async fn receive_file')
  })

  it('keeps all Iroh runtime and local compatibility calls physically absent from the core', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const coreCommands = read('Elephant/backend/tauri/src/core_commands.rs')
    const compatibility = read('Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js')
    const vaultModule = read('Elephant/backend/tauri/src/vault/mod.rs')
    const cargo = read('Elephant/backend/tauri/Cargo.toml')

    for (const file of REMOVED_CORE_SYNC_PATHS) expect(fs.existsSync(absolute(file))).toBe(false)
    expect(fs.existsSync(absolute('Elephant/backend/tauri/src/tauri_extra_commands.rs'))).toBe(false)
    expect(core).not.toContain('mod sync_commands;')
    expect(core).not.toContain('pub mod sync;')
    expect(core).not.toContain('IrohSyncState')
    expect(core).not.toContain('sync_commands::iroh_sync_')
    expect(core).not.toContain('tauri_sync_plan')
    expect(coreCommands).not.toContain('tauri_sync_plan')
    expect(coreCommands).not.toContain('crate::vault::sync')
    expect(compatibility).not.toContain("'sync.status'")
    expect(compatibility).not.toContain("'sync.plan'")
    expect(compatibility).not.toContain("'sync.enqueue'")
    expect(compatibility).not.toContain("'sync.run'")
    expect(vaultModule).not.toContain('pub mod sync;')
    expect(cargo).not.toContain('iroh =')
    expect(cargo).not.toContain('iroh-mdns-address-lookup')
  })

  it('keeps mobile unsupported until a real package-owned host exists', () => {
    const manifest = JSON.parse(read('addons/official/sync/manifest.json'))
    expect(manifest.native.mobile.android.supported).toBe(false)
    expect(manifest.native.mobile.ios.supported).toBe(false)
    expect(manifest.native.mobile.android.reason).toContain('mobile Sync host adapter')
    expect(manifest.native.mobile.ios.reason).toContain('mobile Sync host adapter')
  })
})
