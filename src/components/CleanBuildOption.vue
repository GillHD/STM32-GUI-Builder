<template>
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700">Очистить перед сборкой</label>
    <label class="flex items-center">
      <input
        type="checkbox"
        :checked="clean"
        @change="$emit('update:clean', ($event.target as HTMLInputElement).checked)"
        class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
      />
      <span class="ml-2 text-sm">Использовать -cleanBuild</span>
    </label>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  clean: boolean;
}>();

defineEmits(['update:clean']);
</script>
