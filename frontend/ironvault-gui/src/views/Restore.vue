<template>
  <div class="restore-page">
    <section class="restore-hero">
      <div>
        <p class="eyebrow-small">Restore</p>
        <h2>Preview before unlock</h2>
        <p>
          Build a restore plan first. IronVault shows what would happen before anything is restored.
        </p>
      </div>

      <button class="iv-button iv-button-primary" type="button" @click="previewRestore" :disabled="isLoading">
        {{ isLoading ? 'Building plan...' : 'Preview restore' }}
      </button>
    </section>

    <section class="restore-grid">
      <article class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow-small">Current vault</p>
            <h3>Restore source</h3>
          </div>
          <span :class="['status-badge', statusClass]">{{ status }}</span>
        </div>

        <p class="repo-path">{{ repoPath }}</p>

        <label class="setting-label" for="snapshotName">Snapshot</label>
        <input
          id="snapshotName"
          v-model="snapshotName"
          class="restore-input"
          type="text"
          placeholder="latest"
        />

        <label class="setting-label" for="targetPath">Restore target</label>
        <input
          id="targetPath"
          v-model="targetPath"
          class="restore-input"
          type="text"
          placeholder="/tmp/ironvault-gui-restore-preview"
        />

        <p class="panel-note">
          {{ statusMessage }}
        </p>
      </article>

      <article class="panel safety-panel">
        <p class="eyebrow-small">Safety promise</p>
        <h3>No overwrite by surprise</h3>
        <p>
          This screen only previews a restore plan. It does not restore files yet. Conflicts stay visible before any unlock step.
        </p>
      </article>
    </section>

    <section v-if="plan" class="plan-grid" aria-label="Restore plan summary">
      <article class="summary-card">
        <span>Safe to restore</span>
        <strong :class="plan.safe_to_restore ? 'text-ready' : 'text-error'">
          {{ plan.safe_to_restore ? 'Yes' : 'No' }}
        </strong>
        <small>{{ plan.safe_to_restore ? 'No target conflicts reported.' : 'Conflicts must be handled first.' }}</small>
      </article>

      <article class="summary-card">
        <span>Files</span>
        <strong>{{ plan.files }}</strong>
        <small>Files included in the restore plan.</small>
      </article>

      <article class="summary-card">
        <span>Directories</span>
        <strong>{{ plan.directories }}</strong>
        <small>Folders that would be recreated.</small>
      </article>

      <article class="summary-card">
        <span>Size</span>
        <strong>{{ formatBytes(plan.total_size) }}</strong>
        <small>Total data represented by this plan.</small>
      </article>
    </section>

    <section v-if="plan" class="panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow-small">Plan details</p>
          <h3>{{ plan.snapshot }} -> {{ plan.target }}</h3>
        </div>
        <span class="status-badge">{{ plan.conflict_count }} conflicts</span>
      </div>

      <div v-if="plan.conflicts.length === 0" class="empty-card">
        <h3>Vault door can open safely</h3>
        <p>
          No restore conflicts were reported for this target. This is still only a preview.
        </p>
      </div>

      <div v-else class="conflict-list">
        <article v-for="conflict in plan.conflicts" :key="`${conflict.source_path}-${conflict.target_path}`" class="conflict-card">
          <span>{{ conflict.kind }}</span>
          <strong>{{ conflict.target_path }}</strong>
          <small>From {{ conflict.source_path }}</small>
        </article>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  formatBytes,
  getRestorePlan,
  type RestorePlanInfo
} from '../lib/ironvaultBridge'
import { loadRepoPath } from '../lib/ironvaultSettings'

const repoPath = ref(loadRepoPath())
const snapshotName = ref('latest')
const targetPath = ref('/tmp/ironvault-gui-restore-preview')
const plan = ref<RestorePlanInfo | null>(null)
const isLoading = ref(false)
const status = ref('Waiting')
const statusClass = ref('status-waiting')
const statusMessage = ref('Choose a snapshot and target path, then preview the restore plan.')

async function previewRestore() {
  isLoading.value = true
  repoPath.value = loadRepoPath()
  status.value = 'Planning'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'Asking IronVault to build a restore plan...'

  try {
    plan.value = await getRestorePlan(
      repoPath.value,
      snapshotName.value.trim() || 'latest',
      targetPath.value.trim() || '/tmp/ironvault-gui-restore-preview'
    )

    status.value = plan.value.safe_to_restore ? 'Safe preview' : 'Conflicts found'
    statusClass.value = plan.value.safe_to_restore ? 'status-ready' : 'status-error'
    statusMessage.value = plan.value.safe_to_restore
      ? 'Restore plan is clean. No files were restored.'
      : 'Restore plan found conflicts. No files were restored.'
  } catch (error) {
    plan.value = null
    status.value = 'Needs attention'
    statusClass.value = 'status-error'
    statusMessage.value = error instanceof Error
      ? error.message
      : 'IronVault could not build a restore preview from the saved vault path.'
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.restore-page {
  display: grid;
  gap: 1.25rem;
}

.restore-hero,
.panel,
.summary-card,
.empty-card,
.conflict-card {
  border: 1px solid var(--iv-border);
  background: color-mix(in srgb, var(--iv-surface) 94%, transparent);
  box-shadow: var(--iv-shadow-soft);
}

.restore-hero {
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

.restore-hero h2 {
  margin: 0.2rem 0 0;
  font-size: clamp(2.1rem, 5vw, 4rem);
  line-height: 0.95;
  letter-spacing: -0.075em;
}

.restore-hero p,
.panel-note,
.safety-panel p,
.empty-card p {
  color: var(--iv-muted);
  line-height: 1.6;
}

.restore-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(280px, 0.75fr);
  gap: 1rem;
}

.panel,
.summary-card,
.empty-card,
.conflict-card {
  border-radius: var(--iv-radius-md);
}

.panel {
  padding: 1.25rem;
}

.panel-heading {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: flex-start;
}

.panel h3,
.empty-card h3 {
  margin: 0.2rem 0 0;
  font-size: 1.35rem;
}

.eyebrow-small {
  margin: 0;
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.repo-path {
  margin: 1rem 0;
  padding: 0.85rem;
  overflow-wrap: anywhere;
  border-radius: 12px;
  background: var(--iv-bg-soft);
  color: var(--iv-muted-strong);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
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

.restore-input {
  min-height: 44px;
  width: 100%;
  padding: 0.7rem 0.9rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-bg-soft);
  color: var(--iv-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
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

.plan-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1rem;
}

.summary-card {
  display: grid;
  gap: 0.45rem;
  padding: 1.2rem;
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

.text-ready {
  color: var(--iv-success);
}

.text-error {
  color: var(--iv-danger);
}

.empty-card {
  margin-top: 1rem;
  padding: 1rem;
}

.conflict-list {
  display: grid;
  gap: 0.75rem;
  margin-top: 1rem;
}

.conflict-card {
  display: grid;
  gap: 0.35rem;
  padding: 1rem;
}

.conflict-card span {
  color: var(--iv-danger);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.conflict-card strong {
  overflow-wrap: anywhere;
}

.conflict-card small {
  color: var(--iv-muted);
  overflow-wrap: anywhere;
}

@media (max-width: 940px) {
  .restore-hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .restore-grid,
  .plan-grid {
    grid-template-columns: 1fr;
  }
}
</style>
