<template>
  <div id="app">
    <nav class="navbar">
      <div class="brand-container">
        <h1 class="brand">Virus Evolution Tracker</h1>
        <p class="tagline">
          Monitoring variant evolution and epidemiological trends
        </p>
      </div>
      <div class="nav-links">
        <router-link to="/" exact-active-class="active">Dashboard</router-link>
        <router-link to="/variants" exact-active-class="active">Variants</router-link>
        <router-link to="/analysis" exact-active-class="active">Analysis</router-link>
        <router-link to="/phylogeny" exact-active-class="active">Phylogeny</router-link>
        <router-link
          v-if="authStore.isAdmin"
          to="/admin/users"
          exact-active-class="active"
        >
          Admin
        </router-link>
        <router-link
          v-if="!authStore.isAuthenticated"
          to="/login"
          exact-active-class="active"
        >
          Sign In
        </router-link>
        <button
          v-else
          class="btn btn-ghost"
          type="button"
          @click="handleLogout"
        >
          Sign Out
        </button>
      </div>
    </nav>
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup>
import { useAuthStore } from './store/authStore';
import { useRouter } from 'vue-router';

const authStore = useAuthStore();
const router = useRouter();

const handleLogout = () => {
  authStore.logout();
  router.push({ path: '/login' });
};
</script>

<style scoped>
#app {
  min-height: 100vh;
  background: var(--color-background);
}

.navbar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: #2c3e50;
  color: #ffffff;
  padding: 1rem 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1.5rem;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

.brand-container {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.brand {
  font-size: 1.5rem;
  font-weight: 700;
}

.tagline {
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9rem;
}

.nav-links {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-wrap: wrap;
}

.nav-links a {
  color: inherit;
  padding: 0.5rem 1rem;
  border-radius: 999px;
  transition: background 0.2s ease, transform 0.2s ease;
}

.nav-links a:hover {
  background: rgba(255, 255, 255, 0.15);
  transform: translateY(-1px);
}

.nav-links a.active {
  background: #3498db;
  color: #ffffff;
  box-shadow: 0 4px 10px rgba(52, 152, 219, 0.35);
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 999px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s ease, transform 0.2s ease;
}

.btn-ghost {
  background: transparent;
  color: #ecf0f1;
}

.btn-ghost:hover {
  background: rgba(255, 255, 255, 0.15);
  transform: translateY(-1px);
}

.main-content {
  min-height: calc(100vh - 64px);
}

@media (max-width: 768px) {
  .navbar {
    flex-direction: column;
    gap: 1rem;
    align-items: flex-start;
  }
}
</style>