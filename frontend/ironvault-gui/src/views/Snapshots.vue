<template>
  <div class="snapshots">
    <h2>Snapshots</h2>

    <div class="snapshot-list">
      <div v-for="snapshot in snapshots" :key="snapshot.name" class="snapshot-card">
        <div class="snapshot-info">
          <h3>{{ snapshot.name }}</h3>
          <p class="meta">
            Created: {{ formatDate(snapshot.created_at) }} |
            Files: {{ snapshot.file_count }} |
            Size: {{ formatBytes(snapshot.total_size) }}
          </p>
        </div>
        <div class="snapshot-actions">
          <button @click="restore(snapshot)">Restore</button>
          <button @click="deleteSnapshot(snapshot)">Delete</button>
        </div>
      </div>
    </div>

    <div v-if="snapshots.length === 0" class="empty-state">
      <p>No snapshots found.</p>
      <button @click="createBackup">Create your first backup</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Snapshot {
  name: string
  created_at: string
  file_count: number
  total_size: number
}

const snapshots = ref<Snapshot[]>([])

function formatDate(date: string): string {
  return new Date(date).toLocaleString()
}

function formatBytes(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(2) + ' MiB'
}

function restore(snapshot: Snapshot) {
  // Open restore dialog
}

function deleteSnapshot(snapshot: Snapshot) {
  // Delete confirmation
}

function createBackup() {
  // Start backup
}
</script>

<style scoped>
.snapshots {
  max-width: 1000px;
  margin: 0 auto;
}

.snapshot-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.snapshot-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: #0f3460;
  border-radius: 8px;
}

.snapshot-info h3 {
  margin: 0 0 0.5rem 0;
}

.meta {
  margin: 0;
  opacity: 0.8;
  font-size: 0.9rem;
}

.snapshot-actions button {
  background: #0f3460;
  color: white;
  border: 1px solid #e94560;
  padding: 0.5rem 1rem;
  margin-left: 0.5rem;
  border-radius: 4px;
  cursor: pointer;
}

.snapshot-actions button:hover {
  background: #e94560;
}

.empty-state {
  text-align: center;
  padding: 3rem;
  background: #0f3460;
  border-radius: 8px;
}
</style>
*/
-->