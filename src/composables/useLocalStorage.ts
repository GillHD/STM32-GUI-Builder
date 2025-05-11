import { watch } from 'vue';
import type { Ref } from 'vue';

export function useLocalStorage<T>(key: string, data: Ref<T>) {
  // Load data on init
  const stored = localStorage.getItem(key);
  if (stored) {
    try {
      data.value = JSON.parse(stored);
    } catch (e) {
      console.error(`Error loading ${key} from localStorage:`, e);
    }
  }

  // Watch for changes
  watch(
    () => data.value,
    (newValue) => {
      localStorage.setItem(key, JSON.stringify(newValue));
    },
    { deep: true }
  );

  return data;
}
