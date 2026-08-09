import { h } from 'vue'

const copyToClipboard = async(text) => {
  if (globalThis.navigator?.clipboard?.writeText) {
    await globalThis.navigator.clipboard.writeText(text)
    return true
  }
  return false
}

export default {
  name: 'RcloneSettings',
  data: () => ({
    remotePath: '',
    statusLabel: 'Not configured',
    details: '',
    inviteText: '',
    copied: false,
    password: '',
    vaultScope: 'active',
    localNetworkEnabled: true
  }),
  computed: {
    canSync() {
      return Boolean(this.remotePath && this.remotePath.trim())
    },
    canPair() {
      return this.password.length >= 8
    }
  },
  methods: {
    readEnvelope(response) {
      if (!response?.ok) throw new Error(response?.error?.message || 'Sync request failed.')
      return response.data || {}
    },
    describeStatus(data = {}) {
      if (data.running) return 'Syncing'
      if (data.lastError) return data.lastError
      if (data.configured || data.remotePath) return data.firstRunDone ? 'Synced once' : 'Ready'
      return 'Waiting for a device or sync location'
    },
    async refreshStatus() {
      try {
        const data = this.readEnvelope(await window.elephantnote.api.call('sync.status'))
        this.remotePath = data.remotePath || this.remotePath
        this.statusLabel = this.describeStatus(data)
        this.details = 'Local network sync waits for paired devices to come back online. Desktop transport can use rclone; mobile joins through the encrypted peer protocol.'
      } catch (error) {
        this.statusLabel = error?.message || 'Unable to read sync status.'
      }
    },
    async saveLocation() {
      try {
        const data = this.readEnvelope(await window.elephantnote.api.call('sync.run', {
          init: { remotePath: this.remotePath }
        }))
        this.statusLabel = this.describeStatus(data)
        this.details = 'Fallback sync location saved. Local peer sync can still run when paired devices are online.'
        this.buildInvite()
      } catch (error) {
        this.statusLabel = error?.message || 'Unable to save sync location.'
      }
    },
    async syncNow() {
      try {
        const data = this.readEnvelope(await window.elephantnote.api.call('sync.run', {
          remotePath: this.remotePath,
          sync: {}
        }))
        this.statusLabel = this.describeStatus(data)
        this.details = data.lastError || 'Sync completed.'
      } catch (error) {
        this.statusLabel = error?.message || 'Unable to run sync.'
      }
    },
    buildInvite() {
      const payload = {
        type: 'elephant-lan-pairing-invite',
        version: 1,
        vaultScope: this.vaultScope,
        encrypted: true,
        transport: 'local-network-rclone',
        remotePath: this.remotePath,
        note: 'Open Elephant Note on the other device, choose Join local network sync, enter the password, then accept this vault.'
      }
      this.inviteText = JSON.stringify(payload, null, 2)
      this.copied = false
    },
    async copyInvite() {
      this.copied = await copyToClipboard(this.inviteText)
    }
  },
  mounted() {
    this.refreshStatus().catch(() => {})
  },
  render() {
    return h('div', { class: 'pref-rclone sync-settings' }, [
      h('h4', 'Sync'),
      h('div', { class: 'sync-card' }, [
        h('h5', '1. Local network sync'),
        h('p', 'Devices announce themselves on the local network. Pair once with a password, then Elephant keeps the vault waiting for the other device when it is offline.'),
        h('label', 'Password for pairing'),
        h('input', {
          class: 'sync-input',
          type: 'password',
          value: this.password,
          placeholder: 'At least 8 characters',
          onInput: (event) => { this.password = event.target.value }
        }),
        h('label', 'Vaults to share'),
        h('select', {
          value: this.vaultScope,
          onChange: (event) => { this.vaultScope = event.target.value }
        }, [
          h('option', { value: 'active' }, 'Only current vault'),
          h('option', { value: 'all' }, 'All vaults')
        ]),
        h('button', { disabled: !this.canPair, onClick: this.buildInvite }, 'Create pairing invite')
      ]),
      this.inviteText ? h('div', { class: 'sync-card' }, [
        h('h5', '2. Connect another device'),
        h('p', 'On the phone or another computer: Join local network sync, enter the same password, then paste or scan this invite.'),
        h('pre', { class: 'sync-invite' }, this.inviteText),
        h('button', { onClick: this.copyInvite }, this.copied ? 'Copied' : 'Copy invite')
      ]) : null,
      h('div', { class: 'sync-card' }, [
        h('h5', 'Fallback shared location'),
        h('p', 'Optional. Use this when devices are not on the same network: NAS/WebDAV/Drive/SFTP/local test folder.'),
        h('input', {
          class: 'sync-input',
          value: this.remotePath,
          placeholder: 'remote:ElephantNote or /Volumes/NAS/ElephantNote',
          onInput: (event) => { this.remotePath = event.target.value }
        }),
        h('div', { class: 'sync-actions' }, [
          h('button', { onClick: this.refreshStatus }, 'Refresh status'),
          h('button', { disabled: !this.canSync, onClick: this.saveLocation }, 'Save location'),
          h('button', { disabled: !this.canSync, onClick: this.syncNow }, 'Sync now')
        ])
      ]),
      h('div', { class: 'sync-card' }, [
        h('h5', 'Status'),
        h('p', this.statusLabel),
        this.details ? h('p', { class: 'sync-details' }, this.details) : null
      ])
    ])
  }
}
