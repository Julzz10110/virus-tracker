<template>
  <div class="dashboard">
    <div v-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <section class="controls">
      <div class="filters">
        <h3>Filters</h3>
        <div class="filters-grid">
          <label class="field">
            <span>Lineage</span>
            <input
              v-model.trim="filters.lineage"
              placeholder="e.g. B.1.1.7"
              type="text"
            />
          </label>
          <label class="field">
            <span>Location</span>
            <input
              v-model.trim="filters.location"
              placeholder="e.g. United Kingdom"
              type="text"
            />
          </label>
        </div>
        <div class="filter-actions">
          <button class="btn btn-primary" @click="applyFilters" :disabled="loading">
            Apply
          </button>
          <button class="btn btn-secondary" @click="resetFilters" :disabled="loading">
            Reset
          </button>
        </div>
      </div>

      <div v-if="isAdmin" class="reference-card">
        <h3>Reference Genome</h3>
        <textarea
          v-model="referenceDraft"
          placeholder="Paste reference sequence (FASTA without header)"
          rows="4"
        />
        <div class="reference-actions">
          <button class="btn btn-secondary" @click="restoreReference">
            Reset
          </button>
          <button
            class="btn btn-primary"
            @click="saveReference"
            :disabled="loading || !referenceDraft"
          >
            Save Reference
          </button>
        </div>
      </div>
      <div v-else class="reference-card locked">
        <h3>Reference Genome</h3>
        <p class="hint">
          Only administrators can update the reference genome. Contact an admin if you need changes.
        </p>
      </div>
    </section>

    <section class="stats-grid">
      <div class="stat-card">
        <h3>Total Variants</h3>
        <p class="stat-number">{{ totalVariants }}</p>
      </div>
      <div class="stat-card">
        <h3>Unique Locations</h3>
        <p class="stat-number">{{ uniqueCountries }}</p>
      </div>
      <div class="stat-card">
        <h3>Total Mutations</h3>
        <p class="stat-number">{{ stats?.total_mutations ?? 0 }}</p>
      </div>
      <div class="stat-card">
        <h3>Avg. Mutations / Variant</h3>
        <p class="stat-number">
          {{ (stats?.average_mutations_per_variant ?? 0).toFixed(2) }}
        </p>
      </div>
      <div class="stat-card">
        <h3>Tracked Samples</h3>
        <p class="stat-number">{{ latestTrackedCountDisplay }}</p>
      </div>
      <div class="stat-card" :class="{ 'alert-card': growthAlertActive }">
        <h3>Latest R_e</h3>
        <p class="stat-number">{{ latestReDisplay }}</p>
        <p v-if="growthAlertActive" class="stat-subtext">Above epidemic threshold</p>
      </div>
      <div class="stat-card">
        <h3>Growth Rate</h3>
        <p class="stat-number">{{ latestGrowthRateDisplay }}</p>
      </div>
      <div class="stat-card">
        <h3>Doubling Time</h3>
        <p class="stat-number">{{ latestDoublingDisplay }}</p>
      </div>

    </section>

    <section class="charts">
      <div class="chart-container">
        <div class="chart-header">
          <h3>Mutation Frequency</h3>
          <span v-if="loading" class="badge">Loading...</span>
        </div>
        <div ref="mutationChartRef" class="chart"></div>
        <p v-if="!mutationFrequency.length" class="chart-placeholder">
          No mutation data available. Upload variants or adjust filters.
        </p>
      </div>
      <div class="chart-container">
        <div class="chart-header">
          <h3>Epidemiological Trends</h3>
          <span v-if="growthLoading" class="badge">Loading...</span>
        </div>
        <div v-if="growthError" class="chart-placeholder error">
          {{ growthErrorMessage }}
        </div>
        <div v-else>
          <div ref="growthChartRef" class="chart"></div>
          <p v-if="!growthHistory.length" class="chart-placeholder">
            Growth metrics will appear once background collection runs.
          </p>
        </div>
      </div>
    </section>

    <section v-if="canUpload" class="upload-section">
      <h3>Upload New Variant</h3>
      <form @submit.prevent="handleUpload" class="upload-form">
        <div class="form-grid">
          <label class="field">
            <span>Variant Name</span>
            <input v-model.trim="newVariant.name" required />
          </label>
          <label class="field">
            <span>Lineage</span>
            <input v-model.trim="newVariant.lineage" required />
          </label>
          <label class="field">
            <span>Location</span>
            <input v-model.trim="newVariant.location" required />
          </label>
        </div>
        <label class="field">
          <span>Sequence</span>
          <textarea
            v-model.trim="newVariant.sequence"
            placeholder="Enter raw sequence"
            required
            rows="5"
          ></textarea>
        </label>
        <div class="upload-actions">
          <button class="btn btn-primary" type="submit" :disabled="loading">
            Upload Variant
          </button>
        </div>
      </form>
    </section>

    <section v-else class="upload-section locked">
      <h3>Upload New Variant</h3>
      <p class="hint">
        Upload privileges are limited to administrators and uploaders. Please sign in with an
        authorized account.
      </p>
    </section>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import Plotly from 'plotly.js-dist-min';

