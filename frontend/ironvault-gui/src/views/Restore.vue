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

      <button class="iv-button iv-button-primary" type="button" @click="previewRestore" :disabled="isLoading || isRestoring">
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
        <select
          id="snapshotName"
          v-model="snapshotName"
          class="restore-input"
          @change="clearPlanAfterEdit"
        >
          <option value="" disabled>Select a snapshot</option>
          <option v-for="snapshot in snapshots" :key="snapshot.name" :value="snapshot.name">
            {{ snapshot.name }} · {{ snapshotFileCount(snapshot) }} files · {{ formatBytes(snapshot.total_size || 0) }}
          </option>
        </select>

        <p class="mini-note">
          {{ snapshotStatusMessage }}
        </p>

        <label class="setting-label" for="targetPath">Restore into folder</label>
        <input
          id="targetPath"
          v-model="targetPath"
          class="restore-input"
          type="text"
          placeholder="/tmp/ironvault-gui-restore-preview"
          @input="clearPlanAfterEdit"
        />

        <p class="panel-note">
          {{ statusMessage }}
        </p>
      </article>

      <article class="panel safety-panel">
        <p class="eyebrow-small">Safety promise</p>
        <h3>No overwrite by surprise</h3>
        <p>
          Restore execution stays locked until the preview is clean and you type RESTORE. The command uses refuse mode, so existing target files block the restore.
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
          No restore conflicts were reported for this target. Restore is still locked until you confirm below.
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

    <section v-if="plan" class="panel execution-panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow-small">Guarded restore</p>
          <h3>Unlock files only after confirmation</h3>
        </div>
        <span :class="['status-badge', canRestore ? 'status-ready' : 'status-error']">
          {{ canRestore ? 'Unlocked' : 'Locked' }}
        </span>
      </div>

      <p class="panel-note">
        Type RESTORE to enable the restore button. IronVault will use --if-exists refuse, so it will not overwrite existing files.
      </p>

      <label class="setting-label" for="restoreConfirm">Confirmation</label>
      <input
        id="restoreConfirm"
        v-model="restoreConfirmation"
        class="restore-input"
        type="text"
        placeholder="Type RESTORE"
      />

      <div class="execution-actions">
        <button class="iv-button iv-button-primary" type="button" @click="executeRestore" :disabled="!canRestore || isRestoring">
          {{ isRestoring ? 'Restoring...' : 'Restore now' }}
        </button>
      </div>

      <div v-if="restoreResult" class="result-card">
        <p class="eyebrow-small">Result</p>
        <h3>{{ restoreResult.success ? 'Restore complete' : 'Restore failed' }}</h3>
        <pre>{{ restoreResult.message }}</pre>
      <p class="field-help">
        After restore, check inside the original folder name under your restore folder. Example: restore-folder/BackupTest/TestFile.txt.
      </p>
      <p class="field-help">
        Restored files are placed inside the folder you selected, using the original backed-up folder name.
      </p>
      </div>
    </section>
  </div>

    <section v-if="plan" class="panel restore-location-guide">
      <div class="panel-heading">
        <div>
          <p class="eyebrow-small">Restore location</p>
          <h3>Where will my files go?</h3>
        </div>
        <span class="status-badge">Safe layout</span>
      </div>

      <p>
        {{ restoreLocationSummary }}
      </p>

      <div class="path-example">
        <span>Example restored file path</span>
        <strong>{{ restoreLocationExample }}</strong>
      </div>

      <p class="field-help">
        This keeps the original folder structure together. So if you backed up a folder named BackupTest and restore into /tmp/restore, your file will be under /tmp/restore/BackupTest/.
      </p>
    </section>

</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  formatBytes,
  getRestorePlan,
  listSnapshots,
  restoreSnapshot,
  snapshotFileCount,
  type RestorePlanInfo,
  type RestoreResult,
  type SnapshotInfo
} from '../lib/ironvaultBridge'
import { loadRepoPath } from '../lib/ironvaultSettings'

const repoPath = ref(loadRepoPath())
const snapshots = ref<SnapshotInfo[]>([])
const snapshotName = ref('')
const snapshotStatusMessage = ref('Loading snapshots from the saved vault path...')
const targetPath = ref('/tmp/ironvault-gui-restore-preview')
const plan = ref<RestorePlanInfo | null>(null)
const restoreResult = ref<RestoreResult | null>(null)
const restoreConfirmation = ref('')
const isLoading = ref(false)
const isRestoring = ref(false)
const status = ref('Waiting')
const statusClass = ref('status-waiting')
const statusMessage = ref('Choose a snapshot and restore folder, then preview the restore plan.')

const planHasRestorableItems = computed(() =>
  Boolean(plan.value && (plan.value.files > 0 || plan.value.directories > 0 || plan.value.symlinks > 0))
)

const cleanTargetPath = computed(() => targetPath.value.trim().replace(/\/+$/, ''))

