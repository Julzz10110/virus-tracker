import { defineStore } from 'pinia';
import api from '../services/api';

export const useAnalyticsStore = defineStore('analytics', {
    state: () => ({
        growthHistory: [],
        growthLoading: false,
        growthError: null,
        liveGrowth: null,
        phylogeny: null,
        phylogenyLoading: false,
        phylogenyError: null,
    }),
    actions: {
        async fetchGrowthHistory(limit = 30) {
            this.growthLoading = true;
            this.growthError = null;
            try {
                const response = await api.get('/metrics/growth', {
                    params: { limit },
                });
                this.growthHistory = response.data ?? [];
            } catch (error) {
                this.growthError = error;
                this.growthHistory = [];
            } finally {
                this.growthLoading = false;
            }
        },
        async fetchLiveGrowth(params = {}) {
            try {
                const response = await api.get('/metrics/growth/live', {
                    params,
                });
                this.liveGrowth = response.data ?? null;
                this.growthError = null;
            } catch (error) {
                this.liveGrowth = null;
                this.growthError = error;
            }
        },
        async fetchPhylogeny(filters = {}) {
            this.phylogenyLoading = true;
            this.phylogenyError = null;
            try {
                const response = await api.get('/analysis/phylogeny', {
                    params: filters,
                });
                this.phylogeny = response.data ?? null;
            } catch (error) {
                this.phylogenyError = error;
                this.phylogeny = null;
            } finally {
                this.phylogenyLoading = false;
            }
        },
        resetPhylogeny() {
            this.phylogeny = null;
            this.phylogenyError = null;
        },
    },
});
