<template>
  <div id="app">
    <header>
      <h1>IronVault</h1>
      <p class="subtitle">Premium Backup System</p>
    </header>

    <nav>
      <router-link to="/dashboard">Dashboard</router-link>
      <router-link to="/backup">Backup</router-link>
      <router-link to="/snapshots">Snapshots</router-link>
      <router-link to="/restore">Restore</router-link>
      <router-link to="/settings">Settings</router-link>
      <router-link to="/logs">Logs</router-link>
    </nav>

    <main>
      <router-view />
    </main>

    <footer>
      <p>IronVault v0.1.0 - A deduplicating backup system for Arch Linux</p>
    </footer>
  </div>
</template>

<script setup lang="ts">
// App root component
</script>

<style scoped>
#app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

header {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  color: white;
  padding: 1rem 2rem;
}

header h1 {
  margin: 0;
  font-size: 1.5rem;
}

.subtitle {
  margin: 0;
  opacity: 0.8;
  font-size: 0.9rem;
}

nav {
  background: #0f3460;
  padding: 0.5rem 2rem;
  display: flex;
  gap: 1rem;
}

nav a {
  color: #e94560;
  text-decoration: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  transition: background 0.2s;
}

nav a:hover,
nav a.router-link-active {
  background: #e94560;
  color: white;
}

main {
  flex: 1;
  padding: 2rem;
  background: #1a1a2e;
  color: white;
}

footer {
  background: #0f3460;
  padding: 1rem 2rem;
  text-align: center;
  font-size: 0.8rem;
  opacity: 0.8;
}
</style>
*/
-->