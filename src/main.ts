import { createApp } from "vue";
import App from "./App.vue";
import './style.css';
import VueVirtualScroller from 'vue3-virtual-scroller';
import 'vue3-virtual-scroller/dist/vue3-virtual-scroller.css';

// Импорт плагинов Tauri для глобальной доступности
import '@tauri-apps/plugin-dialog';

// Отключение контекстного меню в продакшене
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (event) => event.preventDefault());
}

const app = createApp(App);
app.use(VueVirtualScroller);
app.mount("#app");