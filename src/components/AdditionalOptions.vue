<template>
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700">{{ setting.label }}</label>
    <div class="space-y-2">
      <label v-for="opt in setting.options" :key="opt.value" :for="opt.value" class="flex items-start">
        <input
          :id="opt.value"
          type="checkbox"
          :value="opt.value"
          :checked="value.includes(opt.value)"
          @change="updateOptions(opt.value)"
          class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded mt-1"
        />
        <div class="ml-2">
          <span class="text-sm">{{ opt.label }}</span>
          <p v-if="opt.description" class="text-xs text-gray-500">{{ opt.description }}</p>
        </div>
      </label>
      <p v-if="validationError" class="text-sm text-red-600">{{ validationError }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType, ref } from 'vue';
import type { BuildSettingsConfig } from '../types/index';

const props = defineProps({
  value: {
    type: Array as PropType<string[]>,
    required: true
  },
  setting: {
    type: Object as PropType<BuildSettingsConfig['build_settings'][0]>,
    required: true
  }
});

const emit = defineEmits(['update:value']);
const validationError = ref<string>('');

const updateOptions = (opt: string) => {
  const newOptions = props.value.includes(opt)
    ? props.value.filter(o => o !== opt)
    : [...props.value, opt];
  if (props.setting.min_selected && newOptions.length < props.setting.min_selected) {
    validationError.value = `At least ${props.setting.min_selected} option(s) must be selected`;
  } else {
    validationError.value = '';
    emit('update:value', newOptions);
  }
};
</script>