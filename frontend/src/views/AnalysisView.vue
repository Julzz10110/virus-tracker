<template>
  <section class="view">
    <header class="view-header">
      <div>
        <h1>Mutation Analysis</h1>
        <p class="subtitle">
          Inspect mutation frequency and entropy across uploaded sequences.
        </p>
      </div>
      <button class="btn btn-secondary" @click="reset" :disabled="loading">
        Reset Filters
      </button>
    </header>

    <div class="card filters">
      <form class="filters-form" @submit.prevent="apply">
        <label class="field">
          <span>Lineage</span>
          <input v-model.trim="filters.lineage" placeholder="e.g. XBB.1.5" />
        </label>
        <label class="field">
          <span>Location</span>
          <input v-model.trim="filters.location" placeholder="e.g. India" />
        </label>
        <div class="actions">
          <button class="btn btn-primary" type="submit" :disabled="loading">
            Run Analysis
          </button>
        </div>
      </form>
    </div>

    <div v-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <div class="card results">
      <div class="results-header">
        <h2>Frequency Summary</h2>
        <span class="badge">{{ mutations.length }} positions</span>
      </div>
      <div class="table-wrapper" v-if="mutations.length">
        <table class="results-table">
          <thead>
            <tr>
              <th>Gene</th>
              <th>Position</th>
              <th>Reference</th>
              <th>Mutation</th>
              <th>Frequency</th>
              <th>Entropy</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in mutations" :key="`${item.gene}-${item.position}`">
              <td>{{ item.gene }}</td>
              <td>{{ item.position }}</td>
              <td>{{ item.reference }}</td>
              <td>{{ item.mutation }}</td>
              <td>{{ (item.frequency * 100).toFixed(2) }}%</td>
              <td>{{ item.entropy.toFixed(3) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-else class="empty-state">
        No mutation data for the selected filters. Upload variants or broaden
        your query.
      </p>
    </div>
  </section>
</template>

<script setup>
import { computed, reactive } from 'vue';
import { storeToRefs } from 'pinia';

import { useVariantStore } from '../store/variantStore';

const store = useVariantStore();
const { mutationFrequency, loading, error } = storeToRefs(store);

const filters = reactive({
  lineage: '',
  location: '',
});

const mutations = computed(() =>
  [...mutationFrequency.value].sort((a, b) => b.frequency - a.frequency)
);

const apply = async () => {
  try {
    await store.fetchMutationFrequency(filters);
  } catch (applyError) {
    console.error(applyError);
  }
};

const reset = async () => {
  filters.lineage = '';
  filters.location = '';
  await apply();
};

if (!mutationFrequency.value.length) {
  apply();
}
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

.card {
  background: var(--color-surface);
  border-radius: 12px;
  box-shadow: var(--shadow-elevation);
  padding: 1.5rem;
}

.filters-form {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  align-items: end;
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

.actions {
  display: flex;
  gap: 1rem;
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

.btn-secondary {
  background: #ecf0f1;
  color: var(--color-text);
}

.btn-secondary:hover:not(:disabled) {
  box-shadow: var(--shadow-elevation);
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

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.badge {
  background: rgba(52, 152, 219, 0.15);
  color: var(--color-primary);
  padding: 0.25rem 0.75rem;
  border-radius: 999px;
  font-size: 0.875rem;
}

.table-wrapper {
  overflow-x: auto;
}

.results-table {
  width: 100%;
  border-collapse: collapse;
}

.results-table th,
.results-table td {
  padding: 0.75rem 1rem;
  text-align: left;
  border-bottom: 1px solid #ecf0f1;
}

.results-table th {
  background: #f8f9fa;
  font-weight: 600;
}

.results-table tr:hover {
  background: rgba(52, 152, 219, 0.08);
}

.empty-state {
  text-align: center;
  color: #7f8c8d;
  font-style: italic;
}
</style>


