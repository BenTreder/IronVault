<template>
  <div class="backup-page">
    <section class="backup-hero">
      <div>
        <p class="eyebrow-small">Backup</p>
        <h2>Seal a new snapshot</h2>
        <p>
          Run a backup from a saved IronVault config file. This keeps the GUI honest and lets the core engine do the real work.
        </p>
      </div>

      <button class="iv-button iv-button-primary" type="button" @click="runBackup" :disabled="!canRunBackup">
        {{ isRunning ? 'Sealing...' : 'Run backup' }}
      </button>
    </section>

    <section class="backup-grid">
      <article class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow-small">Backup config</p>
            <h3>Choose what IronVault should run</h3>
          </div>
          <span :class="['status-badge', statusClass]">{{ status }}</span>
        </div>

        <label class="setting-label" for="configPath">Config path</label>
        <input
          id="configPath"
          v-model="configPath"
          class="backup-input"
          type="text"
          placeholder="/tmp/ironvault-gui-live-test/ironvault.toml"
          @input="clearResult"
        />

        <p class="mini-note">
          For the current test vault, use /tmp/ironvault-gui-live-test/ironvault.toml.
        </p>

        <label class="setting-label" for="backupConfirm">Confirmation</label>
        <input
          id="backupConfirm"
          v-model="backupConfirmation"
          class="backup-input"
          type="text"
          placeholder="Type SEAL"
        />

        <p class="panel-note">
          {{ statusMessage }}
        </p>
      </article>

      <article class="panel safety-panel">
        <p class="eyebrow-small">Current Settings vault</p>
        <h3>{{ repoPath }}</h3>
        <p>
          After a backup runs, this page refreshes snapshots from the vault path saved in Settings.
        </p>

        <div class="mini-stats">
          <div>
            <span>Snapshots</span>
            <strong>{{ snapshots.length }}</strong>
          </div>
          <div>
            <span>Total files</span>
            <strong>{{ totalFiles }}</strong>
          </div>
        </div>
      </article>
    </section>

    <section v-if="latestSnapshot" class="panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow-small">Latest snapshot</p>
          <h3>{{ latestSnapshot.name }}</h3>
        </div>
        <span class="status-badge">{{ formatBytes(latestSnapshot.total_size || 0) }}</span>
      </div>

      <div class="snapshot-detail-grid">
        <div>
          <span>Files</span>
          <strong>{{ snapshotFileCount(latestSnapshot) }}</strong>
        </div>
        <div>
          <span>Directories</span>
          <strong>{{ latestSnapshot.directories || 0 }}</strong>
        </div>
        <div>
          <span>Symlinks</span>
          <strong>{{ latestSnapshot.symlinks || 0 }}</strong>
        </div>
      </div>
    </section>

    <section v-if="backupResult" class="panel result-panel">
      <p class="eyebrow-small">Backup result</p>
      <h3>{{ backupResult.success ? 'Backup sealed' : 'Backup failed' }}</h3>
      <pre>{{ backupResult.message }}</pre>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  createBackup,
  formatBytes,
  listSnapshots,
  snapshotFileCount,
  type BackupResult,
  type SnapshotInfo
} from '../lib/ironvaultBridge'
import { loadRepoPath } from '../lib/ironvaultSettings'

const defaultConfigPath = '/tmp/ironvault-gui-live-test/ironvault.toml'

const repoPath = ref(loadRepoPath())
const configPath = ref(localStorage.getItem('ironvault-backup-config-path') || defaultConfigPath)
const backupConfirmation = ref('')
const backupResult = ref<BackupResult | null>(null)
const snapshots = ref<SnapshotInfo[]>([])
const isRunning = ref(false)
const status = ref('Waiting')
const statusClass = ref('status-waiting')
const statusMessage = ref('Type SEAL to unlock backup execution from this config file.')

const totalFiles = computed(() =>
  snapshots.value.reduce((sum, snapshot) => sum + snapshotFileCount(snapshot), 0)
)

const latestSnapshot = computed(() => snapshots.value[0] || null)

const canRunBackup = computed(() =>
  configPath.value.trim().length > 0 &&
  backupConfirmation.value === 'SEAL' &&
  !isRunning.value
)

