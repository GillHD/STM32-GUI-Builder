<template>
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700">{{ setting.name }}</label>
    <div class="space-y-2">
      <label v-for="opt in setting.options" :key="opt.value" :for="opt.value" class="flex items-center">
        <input
          :id="opt.value"
          type="checkbox"
          :value="opt.value"
          :checked="value.includes(opt.value)"
          @change="updateOptions(opt.value)"
          class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        />
        <span class="ml-2 text-sm">{{ opt.label }}</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType } from 'vue';
import type { BuildSetting } from '../types/build';

const props = defineProps({
  value: {
    type: Array as PropType<string[]>,
    required: true
  },
  setting: {
    type: Object as PropType<BuildSetting>,
    required: true
  }
});

const emit = defineEmits(['update:value']);

const updateOptions = (opt: string) => {
  const newOptions = props.value.includes(opt)
    ? props.value.filter(o => o !== opt)
    : [...props.value, opt];
  emit('update:value', newOptions);
};
</script>