<template>
  <section class="space-y-2">
    <h2 class="text-xl font-semibold text-gray-700 border-b pb-2">Build Logs</h2>
    <div
      ref="logContainerEl"
      class="bg-gray-900 text-white p-4 rounded-lg font-mono text-sm leading-relaxed whitespace-pre-wrap"
      :class="{
        'overflow-y-auto': logs && logs.length > 0,
        'overflow-y-hidden': !logs || logs.length === 0
      }"
      style="height: 28px * 10; max-height: 280px; min-height: 280px;"
    >
      <p v-if="!logs?.length" class="text-gray-400">No logs available</p>
      <RecycleScroller
        v-else
        :items="logs"
        :item-size="28"
        key-field="logKey"
        class="h-full"
      >
        <template #default="{ item, index }">
          <div
            :key="`${index}-${item.slice(0, 20)}`"
            class="mb-1"
            :class="[
              item.includes('[INFO]') ? 'text-green-400'
                : item.includes('[ERROR]') ? 'text-red-400'
                : 'text-gray-400'
            ]"
            style="white-space: pre-wrap;"
          >
            {{ item }}
          </div>
        </template>
      </RecycleScroller>
      <div v-else>
        <div
          v-for="(item, index) in logs"
          :key="`${index}-${item.slice(0, 20)}`"
          class="mb-1"
          :class="[
            item.includes('[INFO]') ? 'text-green-400'
              : item.includes('[ERROR]') ? 'text-red-400'
              : 'text-gray-400'
          ]"
          style="white-space: pre-wrap;"
        >
          {{ item }}
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { RecycleScroller } from 'vue3-virtual-scroller';

const props = defineProps<{
  logs: string[];
}>();

const logContainerEl = ref<HTMLElement | null>(null);

// Autoscroll when new logs are added
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
});

defineExpose({
  logContainer: logContainerEl
});
</script>