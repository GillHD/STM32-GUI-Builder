<template>
  <section class="space-y-4">
    <h2 class="text-xl font-semibold text-gray-700 border-b pb-2">Настройки проекта</h2>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="space-y-2">
        <label class="block text-sm font-medium text-gray-700">Директория проекта</label>
        <button
          @click="$emit('select-project')"
          class="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
        >
          Выбрать директорию проекта
        </button>
        <p v-if="modelValue.projectPath" class="text-xs text-gray-500 truncate">{{ modelValue.projectPath }}</p>
        <!-- Uncomment after implementing validate_path command -->
        <!-- <p v-if="modelValue.projectPath && !isValidProjectPath" class="text-xs text-red-500">Invalid project directory</p> -->
      </div>
      <div class="space-y-2">
        <label class="block text-sm font-medium text-gray-700">Папка для билдов</label>
        <button
          @click="$emit('select-build-dir')"
          class="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
        >
          Выбрать папку для билдов
        </button>
        <p v-if="modelValue.buildDir" class="text-xs text-gray-500 truncate">{{ modelValue.buildDir }}</p>
      </div>
      <div class="space-y-2">
        <label class="block text-sm font-medium text-gray-700">Рабочее пространство</label>
        <button
          @click="$emit('select-workspace')"
          class="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
        >
          Выбрать рабочее пространство
        </button>
        <p v-if="modelValue.workspacePath" class="text-xs text-gray-500 truncate">{{ modelValue.workspacePath }}</p>
      </div>
      <div class="space-y-2">
        <label class="block text-sm font-medium text-gray-700">STM32CubeIDE EXE</label>
        <button
          @click="$emit('select-ide')"
          class="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition"
        >
          Выбрать STM32CubeIDE EXE
        </button>
        <p v-if="modelValue.cubeIdeExePath" class="text-xs text-gray-500 truncate">{{ modelValue.cubeIdeExePath }}</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { Settings } from '../types/index';
// import { ref } from 'vue';
// import { invoke } from '@tauri-apps/api/tauri';

defineProps<{
  modelValue: Settings;
}>();
defineEmits(['update:modelValue', 'select-project', 'select-build-dir', 'select-workspace', 'select-ide']);

// В emit('update:modelValue', ...) всегда передавайте объект с projectPath, buildDir, workspacePath, cubeIdeExePath
// Path validation (requires backend validate_path command)
// const isValidProjectPath = ref<boolean>(true);
// const validatePath = async (path: string): Promise<boolean> => {
//   try {
//     await invoke('validate_path', { path });
//     return true;
//   } catch {
//     return false;
//   }
// };
// watch(() => props.modelValue.projectPath, async (newPath) => {
//   if (newPath) {
//     isValidProjectPath.value = await validatePath(newPath);
//   }
// });
</script>