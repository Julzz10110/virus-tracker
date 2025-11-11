import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      buffer: 'buffer',
      'buffer/': 'buffer/',
      process: 'process/browser',
    },
  },
  optimizeDeps: {
    include: ['buffer'],
  },
});