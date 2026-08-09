import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('physical Sync package ownership', () => {
  it('owns identity, pairing, wire schema, sessions, file streams, conflicts, manifests, planning and local operations inside the native package', () => {
    const service = read('addons/official/sync/native/src/main.rs')
    const library = read('addons/official/sync/native/src/lib.rs')
    const conflicts = read('addons/official/sync/native/src/conflicts.rs')
    const invite = read('addons/official/sync/native/src/invite.rs')
    const pairing = read('addons/official/sync/native/src/pairing.rs')
    const protocol = read('addons/official/sync/native/src/protocol.rs')
    const session = read('addons/official/sync/native/src/session.rs')
    const transfer = read('addons/official/sync/native/src/transfer.rs')
    const manifest = read('addons/official/sync/native/src/manifest.rs')
    const plan = read('addons/official/sync/native/src/plan.rs')
    const localOps = read('addons/official/sync/native/src/local_ops.rs')

    expect(service).toContain('use elephant_sync_service::{')
    expect(service).toContain('Router::builder(endpoint.clone())')
    expect(service).toContain('SyncProtocol')
    expect(service).toContain('ControlMessage::SyncOpen(open)')
    expect(service).toContain('serve_sync_session(')
    expect(service).toContain('run_all_sessions(&endpoint, &self.vault_dir)')
    expect(service).toContain('load_or_create_secret_key')
    expect(service).toContain('"sync.create-invite" => service.create_invite(params).await')
    expect(service).toContain('"sync.accept-invite" => service.accept_invite(params).await')
    expect(service).toContain('"sync.scan" => service.scan()')
    expect(service).toContain('"sync.plan" => service.plan(params).await')
    expect(service).toContain('"sync.apply-local" => service.apply_local(params)')
    expect(service).toContain('"sync.conflicts.get" => service.conflict_status(true)')
    expect(service).toContain('"sync.conflicts.set" => service.set_conflict_retention(&params)')
    expect(service).toContain('"sync.conflicts.restore" => service.restore_conflict(&params)')
    expect(service).toContain('"sync.conflicts.delete" => service.delete_conflict(&params)')
    expect(service).toContain('"sync.run" => service.run_sync().await')
    expect(service).toContain('"networkTransfersReady": true')
    expect(service).toContain('"file-streams"')
    expect(service).toContain('"sync-sessions"')
    expect(service).toContain('"conflict-archive"')

    expect(library).toContain('pub mod conflicts;')
    expect(library).toContain('pub mod invite;')
    expect(library).toContain('pub mod pairing;')
    expect(library).toContain('pub mod protocol;')
    expect(library).toContain('pub mod session;')
    expect(library).toContain('pub mod transfer;')
    expect(conflicts).toContain('pub fn conflict_status')
    expect(conflicts).toContain('pub fn conflict_settings_set')
    expect(conflicts).toContain('pub fn conflict_restore')
    expect(conflicts).toContain('pub fn conflict_delete')
    expect(conflicts).toContain('Only files inside the local .conflit archive can be managed')
    expect(invite).toContain('pub struct PendingInvite')
    expect(invite).toContain('pub struct PairingInvite')
    expect(invite).toContain('pub fn create(')
    expect(invite).toContain('pub fn verify_pending_invite')
    expect(pairing).toContain('pub struct SyncConfig')
    expect(pairing).toContain('static PAIRING_STATE_LOCK: Mutex<()>')
    expect(pairing).toContain('PAIRING_STATE_LOCK')
    expect(pairing).toContain('pub fn create_pending_invite')
    expect(pairing).toContain('PairingInvite::create(')
    expect(pairing).toContain('pub fn consume_pair_request')
    expect(pairing).toContain('pub fn register_accepted_peer')
    expect(pairing).toContain('concurrent_consumers_cannot_reuse_one_invitation')
    expect(pairing).toContain('Pairing invite is invalid, expired or already used')
    expect(protocol).toContain('pub const ALPN: &[u8] = b"elephantnote/vault-sync/1"')
    expect(protocol).toContain('pub enum ControlMessage')
    expect(protocol).toContain('PairRequest(PairRequest)')
    expect(protocol).toContain('SyncComplete')
    expect(protocol).toContain('pub async fn write_control')
    expect(protocol).toContain('pub async fn read_control')
    expect(session).toContain('pub async fn run_peer_session')
    expect(session).toContain('pub async fn run_all_sessions')
    expect(session).toContain('pub async fn serve_sync_session')
    expect(session).toContain('peer sync plan does not match the independently computed plan')
    expect(session).toContain('vault manifests still differ after transfer; baseline was not advanced')
    expect(session).toContain('write_baseline')
    expect(transfer).toContain('pub async fn send_file')
    expect(transfer).toContain('pub async fn receive_file')
    expect(transfer).toContain('pub fn validate_header')
    expect(transfer).toContain('ensure_no_symlink_ancestors')
    expect(transfer).toContain('.create_new(true)')
    expect(transfer).toContain('file.sync_all().await')
    expect(transfer).toContain('hash_file(&temporary)')
    expect(manifest).toContain('pub fn scan_vault')
    expect(manifest).toContain('path_is_or_is_below(&normalized, ".elephantnote/addons")')
    expect(plan).toContain('pub fn build_plan')
    expect(plan).toContain('plan_conflict')
    expect(localOps).toContain('pub fn apply_local_plan')
    expect(localOps).toContain('ensure_no_symlink_components')
    expect(localOps).toContain('refuses_operations_through_a_symlinked_parent')
    expect(localOps).toContain('delete_empty_directories')
  })

  it('routes the active UI commands and publishes the complete native service resource', () => {
    const bridge = read('addons/official/sync/main.service.js')
    const manifest = read('addons/official/sync/manifest.json')

    for (const command of [
      'iroh_sync_status',
      'iroh_sync_create_invite',
      'iroh_sync_accept_invite',
      'iroh_sync_run',
      'iroh_sync_shutdown',
      'iroh_sync_conflict_settings_get',
      'iroh_sync_conflict_settings_set',
      'iroh_sync_conflict_restore',
      'iroh_sync_conflict_delete'
    ]) expect(bridge).toContain(`'${command}'`)

    for (const method of [
      'sync.create-invite',
      'sync.accept-invite',
      'sync.run',
      'sync.scan',
      'sync.plan',
      'sync.apply-local',
      'sync.conflicts.get',
      'sync.conflicts.peek',
      'sync.conflicts.set',
      'sync.conflicts.restore',
      'sync.conflicts.delete'
    ]) expect(bridge).toContain(`'${method}'`)

    expect(bridge).toContain("const SERVICE_RESOURCE = 'sync.native-service'")
    expect(bridge).toContain('api.resources.provide(SERVICE_RESOURCE')
    expect(bridge).toContain("'file-streams'")
    expect(bridge).toContain("'sync-sessions'")
    expect(bridge).toContain("'conflict-archive'")
    expect(manifest).toContain('"runner": "service"')
    expect(manifest).toContain('"protocol": "elephant-addon-service-v1"')
  })
})
