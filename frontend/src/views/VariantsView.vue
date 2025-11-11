<template>
  <section class="view">
    <header class="view-header">
      <div>
        <h1>Variant Catalog</h1>
        <p class="subtitle">
          Browse all uploaded variants with metadata and mutation counts.
        </p>
      </div>
      <button class="btn btn-primary" @click="refresh" :disabled="loading">
        Refresh
      </button>
    </header>

    <div v-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <div class="table-wrapper" v-if="variants.length">
      <table class="variants-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Lineage</th>
            <th>Location</th>
            <th>Collected</th>
            <th>Mutations</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="variant in variants" :key="variant.id">
            <td>
              <strong>{{ variant.name }}</strong>
            </td>
            <td>{{ variant.lineage }}</td>
            <td>{{ variant.location }}</td>
            <td>{{ formatDate(variant.date) }}</td>
            <td>{{ variant.mutations.length }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <p v-else class="empty-state">No variants uploaded yet.</p>
  </section>
</template>

<script setup>
import { onMounted } from 'vue';
import { storeToRefs } from 'pinia';

import { useVariantStore } from '../store/variantStore';

const store = useVariantStore();
const { variants, loading, error } = storeToRefs(store);

const refresh = async () => {
  try {
    await store.fetchVariants();
  } catch (refreshError) {
    console.error(refreshError);
  }
};

const formatDate = (value) => {
  try {
    return new Date(value).toLocaleString();
  } catch (e) {
    return value;
  }
};

onMounted(async () => {
  if (!variants.value.length) {
    await refresh();
  }
});
</script>

<style scoped>
.view {
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
}

.subtitle {
  margin-top: 0.25rem;
  color: #7f8c8d;
}

.table-wrapper {
  background: var(--color-surface);
  border-radius: 12px;
  box-shadow: var(--shadow-elevation);
  overflow: hidden;
}

.variants-table {
  width: 100%;
  border-collapse: collapse;
}

.variants-table th,
.variants-table td {
  padding: 0.75rem 1rem;
  text-align: left;
  border-bottom: 1px solid #ecf0f1;
}

.variants-table th {
  background: #f8f9fa;
  font-weight: 600;
}

.variants-table tr:hover {
  background: rgba(52, 152, 219, 0.08);
}

.empty-state {
  text-align: center;
  color: #7f8c8d;
  font-style: italic;
}

.btn {
  padding: 0.65rem 1.25rem;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  font-weight: 600;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-primary {
  background: var(--color-primary);
  color: #ffffff;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-dark);
  box-shadow: 0 8px 16px rgba(52, 152, 219, 0.25);
  transform: translateY(-1px);
}

.alert {
  padding: 1rem;
  border-radius: 8px;
  border: 1px solid transparent;
}

.alert-error {
  background: rgba(231, 76, 60, 0.1);
  border-color: rgba(231, 76, 60, 0.4);
  color: #c0392b;
}
</style>


