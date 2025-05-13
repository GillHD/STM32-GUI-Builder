<template>
  <div class="build-settings space-y-8">
    <!-- General Settings Section -->
    <div class="section-container">
      <h2 class="section-header">General Settings</h2>
      <div class="form-group">
        <div class="flex items-center space-x-3 p-3 bg-gray-50 rounded-md">
          <input
            id="clean-build-checkbox"
            type="checkbox"
            v-model="localBuildConfig.cleanBuild"
            class="h-5 w-5 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
            @change="(e: Event) => updateValue('cleanBuild', (e.target as HTMLInputElement).checked)"
          />
          <span class="text-sm font-medium text-gray-700">Clean Build</span>
        </div>

        <div class="space-y-2">
          <label for="custom-console-args" class="block text-sm font-medium text-gray-700">
            Custom Console Arguments
            <span class="ml-1 text-xs text-gray-500 cursor-help" title="Additional command-line arguments for STM32CubeIDE">
              (?)
            </span>
          </label>
          <input
            id="custom-console-args"
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
            <label for="project-name-select" class="block text-sm font-medium text-gray-700">Project Name</label>
            <select
              id="project-name-select"
              v-model="localBuildConfig.projectName"
              :disabled="!projectPath"
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              @change="(e: Event) => updateValue('projectName', (e.target as HTMLSelectElement).value)"
            >
              <option value="">Select project...</option>
              <option v-if="projectName" :value="projectName">
                {{ projectName }}
              </option>
            </select>
          </div>

          <div class="space-y-2">
            <label for="build-config-select" class="block text-sm font-medium text-gray-700">Build Configuration</label>
            <select
              id="build-config-select"
              v-model="localBuildConfig.configName"
              :disabled="!projectPath || !configurations.length"
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              @change="(e: Event) => updateValue('configName', (e.target as HTMLSelectElement).value)"
            >
              <option value="">Select configuration...</option>
              <option v-for="config in configurations" :key="config" :value="config">
                {{ config }}
              </option>
            </select>
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
            <label
              v-if="setting.field_type !== 'checkbox_group'"
              :for="setting.id"
              class="block text-sm font-semibold text-gray-900"
            >
              {{ setting.label }}
            </label>
            <span
              v-else
              class="block text-sm font-semibold text-gray-900"
            >
              {{ setting.label }}
            </span>
            <p class="text-sm text-gray-500">{{ setting.description }}</p>
          </div>

          <!-- Range Input -->
          <div v-if="setting.field_type === 'range'" class="mt-2">
            <input
              :id="setting.id"
              v-model="localSettings[setting.id]"
              type="text"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              :placeholder="`Range: ${setting.validation?.min}-${setting.validation?.max}`"
              @input="(e) => onRangeInput(setting, e)"
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
import { ref, watch, onMounted } from 'vue';
import type { LocalBuildConfig, BuildSettingsConfig } from '../types/index';
import { validateNumericRange } from '../utils/range-parser';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  modelValue: LocalBuildConfig;
  buildSettings: BuildSettingsConfig;
  projectPath: string;
  buildDir: string;
  workspacePath: string;
  cubeIdeExePath: string;
}>();
const emit = defineEmits(['update:modelValue']);

const localSettings = ref({ ...props.modelValue.settings });
const validationErrors = ref<{ [key: string]: string }>({});
const localBuildConfig = ref({ ...props.modelValue });
const configurations = ref<string[]>([]);
const projectName = ref<string>('');

// Load configurations when project path changes
watch(() => props.projectPath, async (newPath) => {
  if (newPath) {
    try {
      configurations.value = await invoke('get_project_configurations', { 
        projectPath: newPath 
      });
    } catch (e) {
      console.error('Failed to load configurations:', e);
      configurations.value = [];
    }
  } else {
    configurations.value = [];
  }
});

// Load project name when project path changes
watch(() => props.projectPath, async (newPath) => {
  if (newPath) {
    try {
      projectName.value = await invoke('get_project_name_from_path', { 
        projectPath: newPath 
      });
    } catch (e) {
      console.error('Failed to load project name:', e);
      projectName.value = '';
    }
  } else {
    projectName.value = '';
  }
});

// Initialize configurations if project path exists
onMounted(async () => {
  if (props.projectPath) {
    try {
      configurations.value = await invoke('get_project_configurations', {
        projectPath: props.projectPath
      });
    } catch (e) {
      console.error('Failed to load initial configurations:', e);
    }
  }
});

// Initialize project name if project path exists
onMounted(async () => {
  if (props.projectPath) {
    try {
      projectName.value = await invoke('get_project_name_from_path', {
        projectPath: props.projectPath
      });
    } catch (e) {
      console.error('Failed to load initial project name:', e);
    }
  }
});

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

// Обработчик для поля range, сохраняет строку для отображения и в settings
function onRangeInput(setting: BuildSettingsConfig['build_settings'][0], event: Event) {
  const value = (event.target as HTMLInputElement).value;

  // Проверяем валидность
  if (setting.validation) {
    validationErrors.value[setting.id] = validateNumericRange(value, setting.validation.min, setting.validation.max) 
      ? '' 
      : `Invalid range. Use format like "11, 23-26, 30" within [${setting.validation.min}, ${setting.validation.max}]`;
  }

  // Обновляем значение в localSettings и эмитим изменение
  localSettings.value[setting.id] = value;
  updateValue(`settings.${setting.id}`, value);
}

// Удалить лишние watch
// Оставить только один:
watch(
  () => props.modelValue,
  (newValue) => {
    localBuildConfig.value = { ...newValue };
    localSettings.value = { ...newValue.settings };
  },
  { deep: true, immediate: true }
);

defineExpose({ buildProject });
</script>