<template>
  <div class="settings-page">
    <section class="settings-hero">
      <div>
        <p class="eyebrow-small">Settings</p>
        <h2>Vault control room</h2>
        <p>
          Choose which vault the desktop app reads from. This path powers Dashboard, Snapshots, Restore, and the safety checks.
        </p>
      </div>

      <span :class="['status-badge', statusClass]">{{ status }}</span>
    </section>

    <section class="settings-grid">
      <article class="panel main-panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow-small">Vault path</p>
            <h3>Saved repository</h3>
          </div>
        </div>

        <label class="setting-label" for="repoPath">Repository path</label>
        <input
          id="repoPath"
          v-model="repoPath"
          class="settings-input"
          type="text"
          placeholder="/mnt/backups/ironvault"
        />

        <div class="button-row">
          <button class="iv-button iv-button-primary" type="button" @click="savePath">
            Save path
          </button>
          <button class="iv-button iv-button-secondary" type="button" @click="useTestVault">
            Use test vault
          </button>
          <button class="iv-button iv-button-secondary" type="button" @click="resetPath">
            Reset default
          </button>
        </div>

        <p class="panel-note">
          {{ message }}
        </p>
      </article>

      <article class="panel helper-panel">
        <p class="eyebrow-small">Demo helper</p>
        <h3>Set up a safe test vault</h3>
        <p>
          Creates a small source folder, writes a matching config, initializes the test repo if needed, then saves the paths for the GUI.
        </p>

        <button class="iv-button iv-button-primary" type="button" @click="setupDemoVault" :disabled="isSettingUp">
          {{ isSettingUp ? 'Setting up...' : 'Set up test vault' }}
        </button>

        <div v-if="setupResult" class="result-card">
          <p class="eyebrow-small">Created paths</p>
          <dl>
            <div>
              <dt>Vault</dt>
              <dd>{{ setupResult.repo_path }}</dd>
            </div>
            <div>
              <dt>Config</dt>
              <dd>{{ setupResult.config_path }}</dd>
            </div>
            <div>
              <dt>Source</dt>
              <dd>{{ setupResult.source_path }}</dd>
            </div>
          </dl>
        </div>
      </article>

      <article class="panel helper-panel real-helper">
        <p class="eyebrow-small">Real setup helper</p>
        <h3>Create a backup config for your folder</h3>
        <p>
          Enter a source folder, vault repo path, and config file path. IronVault will write a valid config and initialize the repo if needed.
        </p>

        <label class="setting-label" for="customSourcePath">Source folder</label>
        <input
          id="customSourcePath"
          v-model="customSourcePath"
          class="settings-input"
          type="text"
          placeholder="/home/chr0nichacker/Documents"
        />

        <label class="setting-label" for="customRepoPath">Vault repo path</label>
        <input
          id="customRepoPath"
          v-model="customRepoPath"
          class="settings-input"
          type="text"
          placeholder="/home/chr0nichacker/IronVaultBackups/repo"
        />

        <label class="setting-label" for="customConfigPath">Config file path</label>
        <input
          id="customConfigPath"
          v-model="customConfigPath"
          class="settings-input"
          type="text"
          placeholder="/home/chr0nichacker/.config/ironvault/ironvault.toml"
        />

        <button class="iv-button iv-button-primary setup-real-button" type="button" @click="setupRealVault" :disabled="isCustomSettingUp">
          {{ isCustomSettingUp ? 'Creating...' : 'Create real backup setup' }}
        </button>

        <div v-if="customSetupResult" class="result-card">
          <p class="eyebrow-small">Saved paths</p>
          <dl>
            <div>
              <dt>Vault</dt>
              <dd>{{ customSetupResult.repo_path }}</dd>
            </div>
            <div>
              <dt>Config</dt>
              <dd>{{ customSetupResult.config_path }}</dd>
            </div>
            <div>
              <dt>Source</dt>
              <dd>{{ customSetupResult.source_path }}</dd>
            </div>
          </dl>
        </div>
      </article>
    </section>

    <section class="panel note-panel">
      <p class="eyebrow-small">Reminder</p>
      <h3>Settings path is the app’s source of truth</h3>
      <p>
        Backup uses its own config file path, but Dashboard, Snapshots, and Restore read from the repository path saved here.
      </p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  defaultRepoPath,
  setupCustomVault,
  setupTestVault,
  type SetupCustomVaultResult,
  type SetupTestVaultResult
} from '../lib/ironvaultBridge'
import {
  loadRepoPath,
  resetRepoPath,
  saveRepoPath
} from '../lib/ironvaultSettings'

const backupConfigStorageKey = 'ironvault-backup-config-path'
const testRepoPath = '/tmp/ironvault-gui-live-test/repo'

const repoPath = ref(loadRepoPath())
const message = ref('Vault path loaded from local settings.')
const status = ref('Ready')
const statusClass = ref('status-ready')
const isSettingUp = ref(false)
const isCustomSettingUp = ref(false)
const setupResult = ref<SetupTestVaultResult | null>(null)
const customSetupResult = ref<SetupCustomVaultResult | null>(null)

