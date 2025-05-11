<template>
  <section class="space-y-2">
    <h2 class="text-xl font-semibold text-gray-700 border-b pb-2">Логи сборки</h2>
    <div
      ref="logContainerEl"
      class="bg-gray-900 text-white p-4 rounded-lg h-[400px] overflow-y-auto font-mono text-sm leading-relaxed whitespace-pre-wrap"
    >
      <p v-if="!logs?.length" class="text-gray-400">Логи отсутствуют</p>
      <template v-else>
        <p v-for="(log, index) in logs" :key="index" class="mb-1">{{ log }}</p>
      </template>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  logs: string[];
}>();

const logContainerEl = ref<HTMLElement | null>(null);

// Автоскролл при новых логах
watch(() => props.logs, async () => {
  if (logContainerEl.value) {
    await nextTick();
    const el = logContainerEl.value;
    const isAtBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 100;
    if (isAtBottom) {
      requestAnimationFrame(() => {
        if (el) {
          el.scrollTop = el.scrollHeight;
        }
      });
    }
  }
}, { deep: true });

defineExpose({
  logContainer: logContainerEl
});
</script>
