<template>
  <div class="snapshots-page">
    <section class="snapshots-hero">
      <div>
        <p class="eyebrow-small">Snapshots</p>
        <h2>Your sealed backups</h2>
        <p>
          Each snapshot is a saved backup point. Review what each one contains, then use Restore when you need files back.
        </p>
      </div>

      <button class="iv-button iv-button-primary" type="button" @click="refreshSnapshots" :disabled="isLoading">
        {{ isLoading ? 'Refreshing...' : 'Refresh snapshots' }}
      </button>
    </section>

    <section class="panel vault-panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow-small">Current vault</p>
          <h3>{{ repoPath }}</h3>
        </div>
        <span :class="['status-badge', statusClass]">{{ status }}</span>
      </div>

      <p class="panel-note">
        {{ message }}
      </p>
    </section>

    <section v-if="snapshots.length" class="snapshot-cards">
      <article
        v-for="snapshot in snapshots"
        :key="snapshot.name"
        class="panel snapshot-card"
      >
        <div class="panel-heading">
          <div>
            <p class="eyebrow-small">Snapshot</p>
            <h3>{{ snapshot.name }}</h3>
          </div>
          <span class="status-badge status-ready">Saved</span>
        </div>

        <div class="detail-grid">
          <div>
            <span>Files</span>
            <strong>{{ snapshotFileCount(snapshot) }}</strong>
          </div>
          <div>
            <span>Folders</span>
            <strong>{{ snapshot.directories || 0 }}</strong>
          </div>
          <div>
            <span>Symlinks</span>
            <strong>{{ snapshot.symlinks || 0 }}</strong>
          </div>
          <div>
            <span>Size</span>
            <strong>{{ formatBytes(snapshot.total_size || 0) }}</strong>
          </div>
        </div>

        <section class="restore-guide">
          <p class="eyebrow-small">Restore guidance</p>
          <h4>How to restore this backup</h4>
          <p>
            Go to Restore, choose this snapshot, preview it first, then type RESTORE only after the plan looks safe.
          </p>

          <div class="command-card">
            <span>CLI restore command</span>
            <code>{{ restoreCommand(snapshot) }}</code>
          </div>

          <p class="field-help">
            IronVault restores the original backed-up folder inside your restore folder. Example: restoring BackupTest into /tmp/restore creates /tmp/restore/BackupTest/.
          </p>
        </section>
      </article>
    </section>

    <section v-else class="panel empty-state">
      <p class="eyebrow-small">No snapshots yet</p>
      <h3>No sealed backups found</h3>
      <p>
        Go to Backup, check your setup, type SEAL, then run your first backup.
      </p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
  formatBytes,
  listSnapshots,
  snapshotFileCount,
  type SnapshotInfo
} from '../lib/ironvaultBridge'
import { loadRepoPath } from '../lib/ironvaultSettings'

const repoPath = ref(loadRepoPath())
const snapshots = ref<SnapshotInfo[]>([])
const isLoading = ref(false)
const status = ref('Waiting')
const statusClass = ref('status-waiting')
const message = ref('Refresh snapshots to see the latest sealed backups.')

function restoreCommand(snapshot: SnapshotInfo) {
  return `ironvault restore --repo "${repoPath.value}" --snapshot "${snapshot.name}" --target "/tmp/ironvault-restore-preview" --if-exists refuse`
}

async function refreshSnapshots() {
  isLoading.value = true
  repoPath.value = loadRepoPath()
  status.value = 'Checking'
  statusClass.value = 'status-waiting'
  message.value = 'Looking inside the vault for sealed backups...'

  try {
    snapshots.value = await listSnapshots(repoPath.value)

    if (snapshots.value.length) {
      status.value = 'Loaded'
      statusClass.value = 'status-ready'
      message.value = `${snapshots.value.length} snapshot(s) found. Each card below shows its backup details.`
    } else {
      status.value = 'Empty'
      statusClass.value = 'status-waiting'
      message.value = 'No snapshots found in this vault yet.'
    }
  } catch (error) {
    snapshots.value = []
    status.value = 'Needs attention'
    statusClass.value = 'status-error'
    message.value = error instanceof Error
      ? error.message
      : 'Could not load snapshots from this vault.'
  } finally {
    isLoading.value = false
  }
}

onMounted(refreshSnapshots)
</script>

<style scoped>
.snapshots-page {
  display: grid;
  gap: 1.25rem;
}

.snapshots-hero,
.panel {
  border: 1px solid var(--iv-border);
  background: color-mix(in srgb, var(--iv-surface) 94%, transparent);
  box-shadow: var(--iv-shadow-soft);
}

.snapshots-hero {
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

.snapshots-hero h2 {
  margin: 0.2rem 0 0;
  font-size: clamp(2.1rem, 5vw, 4rem);
  line-height: 0.95;
  letter-spacing: -0.075em;
}

.snapshots-hero p,
.panel-note,
.restore-guide p,
.empty-state p,
.field-help {
  color: var(--iv-muted);
  line-height: 1.6;
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
.restore-guide h4 {
  margin: 0.2rem 0 0;
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

.snapshot-cards {
  display: grid;
  gap: 1rem;
}

.snapshot-card {
  display: grid;
  gap: 1rem;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.75rem;
}

.detail-grid div {
  padding: 1rem;
  border: 1px solid var(--iv-border);
  border-radius: 16px;
  background: var(--iv-bg-soft);
}

.detail-grid span,
.command-card span {
  display: block;
  color: var(--iv-muted);
  font-size: 0.72rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.detail-grid strong {
  display: block;
  margin-top: 0.4rem;
  font-size: 1.55rem;
  letter-spacing: -0.04em;
}

.restore-guide {
  padding-top: 1rem;
  border-top: 1px solid var(--iv-border);
}

.command-card {
  display: grid;
  gap: 0.5rem;
  margin-top: 1rem;
  padding: 1rem;
  border: 1px solid var(--iv-border);
  border-radius: 16px;
  background: var(--iv-bg-soft);
}

.command-card code {
  overflow-wrap: anywhere;
  color: var(--iv-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.empty-state {
  display: grid;
  gap: 0.3rem;
}

@media (max-width: 940px) {
  .snapshots-hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .detail-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
