<template>
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700">Clean Before Build</label>
    <label class="flex items-center">
      <input
        type="checkbox"
        :checked="cleanBuild"
        :disabled="disabled"
        @change="$emit('update:cleanBuild', ($event.target as HTMLInputElement).checked)"
        class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded disabled:opacity-50"
      />
      <span class="ml-2 text-sm relative group">
        Use -cleanBuild
        <span class="absolute hidden group-hover:block bg-gray-800 text-white text-xs rounded p-2 -mt-8 left-0 z-10">
          Forces a full rebuild by cleaning previous build artifacts.
        </span>
      </span>
    </label>
  </div>
</template>

<script setup lang="ts">
import type { LocalBuildConfig } from '../types/index';

defineProps<{
  cleanBuild: LocalBuildConfig['cleanBuild'];
  disabled?: boolean;
}>();

defineEmits(['update:cleanBuild']);
</script>