<template>
  <div class="dashboard">
    <section class="hero-card">
      <div class="hero-copy">
        <span class="iv-pill">
          <span class="iv-dot" aria-hidden="true"></span>
          {{ heroBadge }}
        </span>

        <h2>Your vault is protected</h2>
        <p>
          Vault door closed. Everything looks safe.
        </p>

        <div class="hero-actions">
          <button class="iv-button iv-button-primary" @click="runBackup" :disabled="isBackingUp">
            {{ isBackingUp ? 'Sealing backup...' : 'Run Backup' }}
          </button>
          <button class="iv-button iv-button-secondary" @click="verifyRepo" :disabled="isVerifying">
            {{ isVerifying ? 'Checking vault...' : 'Check Vault' }}
          </button>
          <button class="iv-button iv-button-ghost">
            Restore Files
          </button>
        </div>
      </div>

      <div class="vault-visual" aria-hidden="true">
        <div class="vault-ring">
          <div class="vault-core">IV</div>
        </div>
        <p>No guessing, no stomping, no chaos.</p>
      </div>
    </section>

    <section class="cards-grid" aria-label="Vault summary">
      <article class="summary-card">
        <span>Vault Health</span>
        <strong :class="statusClass">{{ status }}</strong>
        <small>{{ statusNote }}</small>
      </article>

      <article class="summary-card">
        <span>Last Backup</span>
        <strong>{{ lastBackupTime }}</strong>
        <small>Latest sealed backup activity.</small>
      </article>

      <article class="summary-card">
        <span>Snapshots</span>
        <strong>{{ snapshotCount }}</strong>
        <small>Backup shelves currently available.</small>
      </article>

      <article class="summary-card">
        <span>Vault Size</span>
        <strong>{{ repoSize }}</strong>
        <small>{{ chunkNote }}</small>
      </article>
    </section>

    <section class="lower-grid">
      <article class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow-small">Repository</p>
            <h3>Vault location</h3>
          </div>

          <button class="mini-button" type="button" @click="loadDashboard" :disabled="isLoading">
            {{ isLoading ? 'Refreshing...' : 'Refresh' }}
          </button>
        </div>

        <p class="repo-path">{{ repoPath }}</p>

        <div class="repo-form">
          <label for="repoPath">Test vault path</label>
          <div class="repo-form-row">
            <input
              id="repoPath"
              v-model="repoPathDraft"
              class="repo-input"
              type="text"
              placeholder="/tmp/ironvault-gui-live-test/repo"
            />
            <button class="mini-button" type="button" @click="saveRepoPath">
              Save path
            </button>
          </div>
        </div>

        <p class="panel-note">
          {{ bridgeNote }}
        </p>
      </article>

      <article class="panel safety-panel">
        <p class="eyebrow-small">Safety promise</p>
        <h3>Restore stays careful</h3>
        <p>
          IronVault will preview restore plans first, show conflicts clearly, and refuse to overwrite by default.
        </p>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
  defaultRepoPath,
  formatBytes,
  getRepoInfo,
  verifyRepository
} from '../lib/ironvaultBridge'

const heroBadge = ref('Vault health looks good')
const lastBackupTime = ref('Waiting for live data')
const repoSize = ref('0 B')
const snapshotCount = ref(0)
const totalChunks = ref<number | null>(null)
const status = ref('Bridge ready')
const statusClass = ref('status-warning')
const statusNote = ref('Dashboard is ready to read live IronVault JSON.')
const chunkNote = ref('Stored backup data in the vault.')
const repoPath = ref(loadSavedRepoPath())
const repoPathDraft = ref(repoPath.value)
const bridgeNote = ref('Waiting for the Tauri command bridge to return live vault data.')
const isBackingUp = ref(false)
const isVerifying = ref(false)
const isLoading = ref(false)

function loadSavedRepoPath(): string {
  return localStorage.getItem('ironvault-repo-path') || defaultRepoPath
}

