<template>
  <div v-if="status !== 'idle'" class="space-y-2">
    <div v-if="status === 'building'" class="flex flex-col items-center justify-center space-y-2">
      <div class="flex items-center">
        <svg
          class="animate-spin h-5 w-5 text-blue-600"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          ></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8v8H4z"
          ></path>
        </svg>
        <span class="ml-2 text-sm text-gray-600">{{ currentStdout || 'Current Build Output...' }}</span>
      </div>
    </div>
    <div class="flex justify-end">
      <button
        v-if="messages.length"
        @click="clearMessages"
        class="text-sm text-blue-600 hover:underline"
      >
        Clear Messages
      </button>
    </div>
    <TransitionGroup
      name="fade"
      tag="div"
      class="space-y-2"
    >
      <div
        v-for="(message, index) in messages"
        :key="index"
        :class="[
          'p-4 rounded-lg text-sm transition-all duration-300 ease-in-out',
          message.type === 'success' ? 'bg-green-100 text-green-800 border-l-4 border-green-500' : 'bg-red-100 text-red-800 border-l-4 border-red-500'
        ]"
      >
        <div class="flex items-center">
          <svg v-if="message.type === 'success'" class="w-5 h-5 text-green-500 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <svg v-else class="w-5 h-5 text-red-500 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
          {{ message.text }}
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import type { BuildStatusType, BuildMessage } from '../types/index';

const props = defineProps<{
  status: BuildStatusType;
  messages: BuildMessage[];
  currentStdout: string;
}>();
const emit = defineEmits(['clear-messages']);

// Fix: clearMessages must mutate the messages array in parent
function clearMessages() {
  emit('clear-messages');
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>