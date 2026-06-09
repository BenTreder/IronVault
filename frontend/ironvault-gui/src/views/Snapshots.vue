<template>
  <div class="snapshots-page">
    <section class="snapshots-hero">
      <div>
        <p class="eyebrow-small">Snapshots</p>
        <h2>Vault shelves</h2>
        <p>
          Review the backup snapshots available in your saved IronVault repo.
        </p>
      </div>

      <button class="iv-button iv-button-primary" type="button" @click="loadSnapshots" :disabled="isLoading">
        {{ isLoading ? 'Checking shelves...' : 'Refresh snapshots' }}
      </button>
    </section>

    <section class="snapshot-status-card">
      <div>
        <p class="eyebrow-small">Current vault</p>
        <p class="repo-path">{{ repoPath }}</p>
      </div>

      <div class="snapshot-status">
        <span :class="['status-dot', statusClass]" aria-hidden="true"></span>
        <strong>{{ status }}</strong>
        <small>{{ statusMessage }}</small>
      </div>
    </section>

    <section class="snapshot-summary-grid" aria-label="Snapshot summary">
      <article class="summary-card">
        <span>Total snapshots</span>
        <strong>{{ snapshots.length }}</strong>
        <small>Backup shelves currently available.</small>
      </article>

      <article class="summary-card">
        <span>Total files</span>
        <strong>{{ totalFiles }}</strong>
        <small>Files tracked across listed snapshots.</small>
      </article>

      <article class="summary-card">
        <span>Total size</span>
        <strong>{{ totalSizeLabel }}</strong>
        <small>Stored data reported by snapshot metadata.</small>
      </article>
    </section>

    <section class="snapshot-list">
      <article v-if="snapshots.length === 0" class="empty-card">
        <p class="eyebrow-small">No shelves shown</p>
        <h3>{{ emptyTitle }}</h3>
        <p>
          {{ emptyMessage }}
        </p>
      </article>

      <article v-for="snapshot in snapshots" :key="snapshot.name" class="snapshot-card">
        <div class="snapshot-card-top">
          <div>
            <p class="eyebrow-small">Snapshot</p>
            <h3>{{ snapshot.name }}</h3>
          </div>
          <span class="snapshot-size">{{ formatBytes(snapshot.total_size ?? 0) }}</span>
        </div>

        <div class="snapshot-metrics">
          <span>
            <strong>{{ snapshotFileCount(snapshot) }}</strong>
            files
          </span>
          <span>
            <strong>{{ snapshot.directories ?? 0 }}</strong>
            dirs
          </span>
          <span>
            <strong>{{ snapshot.symlinks ?? 0 }}</strong>
            links
          </span>
        </div>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
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
const statusMessage = ref('Snapshots will load from the vault path saved in Settings.')
const emptyTitle = ref('No snapshots loaded yet')
const emptyMessage = ref('Click refresh to ask IronVault what shelves are available.')

const totalFiles = computed(() =>
  snapshots.value.reduce((total, snapshot) => total + snapshotFileCount(snapshot), 0)
)

const totalSize = computed(() =>
  snapshots.value.reduce((total, snapshot) => total + (snapshot.total_size ?? 0), 0)
)

const totalSizeLabel = computed(() => formatBytes(totalSize.value))

async function loadSnapshots() {
  isLoading.value = true
  repoPath.value = loadRepoPath()
  status.value = 'Checking'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'Asking IronVault for live snapshot data...'

  try {
    snapshots.value = await listSnapshots(repoPath.value)

    status.value = 'Connected'
    statusClass.value = 'status-ready'
    statusMessage.value = snapshots.value.length === 0
      ? 'The vault replied, but no snapshots were found.'
      : 'Live snapshots loaded from the IronVault command bridge.'

    emptyTitle.value = 'No snapshots found'
    emptyMessage.value = 'This vault is reachable, but it does not have any backup shelves yet.'
  } catch (error) {
    snapshots.value = []
    status.value = 'Needs path'
    statusClass.value = 'status-error'
    statusMessage.value = 'IronVault could not load snapshots from the saved vault path.'
    emptyTitle.value = 'Snapshot bridge waiting'
    emptyMessage.value = error instanceof Error
      ? error.message
      : 'Check Settings and make sure the saved vault path points to a real IronVault repo.'
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  loadSnapshots()
})
</script>

<style scoped>
.snapshots-page {
  display: grid;
  gap: 1.25rem;
}

.snapshots-hero,
.snapshot-status-card,
.summary-card,
.empty-card,
.snapshot-card {
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
.empty-card p {
  color: var(--iv-muted);
  line-height: 1.6;
}

.eyebrow-small {
  margin: 0;
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.snapshot-status-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 1rem;
  align-items: center;
  padding: 1.2rem;
  border-radius: var(--iv-radius-md);
}

.repo-path {
  margin: 0.65rem 0 0;
  padding: 0.85rem;
  overflow-wrap: anywhere;
  border-radius: 12px;
  background: var(--iv-bg-soft);
  color: var(--iv-muted-strong);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.snapshot-status {
  display: grid;
  justify-items: end;
  gap: 0.25rem;
  text-align: right;
}

.snapshot-status small {
  color: var(--iv-muted);
}

.status-dot {
  width: 0.72rem;
  height: 0.72rem;
  border-radius: 999px;
  background: var(--iv-muted);
}

.status-ready {
  background: var(--iv-success);
}

.status-waiting {
  background: var(--iv-warning);
}

.status-error {
  background: var(--iv-danger);
}

.snapshot-summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1rem;
}

.summary-card {
  display: grid;
  gap: 0.45rem;
  padding: 1.2rem;
  border-radius: var(--iv-radius-md);
}

.summary-card span {
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.summary-card strong {
  font-size: clamp(1.35rem, 3vw, 2rem);
  letter-spacing: -0.04em;
}

.summary-card small {
  color: var(--iv-muted);
  line-height: 1.45;
}

.snapshot-list {
  display: grid;
  gap: 0.85rem;
}

.empty-card,
.snapshot-card {
  padding: 1.15rem;
  border-radius: var(--iv-radius-md);
}

.empty-card h3,
.snapshot-card h3 {
  margin: 0.2rem 0 0;
  font-size: 1.25rem;
}

.snapshot-card-top {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
}

.snapshot-size {
  align-self: flex-start;
  padding: 0.42rem 0.72rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-accent-soft);
  color: var(--iv-text);
  font-size: 0.82rem;
  font-weight: 900;
}

.snapshot-metrics {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin-top: 1rem;
}

.snapshot-metrics span {
  padding: 0.6rem 0.8rem;
  border-radius: 999px;
  background: var(--iv-bg-soft);
  color: var(--iv-muted-strong);
  font-weight: 800;
}

.snapshot-metrics strong {
  color: var(--iv-text);
}

@media (max-width: 860px) {
  .snapshots-hero,
  .snapshot-status-card {
    grid-template-columns: 1fr;
  }

  .snapshots-hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .snapshot-status {
    justify-items: start;
    text-align: left;
  }

  .snapshot-summary-grid {
    grid-template-columns: 1fr;
  }
}
</style>