import { useVariantStore } from '../store/variantStore';
import { useAuthStore } from '../store/authStore';
import { useAnalyticsStore } from '../store/analyticsStore';

const store = useVariantStore();
const authStore = useAuthStore();
const analyticsStore = useAnalyticsStore();
const { variants, stats, reference, mutationFrequency, loading, error } =
  storeToRefs(store);
const { growthHistory, growthLoading, growthError, liveGrowth } =
  storeToRefs(analyticsStore);

const mutationChartRef = ref(null);
const growthChartRef = ref(null);
const referenceDraft = ref('');
const filters = reactive({
  lineage: '',
  location: '',
});

const newVariant = reactive({
  name: '',
  lineage: '',
  location: '',
  sequence: '',
});

const totalVariants = computed(() => stats.value?.total_variants ?? 0);
const uniqueCountries = computed(() => {
  return new Set(variants.value.map((variant) => variant.location)).size;
});
const canUpload = computed(() => authStore.canUpload);
const isAdmin = computed(() => authStore.isAdmin);

const latestGrowthHistoryEntry = computed(() => {
  if (!growthHistory.value.length) {
    return null;
  }
  const sorted = [...growthHistory.value].sort((a, b) => {
    return new Date(b.collected_at).getTime() - new Date(a.collected_at).getTime();
  });
  return sorted[0];
});

const latestGrowthSnapshot = computed(() => {
  return liveGrowth.value ?? latestGrowthHistoryEntry.value ?? null;
});

const latestReValue = computed(() => latestGrowthSnapshot.value?.reproduction_number ?? null);
const latestGrowthRateValue = computed(() => latestGrowthSnapshot.value?.growth_rate ?? null);
const latestDoublingTimeValue = computed(() => latestGrowthSnapshot.value?.doubling_time ?? null);
const latestTrackedCount = computed(() => {
  const snapshot = latestGrowthSnapshot.value;
  if (!snapshot) {
    return null;
  }
  return snapshot.total_count ?? snapshot.total_variants ?? null;
});

const latestReDisplay = computed(() => {
  if (latestReValue.value == null) {
    return 'N/A';
  }
  return latestReValue.value.toFixed(2);
});

const latestGrowthRateDisplay = computed(() => {
  if (latestGrowthRateValue.value == null) {
    return 'N/A';
  }
  return latestGrowthRateValue.value.toFixed(2);
});

const latestDoublingDisplay = computed(() => {
  if (latestDoublingTimeValue.value == null) {
    return 'N/A';
  }
  return `${latestDoublingTimeValue.value.toFixed(1)} days`;
});

const latestTrackedCountDisplay = computed(() => {
  if (latestTrackedCount.value == null) {
    return 'N/A';
  }
  return latestTrackedCount.value.toLocaleString();
});

const growthAlertActive = computed(() => {
  if (latestReValue.value == null) {
    return false;
  }
  return latestReValue.value > 1;
});

const growthErrorMessage = computed(() => {
  if (!growthError.value) {
    return '';
  }
  return (
    growthError.value.response?.data?.message ??
    growthError.value.message ??
    'Failed to load growth metrics'
  );
});

const initialize = async () => {
  try {
    await Promise.all([
      store.fetchVariants(),
      store.fetchStats(),
      store.fetchReference(),
      store.fetchMutationFrequency(),
      analyticsStore.fetchGrowthHistory(),
      analyticsStore.fetchLiveGrowth(),
    ]);
    referenceDraft.value = reference.value;
  } catch (fetchError) {
    console.error(fetchError);
  } finally {
    renderMutationChart();
    renderGrowthChart();
  }
};

