<template>
  <section class="space-y-4">
    <h2 class="text-xl font-semibold text-gray-700 border-b pb-2">Project Settings</h2>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="space-y-2">
        <label for="project-path-btn" class="block text-sm font-medium text-gray-700">Project Directory</label>
        <div class="flex relative">
          <button
            id="project-path-btn"
            @click="$emit('select-project')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white transition"
            :class="modelValue.projectPath ? 'rounded-l-lg' : 'rounded-lg'"
            style="border-top-right-radius: 0; border-bottom-right-radius: 0;"
            v-if="modelValue.projectPath"
          >
            Select Project Directory
          </button>
          <button
            @click="$emit('select-project')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
            v-else
          >
            Select Project Directory
          </button>
          <button
            v-if="modelValue.projectPath"
            @click="clearPath('projectPath')"
            class="py-2 px-3 bg-gray-200 hover:bg-red-100 text-gray-500 hover:text-red-600 rounded-r-lg border-l border-gray-300 transition"
            title="Clear path"
            tabindex="-1"
            style="border-top-left-radius: 0; border-bottom-left-radius: 0;"
          >
            &times;
          </button>
        </div>
        <p v-if="modelValue.projectPath" class="text-xs text-gray-500 truncate">{{ modelValue.projectPath }}</p>
      </div>
      <div class="space-y-2">
        <label for="build-dir-btn" class="block text-sm font-medium text-gray-700">Build Directory</label>
        <div class="flex relative">
          <button
            id="build-dir-btn"
            @click="$emit('select-build-dir')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white transition"
            :class="modelValue.buildDir ? 'rounded-l-lg' : 'rounded-lg'"
            style="border-top-right-radius: 0; border-bottom-right-radius: 0;"
            v-if="modelValue.buildDir"
          >
            Select Build Directory
          </button>
          <button
            @click="$emit('select-build-dir')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
            v-else
          >
            Select Build Directory
          </button>
          <button
            v-if="modelValue.buildDir"
            @click="clearPath('buildDir')"
            class="py-2 px-3 bg-gray-200 hover:bg-red-100 text-gray-500 hover:text-red-600 rounded-r-lg border-l border-gray-300 transition"
            title="Clear path"
            tabindex="-1"
            style="border-top-left-radius: 0; border-bottom-left-radius: 0;"
          >
            &times;
          </button>
        </div>
        <p v-if="modelValue.buildDir" class="text-xs text-gray-500 truncate">{{ modelValue.buildDir }}</p>
      </div>
      <div class="space-y-2">
        <label for="workspace-path-btn" class="block text-sm font-medium text-gray-700">Workspace</label>
        <div class="flex relative">
          <button
            id="workspace-path-btn"
            @click="$emit('select-workspace')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white transition"
            :class="modelValue.workspacePath ? 'rounded-l-lg' : 'rounded-lg'"
            style="border-top-right-radius: 0; border-bottom-right-radius: 0;"
            v-if="modelValue.workspacePath"
          >
            Select Workspace
          </button>
          <button
            @click="$emit('select-workspace')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
            v-else
          >
            Select Workspace
          </button>
          <button
            v-if="modelValue.workspacePath"
            @click="clearPath('workspacePath')"
            class="py-2 px-3 bg-gray-200 hover:bg-red-100 text-gray-500 hover:text-red-600 rounded-r-lg border-l border-gray-300 transition"
            title="Clear path"
            tabindex="-1"
            style="border-top-left-radius: 0; border-bottom-left-radius: 0;"
          >
            &times;
          </button>
        </div>
        <p v-if="modelValue.workspacePath" class="text-xs text-gray-500 truncate">{{ modelValue.workspacePath }}</p>
      </div>
      <div class="space-y-2">
        <label for="cubeide-exe-btn" class="block text-sm font-medium text-gray-700">STM32CubeIDE EXE</label>
        <div class="flex relative">
          <button
            id="cubeide-exe-btn"
            @click="$emit('select-ide')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white transition"
            :class="modelValue.cubeIdeExePath ? 'rounded-l-lg' : 'rounded-lg'"
            style="border-top-right-radius: 0; border-bottom-right-radius: 0;"
            v-if="modelValue.cubeIdeExePath"
          >
            Select STM32CubeIDE EXE
          </button>
          <button
            @click="$emit('select-ide')"
            class="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
            v-else
          >
            Select STM32CubeIDE EXE
          </button>
          <button
            v-if="modelValue.cubeIdeExePath"
            @click="clearPath('cubeIdeExePath')"
            class="py-2 px-3 bg-gray-200 hover:bg-red-100 text-gray-500 hover:text-red-600 rounded-r-lg border-l border-gray-300 transition"
            title="Clear path"
            tabindex="-1"
            style="border-top-left-radius: 0; border-bottom-left-radius: 0;"
          >
            &times;
          </button>
        </div>
        <p v-if="modelValue.cubeIdeExePath" class="text-xs text-gray-500 truncate">{{ modelValue.cubeIdeExePath }}</p>
      </div>
    </div>
    <div class="grid grid-cols-1 gap-4">
      <!-- Add Build Settings Config Section -->
      <div class="space-y-2" v-if="projectSettings">
        <div class="flex items-center space-x-2">
          <svg class="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-600">Project-specific build settings found</span>
        </div>
      </div>
      <div class="space-y-2">
        <label for="external-settings-btn" class="block text-sm font-medium text-gray-700">Build Settings Configuration</label>
        <div v-if="projectSettings" class="flex items-center space-x-2 mb-2 p-2 bg-green-50 rounded">
          <svg class="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-600">Project-specific build settings found</span>
        </div>
        <div class="flex relative">
          <button
            id="external-settings-btn"
            @click="selectExternalSettings"
            :disabled="!modelValue.projectPath"
            class="flex-1 py-2 px-4 text-white transition flex items-center justify-center space-x-2"
            :class="externalSettingsPath ? 'rounded-l-lg' : 'rounded-lg'"
            :style="externalSettingsPath ? 'border-top-right-radius: 0; border-bottom-right-radius: 0;' : ''"
          >
            <span>Select External Build Settings</span>
            <span v-if="externalSettingsPath" class="text-xs bg-blue-500 px-2 py-0.5 rounded">
              Active
            </span>
          </button>
          <button
            v-if="externalSettingsPath"
            @click="clearPath('externalSettingsPath')"
            class="py-2 px-3 bg-gray-200 hover:bg-red-100 text-gray-500 hover:text-red-600 rounded-r-lg border-l border-gray-300 transition"
            title="Clear path"
            tabindex="-1"
            style="border-top-left-radius: 0; border-bottom-left-radius: 0;"
          >
            &times;
          </button>
        </div>
        <p v-if="externalSettingsPath" class="text-xs text-gray-500 truncate">
          {{ externalSettingsPath }}
        </p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { Settings } from '../types/index';
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const props = defineProps<{
  modelValue: Settings;
}>();

