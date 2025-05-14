<template>
  <div class="pt-4 flex space-x-4">
    <button
      @click="$emit('build')"
      :disabled="status === 'building' || isCancelling"
      class="flex-1 py-2 px-4 bg-green-600 hover:bg-green-700 text-white rounded-lg transition disabled:bg-gray-400 disabled:cursor-not-allowed"
    >
      {{ status === 'building' ? 'Building...' : 'Build Project' }}
    </button>
    <button
      v-if="status === 'building'"
      @click="confirmCancel"
      :disabled="isCancelling"
      class="flex-1 py-2 px-4 bg-red-600 hover:bg-red-700 text-white rounded-lg transition disabled:bg-gray-400 disabled:cursor-not-allowed"
    >
      {{ isCancelling ? 'Cancelling...' : 'Cancel Build' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import type { BuildStatusType } from '../types/index';

const props = defineProps<{
  status: BuildStatusType;
  isCancelling: boolean;
}>();

const emit = defineEmits(['build', 'cancel']);

const confirmCancel = () => {
  if (window.confirm('Are you sure you want to cancel the build?')) {
    emit('cancel');
  }
};

// Keyboard shortcut for build (Ctrl+B)
onMounted(() => {
  const handleKeydown = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.key === 'b' && props.status !== 'building' && !props.isCancelling) {
      emit('build');
    }
  };
  window.addEventListener('keydown', handleKeydown);
  onUnmounted(() => window.removeEventListener('keydown', handleKeydown));
});
</script>