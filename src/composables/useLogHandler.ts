import { watch, nextTick, type Ref } from 'vue';
import { formatTimestamp } from '../services/buildService';

export function useLogHandler(logs: Ref<string[]>, container?: Ref<HTMLElement | null>) {
  watch(logs, async () => {
    await nextTick();
    if (container?.value) {
      const { scrollTop, scrollHeight, clientHeight } = container.value;
      const isNearBottom = scrollTop + clientHeight >= scrollHeight - 50;
      if (isNearBottom) {
        requestAnimationFrame(() => {
          container.value?.scrollTo({
            top: scrollHeight,
            behavior: 'smooth'
          });
        });
      }
    }
  }, { deep: true });

  function addLog(message: string) {
    logs.value.push(`[${formatTimestamp()}] ${message}`);
  }

  return { addLog };
}