const emit = defineEmits(['update:modelValue', 'select-project', 'select-build-dir', 'select-workspace', 'select-ide']);

const projectSettings = ref(false);
const externalSettingsPath = ref(props.modelValue.externalSettingsPath || '');

// Check for project-specific build settings when project path changes
watch(() => props.modelValue.projectPath, async (newPath) => {
  if (newPath) {
    try {
      projectSettings.value = await invoke('check_project_settings', { projectPath: newPath });
    } catch (e) {
      console.error('Failed to check project settings:', e);
      projectSettings.value = false;
    }
  } else {
    projectSettings.value = false;
  }
});

// Обновляем externalSettingsPath при изменении в родителе
watch(() => props.modelValue.externalSettingsPath, (newPath) => {
  externalSettingsPath.value = newPath || '';
});

// Исправление: используем open из plugin-dialog для выбора внешнего файла
async function selectExternalSettings() {
  if (!props.modelValue.projectPath) return;

  try {
    const selected = await open({
      multiple: false,
      directory: false, // ВАЖНО: явно указываем, что нужен файл, а не папка!
      filters: [{
        name: 'Build Settings',
        extensions: ['yaml', 'yml']  // Changed from json to yaml
      }]
    });

    // Для plugin-dialog selected может быть строкой или массивом строк
    const filePath = Array.isArray(selected) ? selected[0] : selected;
    if (filePath && typeof filePath === 'string') {
      externalSettingsPath.value = filePath;
      emit('update:modelValue', {
        ...props.modelValue,
        externalSettingsPath: filePath
      });
    }
  } catch (e) {
    console.error('Failed to select external settings file:', e);
  }
}

// Кнопка очистки любого пути
function clearPath(key: keyof Settings) {
  emit('update:modelValue', {
    ...props.modelValue,
    [key]: null
  });
  if (key === 'externalSettingsPath') {
    externalSettingsPath.value = '';
  }
}

// Если вы хотите чтобы кнопки выбора директорий и exe работали прямо здесь (без emit):
// Ниже пример для выбора директории проекта (аналогично для других кнопок):

/*
async function selectProjectDirectory() {
  try {
    const selected = await open({ directory: true, multiple: false });
    const dirPath = Array.isArray(selected) ? selected[0] : selected;
    if (dirPath && typeof dirPath === 'string') {
      emit('update:modelValue', { ...props.modelValue, projectPath: dirPath });
    }
  } catch (e) {
    console.error('Failed to select project directory:', e);
  }
}
*/

</script>