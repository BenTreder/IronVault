<template>
  <div class="app-shell">
    <aside class="sidebar">
      <RouterLink to="/dashboard" class="brand" aria-label="IronVault Dashboard">
        <span class="brand-mark">IV</span>
        <span>
          <strong>IronVault</strong>
          <small>Backup vault</small>
        </span>
      </RouterLink>

      <nav class="nav-links" aria-label="Primary navigation">
        <RouterLink to="/dashboard">Dashboard</RouterLink>
        <RouterLink to="/backup">Backup</RouterLink>
        <RouterLink to="/restore">Restore</RouterLink>
        <RouterLink to="/snapshots">Snapshots</RouterLink>
        <RouterLink to="/settings">Settings</RouterLink>
      </nav>

      <div class="sidebar-footer">
        <button class="theme-toggle" type="button" @click="toggleTheme">
          {{ themeLabel }}
        </button>

        <a
          class="creator-link"
          href="https://bentreder.com"
          target="_blank"
          rel="noreferrer"
        >
          Made by Ben Treder
        </a>
      </div>
    </aside>

    <main class="main-panel">
      <header class="topbar">
        <div>
          <p class="eyebrow">Premium local backup protection</p>
          <h1>IronVault</h1>
        </div>

        <div class="topbar-status">
          <span class="iv-dot" aria-hidden="true"></span>
          Vault door closed
        </div>
      </header>

      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

const theme = ref<'dark' | 'light'>('dark')

const themeLabel = computed(() =>
  theme.value === 'dark' ? 'Light mode' : 'Dark mode'
)

function applyTheme() {
  document.documentElement.dataset.theme = theme.value
  localStorage.setItem('ironvault-theme', theme.value)
}

function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  applyTheme()
}

onMounted(() => {
  const saved = localStorage.getItem('ironvault-theme')
  if (saved === 'light' || saved === 'dark') {
    theme.value = saved
  }

  applyTheme()
})
</script>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
}

.sidebar {
  position: sticky;
  top: 0;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 2rem;
  padding: 1.35rem;
  border-right: 1px solid var(--iv-border);
  background: color-mix(in srgb, var(--iv-surface) 88%, transparent);
  backdrop-filter: blur(18px);
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  padding: 0.65rem;
  border-radius: var(--iv-radius-md);
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 46px;
  height: 46px;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--iv-accent), var(--iv-accent-strong));
  color: white;
  font-weight: 950;
  box-shadow: 0 16px 34px rgba(249, 115, 22, 0.26);
}

.brand strong {
  display: block;
  letter-spacing: -0.03em;
}

.brand small {
  display: block;
  color: var(--iv-muted);
  margin-top: 0.15rem;
}

.nav-links {
  display: grid;
  gap: 0.45rem;
}

.nav-links a {
  padding: 0.82rem 0.9rem;
  border-radius: 14px;
  color: var(--iv-muted-strong);
  font-weight: 750;
  transition: background 160ms ease, color 160ms ease, transform 160ms ease;
}

.nav-links a:hover {
  transform: translateX(2px);
  background: var(--iv-surface-raised);
  color: var(--iv-text);
}

.nav-links a.router-link-active {
  background: var(--iv-accent-soft);
  color: var(--iv-text);
  box-shadow: inset 3px 0 0 var(--iv-accent);
}

.sidebar-footer {
  margin-top: auto;
  display: grid;
  gap: 0.75rem;
}

.theme-toggle {
  width: 100%;
  min-height: 42px;
  border-radius: 999px;
  background: var(--iv-surface-raised);
  color: var(--iv-text);
  border: 1px solid var(--iv-border);
  cursor: pointer;
  font-weight: 800;
}

.creator-link {
  color: var(--iv-muted);
  font-size: 0.88rem;
  text-align: center;
}

.creator-link:hover {
  color: var(--iv-accent);
}

.main-panel {
  min-width: 0;
  padding: 1.35rem clamp(1rem, 3vw, 2.5rem) 3rem;
}

.topbar {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: center;
  margin-bottom: 1.5rem;
}

.eyebrow {
  margin: 0 0 0.2rem;
  color: var(--iv-accent);
  font-size: 0.78rem;
  font-weight: 900;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.topbar h1 {
  margin: 0;
  font-size: clamp(1.6rem, 3vw, 2.4rem);
  letter-spacing: -0.05em;
}

.topbar-status {
  display: inline-flex;
  align-items: center;
  gap: 0.65rem;
  padding: 0.72rem 0.95rem;
  border: 1px solid var(--iv-border);
  border-radius: 999px;
  background: var(--iv-surface);
  color: var(--iv-muted-strong);
  font-weight: 800;
  box-shadow: var(--iv-shadow-soft);
}

@media (max-width: 860px) {
  .app-shell {
    grid-template-columns: 1fr;
  }

  .sidebar {
    position: static;
    min-height: auto;
    border-right: 0;
    border-bottom: 1px solid var(--iv-border);
  }

  .nav-links {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }

  .nav-links a {
    text-align: center;
    padding: 0.75rem 0.4rem;
  }

  .topbar {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
