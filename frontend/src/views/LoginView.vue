<template>
  <section class="login-view">
    <form class="card" @submit.prevent="handleSubmit">
      <h1>Sign In</h1>
      <p class="subtitle">Use your email and password to access the dashboard.</p>

      <label class="field">
        <span>Email</span>
        <input v-model="form.email" type="email" required autocomplete="email" />
      </label>

      <label class="field">
        <span>Password</span>
        <input
          v-model="form.password"
          type="password"
          required
          autocomplete="current-password"
        />
      </label>

      <p v-if="error" class="error">{{ error }}</p>

      <button class="btn btn-primary" type="submit" :disabled="loading">
        <span v-if="loading">Signing in…</span>
        <span v-else>Sign In</span>
      </button>
    </form>
  </section>
</template>

<script setup>
import { reactive, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../store/authStore';

const form = reactive({
  email: '',
  password: '',
});

const authStore = useAuthStore();
const loading = computed(() => authStore.loading);
const error = computed(() => authStore.error);

const router = useRouter();
const route = useRoute();

const handleSubmit = async () => {
  try {
    await authStore.login(form);
    const redirect = route.query.redirect || '/';
    router.replace(redirect);
  } catch (err) {
    console.error(err);
  }
};
</script>

<style scoped>
.login-view {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background);
  padding: 2rem;
}

.card {
  width: min(400px, 100%);
  background: var(--color-surface);
  border-radius: 12px;
  box-shadow: var(--shadow-elevation);
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.subtitle {
  color: #7f8c8d;
  font-size: 0.95rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field input {
  padding: 0.75rem;
  border: 1px solid #dfe6e9;
  border-radius: 8px;
  font-size: 1rem;
}

.field input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.2);
}

.error {
  color: #c0392b;
  font-size: 0.9rem;
}

.btn {
  padding: 0.75rem;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
}

.btn-primary {
  background: var(--color-primary);
  color: #fff;
}

.btn-primary[disabled] {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>