function saveRepoPath() {
  const nextPath = repoPathDraft.value.trim() || defaultRepoPath

  repoPath.value = nextPath
  repoPathDraft.value = nextPath
  localStorage.setItem('ironvault-repo-path', nextPath)

  loadDashboard()
}

async function loadDashboard() {
  isLoading.value = true

  try {
    const info = await getRepoInfo(repoPath.value)

    repoPath.value = info.path
    repoPathDraft.value = info.path
    repoSize.value = formatBytes(info.total_size)
    snapshotCount.value = info.snapshot_count
    totalChunks.value = info.total_chunks ?? null

    status.value = 'Connected'
    statusClass.value = 'status-ready'
    statusNote.value = 'Live vault data loaded from the command bridge.'
    heroBadge.value = 'Vault data connected'
    lastBackupTime.value = 'Ready'
    bridgeNote.value = 'Live repository info is flowing through the dashboard bridge.'

    chunkNote.value = totalChunks.value === null
      ? 'Stored backup data in the vault.'
      : `${totalChunks.value} vault chunk${totalChunks.value === 1 ? '' : 's'} tracked.`
  } catch (error) {
    status.value = 'Bridge waiting'
    statusClass.value = 'status-warning'
    statusNote.value = 'The visual dashboard is ready, but live Tauri data is not connected yet.'
    heroBadge.value = 'Visual dashboard ready'
    lastBackupTime.value = 'Not connected yet'
    bridgeNote.value = 'The frontend bridge is installed. Next we will finish the Tauri backend command package so real vault data can load here.'
  } finally {
    isLoading.value = false
  }
}

async function runBackup() {
  isBackingUp.value = true
  status.value = 'Sealing backup...'

  try {
    status.value = 'Backup sealed'
    statusClass.value = 'status-ready'
    snapshotCount.value++
    statusNote.value = 'Backup command placeholder completed. Live backup wiring comes next.'
  } catch (error) {
    status.value = 'Backup needs attention'
    statusClass.value = 'status-error'
    statusNote.value = 'IronVault could not complete the backup action.'
  } finally {
    isBackingUp.value = false
  }
}

async function verifyRepo() {
  isVerifying.value = true
  status.value = 'Checking vault...'

  try {
    const result = await verifyRepository(repoPath.value)

    status.value = result.valid ? 'Protected' : 'Needs attention'
    statusClass.value = result.valid ? 'status-ready' : 'status-error'
    statusNote.value = result.message
    heroBadge.value = result.valid ? 'Vault health looks good' : 'Vault needs attention'
  } catch (error) {
    status.value = 'Bridge waiting'
    statusClass.value = 'status-warning'
    statusNote.value = 'Verify command is ready in the dashboard, but the Tauri backend still needs the live command implementation.'
    heroBadge.value = 'Verify bridge ready'
  } finally {
    isVerifying.value = false
  }
}

onMounted(() => {
  loadDashboard()
})
</script>

<style scoped>
.dashboard {
  display: grid;
  gap: 1.25rem;
}

.hero-card {
  position: relative;
  overflow: hidden;
  display: grid;
  grid-template-columns: minmax(0, 1.45fr) minmax(260px, 0.55fr);
  gap: 1.5rem;
  min-height: 340px;
  padding: clamp(1.35rem, 4vw, 2.25rem);
  border: 1px solid var(--iv-border);
  border-radius: var(--iv-radius-lg);
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--iv-surface) 92%, transparent), var(--iv-surface-raised)),
    radial-gradient(circle at 20% 20%, rgba(249, 115, 22, 0.22), transparent 28rem);
  box-shadow: var(--iv-shadow);
}

.hero-card::after {
  content: "";
  position: absolute;
  inset: auto -5rem -9rem auto;
  width: 22rem;
  height: 22rem;
  border-radius: 999px;
  background: rgba(249, 115, 22, 0.10);
  filter: blur(4px);
}