const customSourcePath = ref(localStorage.getItem('ironvault-custom-source-path') || '/home/chr0nichacker/Documents')
const customRepoPath = ref(localStorage.getItem('ironvault-custom-repo-path') || '/home/chr0nichacker/IronVaultBackups/repo')
const customConfigPath = ref(localStorage.getItem('ironvault-custom-config-path') || '/home/chr0nichacker/.config/ironvault/ironvault.toml')

function markSaved(savedPath: string, savedMessage: string) {
  repoPath.value = savedPath
  message.value = savedMessage
  status.value = 'Saved'
  statusClass.value = 'status-ready'
}

function savePath() {
  const savedPath = saveRepoPath(repoPath.value)
  markSaved(savedPath, `Saved vault path: ${savedPath}`)
}

function useTestVault() {
  const savedPath = saveRepoPath(testRepoPath)
  markSaved(savedPath, `Using test vault: ${savedPath}`)
}

function resetPath() {
  const savedPath = resetRepoPath()
  markSaved(savedPath || defaultRepoPath, `Reset to default vault path: ${savedPath || defaultRepoPath}`)
}

async function setupDemoVault() {
  isSettingUp.value = true
  setupResult.value = null
  status.value = 'Setting up'
  statusClass.value = 'status-waiting'
  message.value = 'Creating GUI test vault helper files...'

  try {
    const result = await setupTestVault()
    setupResult.value = result

    saveRepoPath(result.repo_path)
    localStorage.setItem(backupConfigStorageKey, result.config_path)

    repoPath.value = result.repo_path
    status.value = 'Test vault ready'
    statusClass.value = 'status-ready'
    message.value = `${result.message} Backup config saved for the Backup page.`
  } catch (error) {
    status.value = 'Needs attention'
    statusClass.value = 'status-error'
    message.value = error instanceof Error
      ? error.message
      : 'Could not set up the GUI test vault.'
  } finally {
    isSettingUp.value = false
  }
}

async function setupRealVault() {
  isCustomSettingUp.value = true
  customSetupResult.value = null
  status.value = 'Setting up'
  statusClass.value = 'status-waiting'
  message.value = 'Creating real vault config and initializing the repo if needed...'

  localStorage.setItem('ironvault-custom-source-path', customSourcePath.value.trim())
  localStorage.setItem('ironvault-custom-repo-path', customRepoPath.value.trim())
  localStorage.setItem('ironvault-custom-config-path', customConfigPath.value.trim())

  try {
    const result = await setupCustomVault(
      customSourcePath.value.trim(),
      customRepoPath.value.trim(),
      customConfigPath.value.trim()
    )

    customSetupResult.value = result

    saveRepoPath(result.repo_path)
    localStorage.setItem(backupConfigStorageKey, result.config_path)

    repoPath.value = result.repo_path
    status.value = 'Real vault ready'
    statusClass.value = 'status-ready'
    message.value = `${result.message} Backup config saved for the Backup page.`
  } catch (error) {
    status.value = 'Needs attention'
    statusClass.value = 'status-error'
    message.value = error instanceof Error
      ? error.message
      : 'Could not set up the real vault config.'
  } finally {
    isCustomSettingUp.value = false
  }
}
</script>

<style scoped>
.settings-page {
  display: grid;
  gap: 1.25rem;
}

.settings-hero,
.panel,
.result-card {
  border: 1px solid var(--iv-border);
  background: color-mix(in srgb, var(--iv-surface) 94%, transparent);
  box-shadow: var(--iv-shadow-soft);
}

.settings-hero {
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

.settings-hero h2 {
  margin: 0.2rem 0 0;
  font-size: clamp(2.1rem, 5vw, 4rem);
  line-height: 0.95;
  letter-spacing: -0.075em;
}

.settings-hero p,
.panel-note,
.helper-panel p,
.note-panel p {
  color: var(--iv-muted);
  line-height: 1.6;
}

.settings-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(280px, 0.95fr);
  gap: 1rem;
}

.panel,
.result-card {
  padding: 1.25rem;
  border-radius: var(--iv-radius-md);
}

.main-panel,
.real-helper,
.note-panel {
  grid-column: 1 / -1;
}

.panel-heading {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: flex-start;
}

.panel h3,
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

.setting-label {
  display: block;
  margin: 1rem 0 0.55rem;
  color: var(--iv-muted);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-input {
  min-height: 44px;
  width: 100%;
  padding: 0.7rem 0.9rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-bg-soft);
  color: var(--iv-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.button-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 1rem;
}

.setup-real-button {
  margin-top: 1rem;
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

.result-card {
  margin-top: 1rem;
  background: var(--iv-bg-soft);
}

.result-card dl {
  display: grid;
  gap: 0.7rem;
  margin: 1rem 0 0;
}

.result-card div {
  display: grid;
  gap: 0.25rem;
}

.result-card dt {
  color: var(--iv-muted);
  font-size: 0.72rem;
  font-weight: 900;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.result-card dd {
  margin: 0;
  overflow-wrap: anywhere;
  color: var(--iv-muted-strong);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

@media (max-width: 940px) {
  .settings-hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .settings-grid {
    grid-template-columns: 1fr;
  }

  .main-panel,
  .real-helper,
  .note-panel {
    grid-column: auto;
  }
}
</style>