function clearResult() {
  backupResult.value = null
  localStorage.setItem('ironvault-backup-config-path', configPath.value.trim())
  status.value = 'Waiting'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'Config path changed. Type SEAL again before running backup.'
  backupConfirmation.value = ''
}

async function refreshSnapshots() {
  repoPath.value = loadRepoPath()

  try {
    snapshots.value = await listSnapshots(repoPath.value)
  } catch {
    snapshots.value = []
  }
}

async function runBackup() {
  if (!canRunBackup.value) {
    return
  }

  isRunning.value = true
  backupResult.value = null
  status.value = 'Sealing'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'IronVault is sealing a new backup snapshot...'
  localStorage.setItem('ironvault-backup-config-path', configPath.value.trim())

  try {
    backupResult.value = await createBackup(configPath.value.trim())
    status.value = 'Backup sealed'
    statusClass.value = 'status-ready'
    statusMessage.value = 'Backup finished. Snapshot list refreshed.'
    backupConfirmation.value = ''
    await refreshSnapshots()
  } catch (error) {
    backupResult.value = {
      success: false,
      message: error instanceof Error ? error.message : 'IronVault could not complete the backup.'
    }
    status.value = 'Backup failed'
    statusClass.value = 'status-error'
    statusMessage.value = 'Backup did not complete. Check the result message below.'
  } finally {
    isRunning.value = false
  }
}

onMounted(refreshSnapshots)
</script>

<style scoped>
.backup-page {
  display: grid;
  gap: 1.25rem;
}

.backup-hero,
.panel,
.result-panel {
  border: 1px solid var(--iv-border);
  background: color-mix(in srgb, var(--iv-surface) 94%, transparent);
  box-shadow: var(--iv-shadow-soft);
}

.backup-hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: clamp(1.35rem, 4vw, 2rem);
  border-radius: var(--iv-radius-lg);
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--iv-surface) 94%, transparent), var(--iv-surface-raised)),
    radial-gradient(circle at 12% 18%, rgba(249, 115, 22, 0.14), transparent 24rem);
}

.backup-hero h2 {
  margin: 0.2rem 0 0;
  font-size: clamp(2.1rem, 5vw, 4rem);
  line-height: 0.95;
  letter-spacing: -0.075em;
}

.backup-hero p,
.panel-note,
.safety-panel p,
.mini-note {
  color: var(--iv-muted);
  line-height: 1.6;
}

.backup-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(280px, 0.75fr);
  gap: 1rem;
}

.panel {
  padding: 1.25rem;
  border-radius: var(--iv-radius-md);
}

.panel-heading {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: flex-start;
}

.panel h3,
.result-panel h3 {
  margin: 0.2rem 0 0;
  font-size: 1.35rem;
  overflow-wrap: anywhere;
}

.eyebrow-small {
  margin: 0;
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.setting-label {
  display: block;
  margin: 1rem 0 0.55rem;
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.backup-input {
  min-height: 44px;
  width: 100%;
  padding: 0.7rem 0.9rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-bg-soft);
  color: var(--iv-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.mini-note {
  margin: 0.6rem 0 0;
  font-size: 0.82rem;
}

.status-badge {
  padding: 0.42rem 0.72rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-accent-soft);
  color: var(--iv-text);
  font-size: 0.78rem;
  font-weight: 900;
  white-space: nowrap;
}

.status-ready {
  color: var(--iv-success);
}

.status-waiting {
  color: var(--iv-warning);
}

.status-error {
  color: var(--iv-danger);
}

.mini-stats,
.snapshot-detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
  margin-top: 1rem;
}

.snapshot-detail-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.mini-stats div,
.snapshot-detail-grid div {
  padding: 1rem;
  border: 1px solid var(--iv-border);
  border-radius: 16px;
  background: var(--iv-bg-soft);
}

.mini-stats span,
.snapshot-detail-grid span {
  display: block;
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.mini-stats strong,
.snapshot-detail-grid strong {
  display: block;
  margin-top: 0.4rem;
  font-size: 1.6rem;
  letter-spacing: -0.04em;
}

.result-panel pre {
  margin: 1rem 0 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  color: var(--iv-muted-strong);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

@media (max-width: 940px) {
  .backup-hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .backup-grid,
  .mini-stats,
  .snapshot-detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