const restoreLocationExample = computed(() => {
  const base = cleanTargetPath.value || '/tmp/ironvault-restore'
  return `${base}/OriginalFolderName/YourFile.txt`
})

const restoreLocationSummary = computed(() => {
  const base = cleanTargetPath.value || 'the folder you choose'
  return `IronVault will restore the original backed-up folder inside ${base}.`
})

const canRestore = computed(() =>
  Boolean(plan.value?.safe_to_restore) &&
  planHasRestorableItems.value &&
  restoreConfirmation.value === 'RESTORE' &&
  !isLoading.value &&
  !isRestoring.value
)

function clearPlanAfterEdit() {
  plan.value = null
  restoreResult.value = null
  restoreConfirmation.value = ''
  status.value = 'Waiting'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'Selection changed. Preview restore again before unlocking files.'
}

async function loadSnapshotsForRestore() {
  repoPath.value = loadRepoPath()
  snapshotStatusMessage.value = 'Loading snapshots from the saved vault path...'

  try {
    snapshots.value = await listSnapshots(repoPath.value)
    snapshotName.value = snapshots.value[0]?.name || ''
    snapshotStatusMessage.value = snapshots.value.length > 0
      ? `${snapshots.value.length} snapshot(s) available.`
      : 'No snapshots found. Create a backup before restoring.'
  } catch (error) {
    snapshots.value = []
    snapshotName.value = ''
    snapshotStatusMessage.value = error instanceof Error
      ? error.message
      : 'Could not load snapshots from the saved vault path.'
  }
}

async function previewRestore() {
  isLoading.value = true
  restoreResult.value = null
  restoreConfirmation.value = ''
  repoPath.value = loadRepoPath()
  status.value = 'Planning'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'Asking IronVault to build a restore plan...'

  try {
    const selectedSnapshot = snapshotName.value.trim()

    if (!selectedSnapshot) {
      plan.value = null
      status.value = 'Needs backup'
      statusClass.value = 'status-error'
      statusMessage.value = 'No snapshot is selected. Create or select a snapshot before previewing restore.'
      return
    }

    plan.value = await getRestorePlan(
      repoPath.value,
      selectedSnapshot,
      targetPath.value.trim() || '/tmp/ironvault-gui-restore-preview'
    )

    const hasItems = plan.value.files > 0 || plan.value.directories > 0 || plan.value.symlinks > 0

    if (!hasItems) {
      status.value = 'Empty plan'
      statusClass.value = 'status-error'
      statusMessage.value = 'Restore plan is empty. IronVault will not unlock a 0-file restore.'
      return
    }

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

onMounted(loadSnapshotsForRestore)

async function executeRestore() {
  if (!plan.value || !canRestore.value) {
    return
  }

  isRestoring.value = true
  status.value = 'Restoring'
  statusClass.value = 'status-waiting'
  statusMessage.value = 'IronVault is restoring with overwrite refusal enabled...'

  try {
    restoreResult.value = await restoreSnapshot(
      repoPath.value,
      plan.value.snapshot,
      plan.value.target
    )

    status.value = 'Restored'
    statusClass.value = 'status-ready'
    statusMessage.value = 'Restore finished. Vault door closed behind us.'
  } catch (error) {
    restoreResult.value = {
      success: false,
      message: error instanceof Error
        ? error.message
        : 'IronVault could not complete the guarded restore.'
    }

    status.value = 'Restore blocked'
    statusClass.value = 'status-error'
    statusMessage.value = 'Restore did not complete. No overwrite mode remains enforced.'
  } finally {
    isRestoring.value = false
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
.conflict-card,
.result-card {
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
.conflict-card,
.result-card {
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
.empty-card h3,
.result-card h3 {
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

.mini-note {
  margin: 0.6rem 0 0;
  color: var(--iv-muted);
  font-size: 0.82rem;
  line-height: 1.45;
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

.empty-card,
.result-card {
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

.execution-panel {
  border-color: color-mix(in srgb, var(--iv-accent) 34%, var(--iv-border));
}

.execution-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 1rem;
}

.result-card pre {
  margin: 1rem 0 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  color: var(--iv-muted-strong);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
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
.field-help {
  margin: 0.55rem 0 0;
  color: var(--iv-muted);
  font-size: 0.84rem;
  line-height: 1.45;
}

.restore-location-guide p {
  color: var(--iv-muted);
  line-height: 1.6;
}

.path-example {
  display: grid;
  gap: 0.35rem;
  margin-top: 1rem;
  padding: 1rem;
  border: 1px solid var(--iv-border);
  border-radius: 16px;
  background: var(--iv-bg-soft);
}

.path-example span {
  color: var(--iv-muted);
  font-size: 0.72rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.path-example strong {
  overflow-wrap: anywhere;
  color: var(--iv-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.95rem;
}

.field-help {
  margin: 0.55rem 0 0;
  color: var(--iv-muted);
  font-size: 0.84rem;
  line-height: 1.45;
}

</style>
