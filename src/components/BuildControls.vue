<template>
  <div class="pt-4 flex space-x-4">
    <button
      @click="$emit('build')"
      :disabled="status === 'building' || isCancelling"
      class="flex-1 py-2 px-4 bg-green-600 hover:bg-green-700 text-white rounded-lg transition disabled:bg-gray-400"
    >
      {{ status === 'building' ? 'Идет сборка...' : 'Собрать проект' }}
    </button>
    <button
      v-if="status === 'building'"
      @click="$emit('cancel')"
      :disabled="isCancelling"
      class="flex-1 py-2 px-4 bg-red-600 hover:bg-red-700 text-white rounded-lg transition disabled:bg-gray-400"
    >
      {{ isCancelling ? 'Отмена...' : 'Отменить сборку' }}
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  status: 'idle' | 'building' | 'success' | 'error' | 'cancelled';
  isCancelling: boolean;
}>();

defineEmits(['build', 'cancel']);
</script>
