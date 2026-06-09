<template>
  <div class="settings-page">
    <section class="settings-hero">
      <div>
        <p class="eyebrow-small">Settings</p>
        <h2>Vault control room</h2>
        <p>
          Choose the vault IronVault should monitor. The dashboard will use this saved path when it loads live vault status.
        </p>
      </div>
    </section>

    <section class="settings-grid">
      <article class="settings-card">
        <div class="settings-card-heading">
          <div>
            <p class="eyebrow-small">Repository</p>
            <h3>Current vault path</h3>
          </div>
          <span class="settings-badge">{{ pathChanged ? 'Unsaved' : 'Saved' }}</span>
        </div>

        <label class="setting-label" for="repoPath">Vault repo path</label>
        <div class="repo-form-row">
          <input
            id="repoPath"
            v-model="repoPathDraft"
            class="repo-input"
            type="text"
            placeholder="/tmp/ironvault-gui-live-test/repo"
          />
          <button class="iv-button iv-button-primary" type="button" @click="savePath">
            Save path
          </button>
        </div>

        <p class="settings-note">
          {{ saveMessage }}
        </p>

        <div class="settings-actions">
          <button class="iv-button iv-button-secondary" type="button" @click="useTestVault">
            Use test vault
          </button>
          <button class="iv-button iv-button-ghost" type="button" @click="restoreDefaultPath">
            Reset default
          </button>
        </div>
      </article>

      <article class="settings-card">
        <p class="eyebrow-small">Safety</p>
        <h3>Restore rules stay protective</h3>
        <p>
          Settings will never make restore overwrite files silently. IronVault still previews restore plans first and refuses overwrite by default.
        </p>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { defaultRepoPath } from '../lib/ironvaultBridge'
import { loadRepoPath, resetRepoPath, saveRepoPath } from '../lib/ironvaultSettings'

const testVaultPath = '/tmp/ironvault-gui-live-test/repo'
const savedPath = ref(loadRepoPath())
const repoPathDraft = ref(savedPath.value)
const saveMessage = ref('Vault path is saved locally for this desktop app.')

const pathChanged = computed(() => repoPathDraft.value.trim() !== savedPath.value)

function savePath() {
  const nextPath = saveRepoPath(repoPathDraft.value)

  savedPath.value = nextPath
  repoPathDraft.value = nextPath
  saveMessage.value = 'Vault path saved. The dashboard will use this path the next time it refreshes.'
}

function useTestVault() {
  repoPathDraft.value = testVaultPath
  savePath()
}

function restoreDefaultPath() {
  const nextPath = resetRepoPath()

  savedPath.value = nextPath
  repoPathDraft.value = nextPath
  saveMessage.value = `Vault path reset to the default: ${defaultRepoPath}`
}
</script>

<style scoped>
.settings-page {
  display: grid;
  gap: 1.25rem;
}

.settings-hero,
.settings-card {
  border: 1px solid var(--iv-border);
  border-radius: var(--iv-radius-lg);
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--iv-surface) 94%, transparent), var(--iv-surface-raised)),
    radial-gradient(circle at 10% 10%, rgba(249, 115, 22, 0.14), transparent 22rem);
  box-shadow: var(--iv-shadow-soft);
}

.settings-hero {
  padding: clamp(1.35rem, 4vw, 2rem);
}

.settings-hero h2 {
  margin: 0.2rem 0 0;
  font-size: clamp(2.1rem, 5vw, 4rem);
  line-height: 0.95;
  letter-spacing: -0.075em;
}

.settings-hero p,
.settings-card p {
  color: var(--iv-muted);
  line-height: 1.6;
}

.settings-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(280px, 0.75fr);
  gap: 1rem;
}

.settings-card {
  padding: 1.25rem;
}

.settings-card-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1rem;
}

.settings-card h3 {
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

.settings-badge {
  padding: 0.42rem 0.72rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-accent-soft);
  color: var(--iv-text);
  font-size: 0.78rem;
  font-weight: 900;
}

.setting-label {
  display: block;
  margin-bottom: 0.55rem;
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
  min-height: 44px;
  width: 100%;
  padding: 0.7rem 0.9rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-bg-soft);
  color: var(--iv-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.settings-note {
  margin-top: 0.9rem;
}

.settings-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 1rem;
}

@media (max-width: 940px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .repo-form-row {
    grid-template-columns: 1fr;
  }
}
</style>
