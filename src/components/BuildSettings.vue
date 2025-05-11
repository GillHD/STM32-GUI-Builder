<template>
  <div class="build-settings space-y-8">
    <!-- General Settings Section -->
    <div class="section-container">
      <h2 class="section-header">General Settings</h2>
      <div class="form-group">
        <div class="flex items-center space-x-3 p-3 bg-gray-50 rounded-md">
          <input
            type="checkbox"
            v-model="localBuildConfig.cleanBuild"
            class="h-5 w-5 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
            @change="(e: Event) => updateValue('cleanBuild', (e.target as HTMLInputElement).checked)"
          />
          <span class="text-sm font-medium text-gray-700">Clean Build</span>
        </div>

        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700">
            Custom Console Arguments
            <span class="ml-1 text-xs text-gray-500 cursor-help" title="Additional command-line arguments for STM32CubeIDE">
              (?)
            </span>
          </label>
          <input
            type="text"
            v-model="localBuildConfig.customConsoleArgs"
            placeholder="Additional command line arguments"
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            @input="(e: Event) => updateValue('customConsoleArgs', (e.target as HTMLInputElement).value)"
          />
        </div>
      </div>
    </div>

    <!-- Build Configuration Section -->
    <div class="section-container">
      <h2 class="section-header">Build Configuration</h2>
      <div class="form-group">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700">Project Name</label>
            <input
              type="text"
              v-model="localBuildConfig.projectName"
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              @input="(e: Event) => updateValue('projectName', (e.target as HTMLInputElement).value)"
            />
          </div>

          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700">Build Configuration</label>
            <input
              type="text"
              v-model="localBuildConfig.configName"
              placeholder="e.g. Debug, Release"
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              @input="(e: Event) => updateValue('configName', (e.target as HTMLInputElement).value)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Build Parameters Section -->
    <div class="section-container">
      <h2 class="section-header">Build Parameters</h2>
      <div v-if="!buildSettings.build_settings.length" class="text-gray-500 italic text-center p-4">
        No build parameters defined
      </div>
      <div v-else class="space-y-6">
        <div v-for="setting in buildSettings.build_settings" :key="setting.id" 
             class="p-4 bg-gray-50 rounded-lg border border-gray-100">
          <div class="mb-3">
            <!-- Исправлено: for/id всегда совпадают -->
            <label :for="setting.id" class="block text-sm font-semibold text-gray-900">
              {{ setting.label }}
            </label>
            <p class="text-sm text-gray-500">{{ setting.description }}</p>
          </div>

          <!-- Range Input -->
          <div v-if="setting.field_type === 'range'" class="mt-2">
            <input
              :id="setting.id"
              :value="rangeInputs[setting.id] ?? ''"
              type="text"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              :placeholder="`Range: ${setting.validation?.min}-${setting.validation?.max}`"
              @input="onRangeInput(setting, $event)"
            />
            <p v-if="validationErrors[setting.id]" class="mt-1 text-sm text-red-600">
              {{ validationErrors[setting.id] }}
            </p>
          </div>

          <!-- Select Input -->
          <div v-else-if="setting.field_type === 'select'" class="mt-2">
            <select
              :id="setting.id"
              v-model="localSettings[setting.id]"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              @change="($event.target as HTMLSelectElement)?.value && updateValue(`settings.${setting.id}`, ($event.target as HTMLSelectElement).value)"
            >
              <option value="">Select option...</option>
              <option v-for="option in setting.options" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
            <p v-if="localSettings[setting.id] && setting.options" class="mt-2 text-sm text-gray-500">
              {{ setting.options.find(opt => opt.value === localSettings[setting.id])?.description }}
            </p>
          </div>

          <!-- Checkbox Group -->
          <div v-else-if="setting.field_type === 'checkbox_group'" class="mt-2 space-y-3">
            <div v-for="option in setting.options" :key="option.value" 
                 class="flex items-start space-x-3 p-2 hover:bg-white rounded-md transition-colors">
              <input
                :id="`${setting.id}-${option.value}`"
                type="checkbox"
                :value="option.value"
                :checked="localSettings[setting.id]?.includes(option.value)"
                class="mt-1 h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                @change="updateCheckboxGroup(setting, $event)"
              />
              <div>
                <label :for="`${setting.id}-${option.value}`" class="block text-sm font-medium text-gray-700">
                  {{ option.label }}
                </label>
                <p v-if="option.description" class="text-sm text-gray-500">
                  {{ option.description }}
                </p>
              </div>
            </div>
            <p v-if="validationErrors[setting.id]" class="mt-1 text-sm text-red-600">
              {{ validationErrors[setting.id] }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Reset Button -->
    <div class="flex justify-end">
      <button
        @click="resetSettings"
        class="py-2 px-4 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition"
      >
        Reset Settings
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { LocalBuildConfig, BuildSettingsConfig, Settings } from '../types/index';
import { parseNumericRange, validateNumericRange } from '../utils/range-parser';
import { invoke } from '@tauri-apps/api/core';


const props = defineProps<{
  modelValue: LocalBuildConfig;
  buildSettings: BuildSettingsConfig;
  // Добавьте эти строки:
  projectPath: string;
  buildDir: string;
  workspacePath: string;
  cubeIdeExePath: string;
}>();
const emit = defineEmits(['update:modelValue']);

const localSettings = ref({ ...props.modelValue.settings });
const validationErrors = ref<{ [key: string]: string }>({});
const localBuildConfig = ref({ ...props.modelValue });
const rangeInputs = ref<{ [key: string]: string }>({});

// Sync with modelValue
watch(
  () => props.modelValue,
  (newValue) => {
    localBuildConfig.value = { ...newValue };
    localSettings.value = { ...newValue.settings };
    validationErrors.value = {};
    // Only initialize rangeInputs if they're empty
    for (const setting of props.buildSettings.build_settings) {
      if (setting.field_type === 'range' && !rangeInputs.value[setting.id]) {
        const val = newValue.settings[setting.id];
        if (typeof val === 'string') {
          rangeInputs.value[setting.id] = val;
        }
      }
    }
  },
  { deep: true, immediate: true }
);

// Sync localSettings with modelValue.settings
watch(
  () => props.modelValue.settings,
  (newSettings) => {
    localSettings.value = { ...newSettings };
    validationErrors.value = {};
  },
  { deep: true }
);

// Validate and update range settings
const validateAndUpdate = /* debounce( */ (setting: BuildSettingsConfig['build_settings'][0], event: Event) => {
  const target = event.target;
  if (!(target instanceof HTMLInputElement)) {
    console.error(`Invalid event target for setting ${setting.id}:`, target);
    validationErrors.value[setting.id] = 'Invalid input element';
    return;
  }
  const value = target.value;
  rangeInputs.value[setting.id] = value; // Сохраняем строку как есть для отображения
  if (setting.validation) {
    if (validateNumericRange(value, setting.validation.min, setting.validation.max)) {
      const parsed = parseNumericRange(value, setting.validation.min, setting.validation.max);
      updateValue(`settings.${setting.id}`, parsed);
      validationErrors.value[setting.id] = '';
    } else {
      validationErrors.value[setting.id] = `Invalid range. Use format like "11, 23-26, 30" within [${setting.validation.min}, ${setting.validation.max}]`;
    }
  }
}/* , 300) */; // Uncomment debounce if using lodash

// Update checkbox group settings
const updateCheckboxGroup = (setting: BuildSettingsConfig['build_settings'][0], event: Event) => {
  const target = event.target;
  if (!(target instanceof HTMLInputElement)) {
    console.error(`Invalid event target for setting ${setting.id}:`, target);
    validationErrors.value[setting.id] = 'Invalid checkbox element';
    return;
  }
  const value = target.value;
  const checked = target.checked;
  const currentValues = (localSettings.value[setting.id] as string[] | undefined) || [];
  let newValues = [...currentValues];

  if (checked && !newValues.includes(value)) {
    newValues.push(value);
  } else if (!checked) {
    newValues = newValues.filter((v) => v !== value);
  }

  if (setting.min_selected && newValues.length < setting.min_selected) {
    validationErrors.value[setting.id] = `At least ${setting.min_selected} option(s) must be selected`;
  } else {
    validationErrors.value[setting.id] = '';
    updateValue(`settings.${setting.id}`, newValues);
  }
};

// Update modelValue
const updateValue = (key: string, value: any) => {
  if (key.startsWith('settings.')) {
    const settingKey = key.split('.')[1];
    localSettings.value[settingKey] = value;
    emit('update:modelValue', {
      ...props.modelValue,
      projectPath: props.projectPath,
      buildDir: props.buildDir,
      workspacePath: props.workspacePath,
      cubeIdeExePath: props.cubeIdeExePath,
      settings: {
        ...props.modelValue.settings,
        [settingKey]: value,
      },
    });
  } else {
    emit('update:modelValue', {
      ...props.modelValue,
      projectPath: props.projectPath,
      buildDir: props.buildDir,
      workspacePath: props.workspacePath,
      cubeIdeExePath: props.cubeIdeExePath,
      [key]: value,
    });
  }
};

// Reset settings to defaults
const resetSettings = () => {
  localSettings.value = {};
  validationErrors.value = {};
  emit('update:modelValue', {
    ...props.modelValue,
    projectPath: props.projectPath,
    buildDir: props.buildDir,
    workspacePath: props.workspacePath,
    cubeIdeExePath: props.cubeIdeExePath,
    settings: {},
    cleanBuild: false,
    customConsoleArgs: null,
    projectName: null,
    configName: null
  });
};

// Пример вызова сборки (теперь используем props для путей):
const buildProject = async () => {
  const buildSettingsKeys = props.buildSettings.build_settings.map(s => s.id);
  const fullSettings: Record<string, any> = { ...localBuildConfig.value.settings };
  for (const key of buildSettingsKeys) {
    const setting = props.buildSettings.build_settings.find(s => s.id === key);
    if (!(key in fullSettings)) {
      if (setting?.field_type === 'checkbox_group' || setting?.field_type === 'range') {
        fullSettings[key] = [];
      } else {
        fullSettings[key] = null;
      }
    }
    // Удаляем пустые строки/массивы для необязательных параметров
    if (
      (setting?.field_type === 'select' && (!fullSettings[key] || fullSettings[key].trim() === '')) ||
      ((setting?.field_type === 'checkbox_group' || setting?.field_type === 'range') &&
        Array.isArray(fullSettings[key]) &&
        fullSettings[key].every((v: any) => typeof v === 'string' ? v.trim() === '' : false) &&
        !setting?.min_selected
      )
    ) {
      delete fullSettings[key];
    }
  }
  await invoke('build_project', {
    config: {
      projectPath: props.projectPath,
      buildDir: props.buildDir,
      workspacePath: props.workspacePath,
      cubeIdeExePath: props.cubeIdeExePath,
      projectName: localBuildConfig.value.projectName,
      configName: localBuildConfig.value.configName,
      cleanBuild: localBuildConfig.value.cleanBuild,
      customConsoleArgs: localBuildConfig.value.customConsoleArgs,
      settings: fullSettings,
      cancelled: (localBuildConfig.value as any).cancelled ?? false
    }
  });
};

// Обработчик для поля range, сохраняет строку для отображения, а в модель кладёт массив
function onRangeInput(setting: BuildSettingsConfig['build_settings'][0], event: Event) {
  const target = event.target as HTMLInputElement;
  const value = target.value;
  rangeInputs.value[setting.id] = value; // Always save what user types
  
  if (value.trim() === '') {
    // Clear validation error if field is empty
    validationErrors.value[setting.id] = '';
    updateValue(`settings.${setting.id}`, []);
    return;
  }

  if (setting.validation) {
    if (validateNumericRange(value, setting.validation.min, setting.validation.max)) {
      const parsed = parseNumericRange(value, setting.validation.min, setting.validation.max);
      updateValue(`settings.${setting.id}`, parsed);
      validationErrors.value[setting.id] = '';
    } else {
      // Don't clear input on validation error
      validationErrors.value[setting.id] = `Invalid range. Use format like "11, 23-26, 30" within [${setting.validation.min}, ${setting.validation.max}]`;
    }
  }
}

defineExpose({ buildProject });
</script>