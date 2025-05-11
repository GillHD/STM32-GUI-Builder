import { createApp } from "vue";
import App from "./App.vue";
import './style.css';

// Импорт плагинов Tauri для глобальной доступности
import '@tauri-apps/plugin-dialog';

// Отключение контекстного меню в продакшене
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (event) => event.preventDefault());
}

createApp(App).mount("#app");