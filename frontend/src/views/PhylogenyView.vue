<template>
  <div class="phylogeny-view">
    <header class="page-header">
      <h2>Phylogeny Explorer</h2>
      <p class="subtitle">
        Inspect reconstructed phylogenetic relationships and alignment statistics from the latest analysis pipeline.
      </p>
    </header>

    <section class="controls">
      <div class="field">
        <label for="lineage">Lineage</label>
        <input
          id="lineage"
          v-model.trim="filters.lineage"
          type="text"
          placeholder="e.g. B.1.1.7"
        />
      </div>
      <div class="field">
        <label for="location">Location</label>
        <input
          id="location"
          v-model.trim="filters.location"
          type="text"
          placeholder="e.g. Europe"
        />
      </div>
      <div class="control-actions">
        <button class="btn btn-primary" type="button" @click="loadPhylogeny" :disabled="phylogenyLoading">
          Refresh
        </button>
        <button class="btn btn-secondary" type="button" @click="resetFilters" :disabled="phylogenyLoading">
          Reset
        </button>
      </div>
    </section>

    <section v-if="phylogenyError" class="alert alert-error">
      {{ phylogenyErrorMessage }}
    </section>

    <section v-if="phylogenyLoading" class="loading">
      Loading phylogenetic tree...
    </section>

    <section v-else-if="phylogeny" class="results">
      <div class="summary-cards">
        <div class="card">
          <h3>Sample Size</h3>
          <p class="value">{{ (phylogeny.count ?? 0).toLocaleString() }}</p>
        </div>
        <div v-if="phylogeny.alignment" class="card">
          <h3>Alignment Length</h3>
          <p class="value">{{ phylogeny.alignment.length.toLocaleString() }} bp</p>
        </div>
        <div v-if="phylogeny.alignment" class="card">
          <h3>Consensus</h3>
          <p class="value monospace">{{ phylogeny.alignment.consensus.slice(0, 32) }}<span v-if="phylogeny.alignment.consensus.length > 32">...</span></p>
        </div>
      </div>

      <div class="tree-container">
        <h3>Tree Structure</h3>
        <pre class="tree-output">
{{ treeText }}
        </pre>
      </div>
    </section>

    <section v-else class="empty-state">
      <p>No phylogenetic reconstruction available yet. Try adjusting filters or run the bioinformatics pipeline.</p>
    </section>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive } from 'vue';
import { storeToRefs } from 'pinia';

import { useAnalyticsStore } from '../store/analyticsStore';

const analyticsStore = useAnalyticsStore();
const { phylogeny, phylogenyLoading, phylogenyError } = storeToRefs(analyticsStore);

const filters = reactive({
  lineage: '',
  location: '',
});

const phylogenyErrorMessage = computed(() => {
  if (!phylogenyError.value) {
    return '';
  }
  return (
    phylogenyError.value.response?.data?.message ??
    phylogenyError.value.message ??
    'Failed to load phylogeny results'
  );
});

const activeFilters = computed(() => {
  const params = {};
  if (filters.lineage) {
    params.lineage = filters.lineage;
  }
  if (filters.location) {
    params.location = filters.location;
  }
  return params;
});

const formatTree = (node, depth = 0) => {
  if (!node) {
    return '';
  }
  const indent = '  '.repeat(depth);
  const name = node.name ?? 'Unnamed';
  const branch = node.branch_length != null ? ` [${node.branch_length.toFixed(3)}]` : '';
  let output = `${indent}- ${name}${branch}`;
  if (node.children && node.children.length) {
    node.children.forEach((child) => {
      output += `\n${formatTree(child, depth + 1)}`;
    });
  }
  return output;
};

const treeText = computed(() => {
  if (!phylogeny.value?.tree) {
    return 'No tree structure available.';
  }
  return formatTree(phylogeny.value.tree, 0);
});

const loadPhylogeny = async () => {
  await analyticsStore.fetchPhylogeny(activeFilters.value);
};

const resetFilters = async () => {
  filters.lineage = '';
  filters.location = '';
  await loadPhylogeny();
};

onMounted(async () => {
  await loadPhylogeny();
});
</script>

<style scoped>
.phylogeny-view {
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.page-header h2 {
  margin: 0;
  font-size: 1.75rem;
}

.subtitle {
  margin-top: 0.5rem;
  color: #7f8c8d;
}

.controls {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1rem;
  align-items: end;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field input {
  padding: 0.65rem;
  border-radius: 8px;
  border: 1px solid #dfe6e9;
  font-size: 1rem;
}

.control-actions {
  display: flex;
  gap: 0.75rem;
}

.btn {
  padding: 0.65rem 1.25rem;
  border-radius: 8px;
  border: none;
  font-weight: 600;
  cursor: pointer;
}

.btn-primary {
  background: var(--color-primary);
  color: #ffffff;
}

.btn-secondary {
  background: #ecf0f1;
  color: var(--color-text);
}

.alert {
  padding: 1rem;
  border-radius: 8px;
}

.alert-error {
  background: rgba(231, 76, 60, 0.1);
  border: 1px solid rgba(231, 76, 60, 0.35);
  color: #c0392b;
}

.loading {
  padding: 1rem;
  font-style: italic;
  color: #7f8c8d;
}

.results {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.summary-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1rem;
}

.card {
  background: var(--color-surface);
  padding: 1.25rem;
  border-radius: 12px;
  box-shadow: var(--shadow-elevation);
}

.card .value {
  margin-top: 0.5rem;
  font-size: 1.5rem;
  font-weight: 700;
}

.card .value.monospace {
  font-family: 'Fira Mono', 'Courier New', monospace;
  font-size: 1.1rem;
  word-break: break-all;
}

.tree-container {
  background: var(--color-surface);
  padding: 1.5rem;
  border-radius: 12px;
  box-shadow: var(--shadow-elevation);
}

.tree-output {
  max-height: 420px;
  overflow: auto;
  background: rgba(236, 240, 241, 0.4);
  padding: 1rem;
  border-radius: 8px;
  font-family: 'Fira Mono', 'Courier New', monospace;
  white-space: pre;
}

.empty-state {
  padding: 1.5rem;
  text-align: center;
  color: #7f8c8d;
  background: rgba(236, 240, 241, 0.4);
  border-radius: 12px;
}
</style>
