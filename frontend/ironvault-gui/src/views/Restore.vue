<template>
  <div class="restore">
    <h2>Restore</h2>

    <div class="restore-form">
      <div class="form-group">
        <label>Snapshot</label>
        <select v-model="selectedSnapshot">
          <option v-for="s in snapshots" :key="s.name" :value="s.name">
            {{ s.name }}
          </option>
        </select>
      </div>

      <div class="form-group">
        <label>Target Directory</label>
        <input v-model="targetPath" type="text" placeholder="/tmp/restore" />
      </div>

      <button @click="generatePlan" :disabled="isGenerating">
        {{ isGenerating ? 'Generating...' : 'Generate Restore Plan' }}
      </button>
    </div>

    <div v-if="plan" class="plan-view">
      <h3>Restore Plan</h3>
      <p>Snapshot: {{ plan.snapshot }}</p>
      <p>Target: {{ plan.target }}</p>
      <p>Files: {{ plan.files.length }}</p>

      <div class="file-list">
        <div v-for="file in plan.files.slice(0, 10)" :key="file.source" class="file-item">
          {{ file.source }} -> {{ file.target }}
        </div>
        <p v-if="plan.files.length > 10">... and {{ plan.files.length - 10 }} more files</p>
      </div>

      <button @click="executeRestore" class="confirm-button">
        Confirm and Execute Restore
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Snapshot {
  name: string
  created_at: string
}

interface RestoreFile {
  source: string
  target: string
}

interface RestorePlan {
  snapshot: string
  target: string
  files: RestoreFile[]
}

const snapshots = ref<Snapshot[]>([])
const selectedSnapshot = ref('')
const targetPath = ref('/tmp/ironvault-restore')
const isGenerating = ref(false)
const plan = ref<RestorePlan | null>(null)

async function generatePlan() {
  isGenerating.value = true
  try {
    // Generate restore plan
    plan.value = {
      snapshot: selectedSnapshot.value,
      target: targetPath.value,
      files: []
    }
  } finally {
    isGenerating.value = false
  }
}

function executeRestore() {
  // Execute restore
}
</script>

<style scoped>
.restore {
  max-width: 800px;
  margin: 0 auto;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 0.75rem;
  background: #0f3460;
  color: white;
  border: 1px solid #333;
  border-radius: 4px;
}

.plan-view {
  margin-top: 2rem;
  padding: 1.5rem;
  background: #0f3460;
  border-radius: 8px;
}

.file-list {
  max-height: 300px;
  overflow-y: auto;
  margin: 1rem 0;
}

.file-item {
  padding: 0.5rem;
  background: #1a1a2e;
  margin-bottom: 0.5rem;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.8rem;
}

.confirm-button {
  background: #e94560;
  color: white;
  border: none;
  padding: 1rem 2rem;
  border-radius: 4px;
  cursor: pointer;
}
</style>
*/
-->