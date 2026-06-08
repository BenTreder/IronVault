<template>
  <div class="dashboard">
    <h2>Dashboard</h2>

    <div class="stats-grid">
      <div class="stat-card">
        <h3>Last Backup</h3>
        <p class="value">{{ lastBackupTime }}</p>
      </div>

      <div class="stat-card">
        <h3>Repository Size</h3>
        <p class="value">{{ repoSize }}</p>
      </div>

      <div class="stat-card">
        <h3>Total Snapshots</h3>
        <p class="value">{{ snapshotCount }}</p>
      </div>

      <div class="stat-card">
        <h3>Status</h3>
        <p class="value status" :class="statusClass">{{ status }}</p>
      </div>
    </div>

    <div class="quick-actions">
      <h3>Quick Actions</h3>
      <button @click="runBackup" :disabled="isBackingUp">
        {{ isBackingUp ? 'Backing up...' : 'Start Backup' }}
      </button>
      <button @click="runDryRun" :disabled="isBackingUp">
        Dry Run
      </button>
      <button @click="verifyRepo" :disabled="isVerifying">
        {{ isVerifying ? 'Verifying...' : 'Verify Repository' }}
      </button>
    </div>

    <div class="info-section">
      <h3>Repository Information</h3>
      <p><strong>Path:</strong> {{ repoPath }}</p>
      <p><strong>Next Scheduled Backup:</strong> Tomorrow at 02:30</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const lastBackupTime = ref('Never')
const repoSize = ref('0 B')
const snapshotCount = ref(0)
const status = ref('Ready')
const statusClass = ref('ready')
const repoPath = ref('/mnt/backups/ironvault')
const isBackingUp = ref(false)
const isVerifying = ref(false)

async function runBackup() {
  isBackingUp.value = true
  status.value = 'Backing up...'
  // Call Tauri command
  try {
    // await invoke('create_backup', { configPath: '/etc/ironvault/config.toml' })
    status.value = 'Backup completed'
    snapshotCount.value++
  } catch (e) {
    status.value = 'Backup failed'
    statusClass.value = 'error'
  } finally {
    isBackingUp.value = false
  }
}

async function runDryRun() {
  // Call Tauri command
}

async function verifyRepo() {
  isVerifying.value = true
  status.value = 'Verifying...'
  try {
    // await invoke('verify_repository', { repoPath: repoPath.value })
    status.value = 'Verification complete'
  } catch (e) {
    status.value = 'Verification failed'
    statusClass.value = 'error'
  } finally {
    isVerifying.value = false
  }
}

onMounted(() => {
  // Load initial data
})
</script>

<style scoped>
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
}

h2 {
  margin-top: 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: #0f3460;
  border-radius: 8px;
  padding: 1.5rem;
}

.stat-card h3 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
  opacity: 0.8;
}

.value {
  margin: 0;
  font-size: 1.5rem;
  font-weight: bold;
}

.status.ready {
  color: #4ade80;
}

.status.error {
  color: #f87171;
}

.quick-actions {
  margin-bottom: 2rem;
}

.quick-actions h3 {
  margin-top: 0;
}

.quick-actions button {
  background: #e94560;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  margin-right: 0.5rem;
  border-radius: 4px;
  cursor: pointer;
}

.quick-actions button:disabled {
  background: #666;
  cursor: not-allowed;
}

.quick-actions button:hover:not(:disabled) {
  background: #ff6b6b;
}

.info-section {
  background: #0f3460;
  border-radius: 8px;
  padding: 1.5rem;
}

.info-section h3 {
  margin-top: 0;
}

.info-section p {
  margin: 0.5rem 0;
}
</style>
*/
-->