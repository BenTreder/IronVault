<template>
  <div class="backup">
    <h2>Backup</h2>

    <div class="backup-controls">
      <button @click="startBackup" :disabled="isRunning" class="primary-button">
        {{ isRunning ? 'Backing up...' : 'Start Backup' }}
      </button>
      <button @click="startDryRun" :disabled="isRunning" class="secondary-button">
        Dry Run
      </button>
    </div>

    <div v-if="isRunning" class="progress-section">
      <h3>Progress</h3>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: progress + '%' }"></div>
      </div>
      <p>{{ currentFile }}</p>
      <div class="stats">
        <p>Chunks added: {{ chunksAdded }}</p>
        <p>Chunks reused: {{ chunksReused }}</p>
        <p>Bytes read: {{ formatBytes(bytesRead) }}</p>
        <p>Bytes stored: {{ formatBytes(bytesWritten) }}</p>
      </div>
    </div>

    <div class="config-section">
      <h3>Configuration</h3>
      <p>Config file: /etc/ironvault/config.toml</p>
      <button @click="editConfig">Edit Configuration</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const isRunning = ref(false)
const progress = ref(0)
const currentFile = ref('')
const chunksAdded = ref(0)
const chunksReused = ref(0)
const bytesRead = ref(0)
const bytesWritten = ref(0)

function formatBytes(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(2) + ' MiB'
}

async function startBackup() {
  isRunning.value = true
  progress.value = 0

  // Simulate progress
  const interval = setInterval(() => {
    progress.value += 10
    if (progress.value >= 100) {
      clearInterval(interval)
      isRunning.value = false
    }
  }, 500)
}

async function startDryRun() {
  // Implement dry run
}

function editConfig() {
  // Open config editor
}
</script>

<style scoped>
.backup {
  max-width: 800px;
  margin: 0 auto;
}

.primary-button, .secondary-button {
  padding: 1rem 2rem;
  font-size: 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  margin-right: 1rem;
}

.primary-button {
  background: #e94560;
  color: white;
}

.secondary-button {
  background: #0f3460;
  color: white;
  border: 1px solid #e94560;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.progress-section {
  margin-top: 2rem;
  padding: 1.5rem;
  background: #0f3460;
  border-radius: 8px;
}

.progress-bar {
  height: 20px;
  background: #1a1a2e;
  border-radius: 10px;
  overflow: hidden;
  margin: 1rem 0;
}

.progress-fill {
  height: 100%;
  background: #e94560;
  transition: width 0.3s;
}

.stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 0.5rem;
  margin-top: 1rem;
}

.config-section {
  margin-top: 2rem;
  padding: 1.5rem;
  background: #0f3460;
  border-radius: 8px;
}
</style>
*/
-->