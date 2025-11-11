import { createApp } from 'vue';
import { createPinia } from 'pinia';

import { Buffer } from 'buffer';
import process from 'process';

if (typeof globalThis.global === 'undefined') {
  globalThis.global = globalThis;
}

globalThis.Buffer = globalThis.Buffer || Buffer;
globalThis.process = globalThis.process || process;

import App from './App.vue';
import router from './router';

import './styles/global.css';

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.mount('#app');