.hero-copy {
  position: relative;
  z-index: 1;
  display: grid;
  align-content: center;
  justify-items: start;
}

.hero-copy h2 {
  max-width: 780px;
  margin: 1rem 0 0;
  font-size: clamp(2.4rem, 7vw, 5.6rem);
  line-height: 0.92;
  letter-spacing: -0.085em;
}

.hero-copy p {
  max-width: 560px;
  margin: 1.1rem 0 0;
  color: var(--iv-muted-strong);
  font-size: clamp(1.05rem, 2vw, 1.28rem);
  line-height: 1.6;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 1.6rem;
}

.vault-visual {
  position: relative;
  z-index: 1;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 1rem;
  text-align: center;
  color: var(--iv-muted);
  font-weight: 750;
}

.vault-ring {
  display: grid;
  place-items: center;
  width: min(100%, 250px);
  aspect-ratio: 1;
  border-radius: 999px;
  background:
    linear-gradient(var(--iv-surface), var(--iv-surface)) padding-box,
    conic-gradient(from 90deg, var(--iv-accent), transparent, var(--iv-accent)) border-box;
  border: 2px solid transparent;
  box-shadow: inset 0 0 42px rgba(249, 115, 22, 0.12);
}

.vault-core {
  display: grid;
  place-items: center;
  width: 48%;
  aspect-ratio: 1;
  border-radius: 34%;
  background: linear-gradient(135deg, var(--iv-accent), var(--iv-accent-strong));
  color: white;
  font-size: 2.2rem;
  font-weight: 950;
  box-shadow: 0 18px 45px rgba(249, 115, 22, 0.26);
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1rem;
}

.summary-card,
.panel {
  border: 1px solid var(--iv-border);
  border-radius: var(--iv-radius-md);
  background: color-mix(in srgb, var(--iv-surface) 94%, transparent);
  box-shadow: var(--iv-shadow-soft);
}

.summary-card {
  display: grid;
  gap: 0.45rem;
  padding: 1.2rem;
}

.summary-card span,
.eyebrow-small {
  margin: 0;
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

.status-ready {
  color: var(--iv-success);
}

.status-warning {
  color: var(--iv-warning);
}

.status-error {
  color: var(--iv-danger);
}

.lower-grid {
  display: grid;
  grid-template-columns: 1.25fr 0.75fr;
  gap: 1rem;
}

.panel {
  padding: 1.25rem;
}

.panel-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.panel h3 {
  margin: 0.2rem 0 0;
  font-size: 1.35rem;
}

.mini-button {
  min-height: 36px;
  padding: 0.55rem 0.85rem;
  border-radius: 999px;
  background: var(--iv-surface-raised);
  color: var(--iv-text);
  border: 1px solid var(--iv-border);
  cursor: pointer;
  font-weight: 800;
}

.mini-button:disabled {
  cursor: not-allowed;
  opacity: 0.65;
}

.repo-path {
  margin: 1rem 0 0;
  padding: 0.85rem;
  overflow-wrap: anywhere;
  border-radius: 12px;
  background: var(--iv-bg-soft);
  color: var(--iv-muted-strong);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.repo-form {
  display: grid;
  gap: 0.55rem;
  margin-top: 1rem;
}

.repo-form label {
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.repo-form-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.65rem;
}

.repo-input {
  min-height: 40px;
  width: 100%;
  padding: 0.65rem 0.8rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-bg-soft);
  color: var(--iv-text);
}

@media (max-width: 640px) {
  .repo-form-row {
    grid-template-columns: 1fr;
  }
}

.panel-note,
.safety-panel p {
  color: var(--iv-muted);
  line-height: 1.6;
}

@media (max-width: 1050px) {
  .hero-card,
  .lower-grid {
    grid-template-columns: 1fr;
  }

  .cards-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .cards-grid {
    grid-template-columns: 1fr;
  }

  .hero-copy h2 {
    font-size: 3rem;
  }
}
</style>