const applyFilters = async () => {
  try {
    await Promise.all([
      store.fetchVariants(filters),
      store.fetchMutationFrequency(filters),
    ]);
    await Promise.all([
      analyticsStore.fetchGrowthHistory(),
      analyticsStore.fetchLiveGrowth(),
    ]);
  } catch (fetchError) {
    console.error(fetchError);
  }
};

const resetFilters = async () => {
  filters.lineage = '';
  filters.location = '';
  await applyFilters();
};

const restoreReference = () => {
  referenceDraft.value = reference.value;
};

const saveReference = async () => {
  try {
    await store.updateReference(referenceDraft.value);
    await store.fetchMutationFrequency(filters);
  } catch (saveError) {
    console.error(saveError);
  }
};

const handleUpload = async () => {
  try {
    await store.uploadVariant({ ...newVariant });
    await Promise.all([
      store.fetchVariants(filters),
      store.fetchStats(),
      store.fetchMutationFrequency(filters),
    ]);
    await Promise.all([
      analyticsStore.fetchGrowthHistory(),
      analyticsStore.fetchLiveGrowth(),
    ]);
    Object.assign(newVariant, {
      name: '',
      lineage: '',
      location: '',
      sequence: '',
    });
  } catch (uploadError) {
    console.error(uploadError);
  }
};

const renderMutationChart = () => {
  if (!mutationChartRef.value) {
    return;
  }
  const xValues = mutationFrequency.value.map(
    (item) => `${item.gene}-${item.position}`
  );
  const yValues = mutationFrequency.value.map((item) => item.frequency);

  const data = [
    {
      x: xValues,
      y: yValues,
      type: 'bar',
      marker: {
        color: '#3498db',
      },
    },
  ];

  const layout = {
    title: 'Mutation Frequency',
    margin: { t: 40, r: 20, l: 40, b: 120 },
    xaxis: { automargin: true },
    yaxis: { title: 'Frequency', rangemode: 'tozero' },
    paper_bgcolor: 'transparent',
    plot_bgcolor: 'transparent',
  };

  Plotly.react(mutationChartRef.value, data, layout, {
    responsive: true,
    displaylogo: false,
  });
};

const renderGrowthChart = () => {
  if (!growthChartRef.value) {
    return;
  }

  if (!growthHistory.value.length) {
    Plotly.react(
      growthChartRef.value,
      [],
      {
        title: 'Epidemiological Trends',
        margin: { t: 40, r: 20, l: 50, b: 80 },
        xaxis: { title: 'Collected at', type: 'date' },
        yaxis: { title: 'Value' },
        paper_bgcolor: 'transparent',
        plot_bgcolor: 'transparent',
      },
      {
        responsive: true,
        displaylogo: false,
      }
    );
    return;
  }

  const sorted = [...growthHistory.value].sort((a, b) => {
    return new Date(a.collected_at).getTime() - new Date(b.collected_at).getTime();
  });
  const timestamps = sorted.map((item) => item.collected_at);
  const reproductionSeries = sorted.map((item) => item.reproduction_number ?? null);
  const growthRateSeries = sorted.map((item) => item.growth_rate ?? null);
  const doublingSeries = sorted.map((item) => item.doubling_time ?? null);

  const traces = [];
  if (reproductionSeries.some((value) => value !== null)) {
    traces.push({
      x: timestamps,
      y: reproductionSeries,
      name: 'R_e',
      type: 'scatter',
      mode: 'lines+markers',
      line: { color: '#e74c3c', width: 2 },
      connectgaps: false,
    });
  }

  if (growthRateSeries.some((value) => value !== null)) {
    traces.push({
      x: timestamps,
      y: growthRateSeries,
      name: 'Growth Rate',
      type: 'scatter',
      mode: 'lines+markers',
      line: { color: '#27ae60', width: 2 },
      connectgaps: false,
    });
  }

  if (doublingSeries.some((value) => value !== null)) {
    traces.push({
      x: timestamps,
      y: doublingSeries,
      name: 'Doubling Time (days)',
      type: 'scatter',
      mode: 'lines+markers',
      line: { color: '#8e44ad', width: 2 },
      connectgaps: false,
    });
  }

  const layout = {
    title: 'Epidemiological Trends',
    margin: { t: 40, r: 20, l: 50, b: 80 },
    xaxis: {
      title: 'Collected at',
      type: 'date',
      automargin: true,
    },
    yaxis: {
      title: 'Value',
      rangemode: 'tozero',
    },
    legend: {
      orientation: 'h',
      y: -0.2,
    },
    paper_bgcolor: 'transparent',
    plot_bgcolor: 'transparent',
  };

  Plotly.react(growthChartRef.value, traces, layout, {
    responsive: true,
    displaylogo: false,
  });
};

