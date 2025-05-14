import { createApp } from "vue";
import App from "./App.vue";
import './style.css';
import VueVirtualScroller from 'vue3-virtual-scroller';
import 'vue3-virtual-scroller/dist/vue3-virtual-scroller.css';

// Import Tauri plugins for global accessibility
import '@tauri-apps/plugin-dialog';

// Disable context menu in production
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (event) => event.preventDefault());
}

const app = createApp(App);
app.use(VueVirtualScroller);
app.mount("#app");