<template>
  <section class="admin-users">
    <header class="header">
      <div>
        <h1>User Management</h1>
        <p class="subtitle">
          Create new accounts and review existing users and their roles.
        </p>
      </div>
      <button class="btn btn-secondary" @click="reload" :disabled="loading">
        Refresh
      </button>
    </header>

    <div class="layout">
      <form class="card" @submit.prevent="handleCreate">
        <h2>Create User</h2>
        <label class="field">
          <span>Email</span>
          <input v-model="form.email" type="email" required />
        </label>
        <label class="field">
          <span>Password</span>
          <input v-model="form.password" type="password" required />
        </label>
        <label class="field">
          <span>Roles</span>
          <select v-model="selectedRoles" multiple>
            <option value="admin">admin</option>
            <option value="analyst">analyst</option>
            <option value="uploader">uploader</option>
            <option value="viewer">viewer</option>
          </select>
          <small>Select one or more roles (default: viewer).</small>
        </label>
        <p v-if="error" class="error">{{ error }}</p>
        <button class="btn btn-primary" type="submit" :disabled="loading">
          <span v-if="loading">Creating…</span>
          <span v-else>Create User</span>
        </button>
      </form>

      <section class="card">
        <h2>Existing Users</h2>
        <div v-if="loading" class="placeholder">Loading users…</div>
        <div v-else-if="users.length === 0" class="placeholder">
          No users found.
        </div>
        <table v-else class="users-table">
          <thead>
            <tr>
              <th>Email</th>
              <th>Roles</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="user in users" :key="user.id">
              <td>{{ user.email }}</td>
              <td>
                <span v-for="role in user.roles" :key="role" class="badge">
                  {{ role }}
                </span>
              </td>
              <td>{{ formatDate(user.created_at) }}</td>
            </tr>
          </tbody>
        </table>
      </section>
    </div>
  </section>
</template>

<script setup>
import { onMounted, reactive, ref, computed } from 'vue';
import { useAuthStore } from '../store/authStore';

const authStore = useAuthStore();
const loading = computed(() => authStore.loading);
const error = computed(() => authStore.error);
const users = computed(() => authStore.users);

const form = reactive({
  email: '',
  password: '',
});
const selectedRoles = ref(['viewer']);

const reload = async () => {
  try {
    await authStore.fetchUsers();
  } catch (err) {
    console.error(err);
  }
};

const handleCreate = async () => {
  try {
    await authStore.createUser({
      email: form.email,
      password: form.password,
      roles: selectedRoles.value,
    });
    form.email = '';
    form.password = '';
    selectedRoles.value = ['viewer'];
  } catch (err) {
    console.error(err);
  }
};

const formatDate = (value) => {
  try {
    return new Date(value).toLocaleString();
  } catch (_err) {
    return value;
  }
};

onMounted(() => {
  reload();
});
</script>

<style scoped>
.admin-users {
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
}

.subtitle {
  margin-top: 0.35rem;
  color: #7f8c8d;
}

.layout {
  display: grid;
  grid-template-columns: minmax(260px, 320px) 1fr;
  gap: 1.5rem;
}

.card {
  background: var(--color-surface);
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: var(--shadow-elevation);
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field input,
.field select {
  padding: 0.75rem;
  border: 1px solid #dfe6e9;
  border-radius: 8px;
  font-size: 1rem;
}

.field select {
  min-height: 120px;
}

.field small {
  color: #95a5a6;
}

.error {
  color: #c0392b;
  font-size: 0.9rem;
}

.users-table {
  width: 100%;
  border-collapse: collapse;
}

.users-table th,
.users-table td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #ecf0f1;
  text-align: left;
}

.badge {
  display: inline-block;
  background: rgba(52, 152, 219, 0.15);
  color: var(--color-primary);
  padding: 0.25rem 0.5rem;
  border-radius: 999px;
  font-size: 0.85rem;
  margin-right: 0.35rem;
}

.placeholder {
  color: #7f8c8d;
  font-style: italic;
}

.btn {
  padding: 0.65rem 1.25rem;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-weight: 600;
}

.btn-secondary {
  background: #ecf0f1;
  color: var(--color-text);
}

.btn-secondary:hover:not(:disabled) {
  box-shadow: var(--shadow-elevation);
}

.btn-primary {
  background: var(--color-primary);
  color: #fff;
}

.btn-primary[disabled] {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 960px) {
  .layout {
    grid-template-columns: 1fr;
  }
}
</style>