watch(mutationFrequency, () => {
  renderMutationChart();
});

watch(
  growthHistory,
  () => {
    if (!growthLoading.value) {
      renderGrowthChart();
    }
  },
  { deep: true }
);

watch(growthLoading, (isLoading) => {
  if (!isLoading) {
    renderGrowthChart();
  }
});

watch(reference, (value) => {
  if (!value) {
    return;
  }
  referenceDraft.value = value;
});

watch(
  () => authStore.roles.slice(),
  async () => {
    await initialize();
  }
);

onMounted(async () => {
  await initialize();
  renderMutationChart();
  renderGrowthChart();
});
</script>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 2rem;
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

.controls {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.5rem;
}

.filters,
.reference-card,
.upload-section,
.chart-container,
.stat-card {
  background: var(--color-surface);
  padding: 1.5rem;
  border-radius: 12px;
  box-shadow: var(--shadow-elevation);
}

.filters-grid {
  display: grid;
  gap: 1rem;
  margin-top: 1rem;
}

.filter-actions {
  display: flex;
  gap: 0.75rem;
  margin-top: 1rem;
}

.reference-card textarea {
  width: 100%;
  margin-top: 0.75rem;
  resize: vertical;
  min-height: 120px;
  padding: 0.75rem;
  border: 1px solid #dfe6e9;
  border-radius: 8px;
}

.reference-actions {
  display: flex;
  justify-content: space-between;
  margin-top: 1rem;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1rem;
}

.stat-card h3 {
  font-size: 1rem;
  margin-bottom: 0.5rem;
  color: #7f8c8d;
}

.stat-number {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-text);
}

.stat-subtext {
  font-size: 0.75rem;
  color: #e74c3c;
  margin-top: 0.25rem;
}

.alert-card {
  background: rgba(231, 76, 60, 0.1);
  border: 1px solid rgba(231, 76, 60, 0.4);
  color: #c0392b;
}

.charts {
  display: grid;
  gap: 1.5rem;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
}

.chart-header {
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

.chart {
  min-height: 320px;
}

.chart-placeholder {
  text-align: center;
  margin-top: 1rem;
  color: #7f8c8d;
  font-style: italic;
}

.error {
  color: #c0392b;
  font-weight: 600;
}

.upload-section {
  margin-top: 1rem;
}

.upload-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-grid {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field input,
.field textarea {
  padding: 0.75rem;
  border: 1px solid #dfe6e9;
  border-radius: 8px;
  font-size: 1rem;
}

.field input:focus,
.field textarea:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.2);
}

.upload-actions {
  display: flex;
  justify-content: flex-end;
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

.locked {
  opacity: 0.7;
  cursor: not-allowed;
  pointer-events: none;
}

.hint {
  font-size: 0.875rem;
  color: #7f8c8d;
  margin-top: 0.5rem;
}

.reference-card.locked {
  background: rgba(255, 255, 255, 0.6);
  border: 1px dashed #bdc3c7;
}

.hint {
  color: #7f8c8d;
  font-size: 0.95rem;
}

.upload-section.locked {
  background: rgba(255, 255, 255, 0.6);
  border: 1px dashed #bdc3c7;
}

.chart-placeholder.error {
  color: #c0392b;
  background: rgba(231, 76, 60, 0.08);
  padding: 1rem;
  border-radius: 8px;
}

.stat-card.alert-card {
  border: 1px solid rgba(231, 76, 60, 0.35);
  box-shadow: 0 8px 16px rgba(231, 76, 60, 0.15);
}

.stat-subtext {
  margin-top: 0.5rem;
  font-size: 0.85rem;
  color: #c0392b;
}

@media (max-width: 768px) {
  .dashboard {
    padding: 1.25rem;
  }

  .chart {
    min-height: 260px;
  }
}
</style>