<template>
  <div class="min-h-screen bg-gray-100">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-6">
      <!-- Header -->
      <h1 class="text-2xl sm:text-3xl font-bold text-center text-gray-800 mb-4 sm:mb-6">STM32 Builder</h1>

      <!-- Main Content -->
      <div class="grid grid-cols-1 xl:grid-cols-2 gap-4">
        <!-- Left Side -->
        <div class="flex flex-col space-y-4">
          <!-- Left Column Content -->
          <div class="flex flex-col space-y-4">
            <ProjectSettings 
              v-model="settings"
              class="bg-white shadow-sm rounded-lg p-4 sm:p-6"
              @select-project="selectProjectDir"
              @select-build-dir="selectBuildDir"
              @select-workspace="selectWorkspaceDir"
              @select-ide="selectCubeIdeExe"
            />
            <BuildControls 
              :status="buildStatus"
              :is-cancelling="isCancelling"
              class="bg-white shadow-sm rounded-lg p-4 sm:p-6"
              @build="build"
              @cancel="cancelBuild"
            />
            <BuildStatus 
              :status="buildStatus"
              :messages="buildMessages"
              :current-stdout="currentStdout"
              class="bg-white shadow-sm rounded-lg p-4 sm:p-6"
              @clear-messages="clearAllMessages"
            />
          </div>
          
          <!-- Logs under left column -->
          <div class="bg-white shadow-sm rounded-lg p-4 sm:p-6">
            <div class="text-sm font-medium text-gray-700 mb-2">Build Logs</div>
            <div class="relative">
              <BuildLogs 
                ref="logContainerRef"
                :logs="buildLogs"
                class="h-[400px] xl:h-[600px] overflow-auto"
              />
            </div>
          </div>
        </div>

        <!-- Right Column -->
        <div class="bg-white shadow-sm rounded-lg p-4 sm:p-6 xl:h-fit">
          <BuildSettings 
            v-model="buildConfig"
            :build-settings="buildSettings ?? { build_settings: [] }"
            :project-path="settings.projectPath ?? ''"
            :build-dir="settings.buildDir ?? ''"
            :workspace-path="settings.workspacePath ?? ''"
            :cube-ide-exe-path="settings.cubeIdeExePath ?? ''"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { formatTimestamp } from './utils/time';
import type { BuildProcessReturn, BuildSettingsConfig } from './types';
import ProjectSettings from './components/ProjectSettings.vue';
import BuildSettings from './components/BuildSettings.vue';
import BuildControls from './components/BuildControls.vue';
import BuildStatus from './components/BuildStatus.vue';
import BuildLogs from './components/BuildLogs.vue';
import { useLocalStorage } from './composables/useLocalStorage';
import { useBuildProcess } from './composables/useBuildProcess';
import { useLogHandler } from './composables/useLogHandler';
import { open } from '@tauri-apps/plugin-dialog';

const logContainerRef = ref<{ logContainer: HTMLElement | null } | null>(null);
const logContainer = ref<HTMLElement | null>(null);
let unsubscribe: (() => void) | null = null;

const { 
  settings,
  buildConfig,
  buildStatus,
  buildMessages,
  buildLogs,
  currentStdout,
  isCancelling,
  build,
  cancelBuild
}: BuildProcessReturn = useBuildProcess();

const selectProjectDir = async () => {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === 'string') {
    settings.value.projectPath = selected;
  }
};

const selectBuildDir = async () => {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === 'string') {
    settings.value.buildDir = selected;
  }
};

const selectWorkspaceDir = async () => {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === 'string') {
    settings.value.workspacePath = selected;
  }
};

const selectCubeIdeExe = async () => {
  const selected = await open({
    filters: [{ name: 'Executable', extensions: ['exe'] }],
    multiple: false
  });
  if (typeof selected === 'string') {
    settings.value.cubeIdeExePath = selected;
  }
};

const buildSettings = ref<BuildSettingsConfig>({ build_settings: [] });

onMounted(async () => {
  // Load build settings
  try {
    buildSettings.value = await invoke<BuildSettingsConfig>('load_build_settings_schema');
    console.log('Loaded build settings:', buildSettings.value);
  } catch (e) {
    console.error('Failed to load build settings:', e);
    buildLogs.value.push(`[${formatTimestamp()}] Failed to load build settings: ${e}`);
  }

  // Set up build-log listener
  unsubscribe = await listen('build-log', (event) => {
    const eventText = String(event.payload);
    if (eventText.startsWith('stdout:')) {
      const stdoutLine = eventText.substring(7).trim();
      currentStdout.value = stdoutLine;
      if (stdoutLine) {
        buildLogs.value.push(`[${formatTimestamp()}] ${stdoutLine}`);
      }
    } else if (eventText.startsWith('stderr:')) {
      const stderrLine = eventText.substring(7).trim();
      if (stderrLine) {
        buildLogs.value.push(`[${formatTimestamp()}] [ERROR] ${stderrLine}`);
      }
    } else {
      buildLogs.value.push(`[${formatTimestamp()}] ${eventText}`);
    }
  });

  if (logContainerRef.value && 'logContainer' in logContainerRef.value) {
    logContainer.value = logContainerRef.value.logContainer;
  }
});

watch(buildLogs, async () => {
  await nextTick();
  if (logContainer.value) {
    const { scrollTop, scrollHeight, clientHeight } = logContainer.value;
    const isAtBottom = scrollTop + clientHeight >= scrollHeight - 50;
    if (isAtBottom) {
      requestAnimationFrame(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
      });
    }
  }
}, { deep: true });

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe();
  }
});

useLogHandler(buildLogs, logContainer);
useLocalStorage('buildParams', settings);
useLocalStorage('buildSettings', settings);
useLocalStorage('buildConfig', buildConfig);

function clearAllMessages() {
  buildMessages.value = [];
  buildLogs.value = [];
}
</script>