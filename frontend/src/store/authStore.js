import { defineStore } from 'pinia';
import api from '../services/api';

const TOKEN_KEY = 'auth_token';
const ROLES_KEY = 'auth_roles';

const loadRoles = () => {
  try {
    const stored = window.localStorage.getItem(ROLES_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch (_err) {
    return [];
  }
};

export const useAuthStore = defineStore('auth-store', {
  state: () => ({
    token: window.localStorage.getItem(TOKEN_KEY) || '',
    roles: loadRoles(),
    loading: false,
    error: null,
    users: [],
  }),
  getters: {
    isAuthenticated(state) {
      return Boolean(state.token);
    },
    isAdmin(state) {
      return state.roles.includes('admin');
    },
    canUpload(state) {
      return state.roles.includes('admin') || state.roles.includes('uploader');
    },
  },
  actions: {
    setToken(token, roles = []) {
      this.token = token;
      this.roles = roles;
      window.localStorage.setItem(TOKEN_KEY, token);
      window.localStorage.setItem(ROLES_KEY, JSON.stringify(roles));
    },
    clearAuth() {
      this.token = '';
      this.roles = [];
      window.localStorage.removeItem(TOKEN_KEY);
      window.localStorage.removeItem(ROLES_KEY);
    },
    async login(credentials) {
      this.loading = true;
      this.error = null;
      try {
        const { data } = await api.post('/auth/login', credentials);
        this.setToken(data.token, data.roles || []);
      } catch (error) {
        this.clearAuth();
        this.error = error.message;
        throw error;
      } finally {
        this.loading = false;
      }
    },
    logout() {
      this.clearAuth();
    },
    async fetchUsers() {
      if (!this.isAuthenticated) {
        return [];
      }
      this.loading = true;
      this.error = null;
      try {
        const { data } = await api.get('/admin/users');
        this.users = data;
        return data;
      } catch (error) {
        this.error = error.message;
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async createUser(payload) {
      this.loading = true;
      this.error = null;
      try {
        await api.post('/admin/users', payload);
        await this.fetchUsers();
      } catch (error) {
        this.error = error.message;
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});

