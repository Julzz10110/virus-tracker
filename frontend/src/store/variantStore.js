import { defineStore } from 'pinia';
import api from '../services/api';

const parseFilters = (filters = {}) => {
  const params = {};
  if (filters.lineage) {
    params.lineage = filters.lineage.trim();
  }
  if (filters.location) {
    params.location = filters.location.trim();
  }
  return params;
};

export const useVariantStore = defineStore('variant-store', {
  state: () => ({
    variants: [],
    stats: null,
    reference: '',
    mutationFrequency: [],
    loading: false,
    error: null,
  }),
  actions: {
    setError(error) {
      this.error = error instanceof Error ? error.message : String(error);
    },
    clearError() {
      this.error = null;
    },
    async fetchVariants(filters = {}) {
      this.loading = true;
      this.clearError();
      try {
        const response = await api.get('/variants', {
          params: parseFilters(filters),
        });
        this.variants = response.data;
      } catch (error) {
        this.setError(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async fetchStats() {
      this.clearError();
      try {
        const response = await api.get('/variants/stats');
        this.stats = response.data;
      } catch (error) {
        this.setError(error);
        throw error;
      }
    },
    async uploadVariant(payload) {
      this.clearError();
      try {
        await api.post('/upload', payload);
      } catch (error) {
        this.setError(error);
        throw error;
      }
    },
    async fetchReference() {
      this.clearError();
      try {
        const response = await api.get('/reference');
        this.reference = response.data.reference;
      } catch (error) {
        this.setError(error);
        throw error;
      }
    },
    async updateReference(reference) {
      this.clearError();
      try {
        const response = await api.post('/reference', { reference });
        this.reference = response.data.reference;
      } catch (error) {
        this.setError(error);
        throw error;
      }
    },
    async fetchMutationFrequency(filters = {}) {
      this.clearError();
      try {
        const response = await api.get('/analysis/frequency', {
          params: parseFilters(filters),
        });
        this.mutationFrequency = response.data;
      } catch (error) {
        this.setError(error);
        throw error;
      }
    },
  },
});

