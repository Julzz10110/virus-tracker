import axios from 'axios';

const apiBaseUrl =
  import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';

const api = axios.create({
  baseURL: apiBaseUrl,
  timeout: 15_000,
});

api.interceptors.request.use((config) => {
  const token = window.localStorage.getItem('auth_token');
  if (token) {
    config.headers = config.headers || {};
    config.headers.Authorization = `Bearer ${token}`;
  }
  const apiKey = import.meta.env.VITE_API_KEY;
  if (apiKey) {
    config.headers = config.headers || {};
    config.headers['x-api-key'] = apiKey;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    const status = error?.response?.status;
    if (status === 401) {
      window.localStorage.removeItem('auth_token');
      window.localStorage.removeItem('auth_roles');
      if (window.location.pathname !== '/login') {
        window.location.assign('/login');
      }
    }
    const message =
      error?.response?.data?.error ||
      error?.response?.data?.message ||
      error?.message ||
      'Unexpected error occurred';
    return Promise.reject(new Error(message));
  }
);

export default api;


