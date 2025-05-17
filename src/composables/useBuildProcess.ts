import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { BuildMessage } from '../types/build';
import type { BuildConfig, BuildStatusType, BuildProcessReturn, LocalBuildConfig, Settings } from '../types/index';
import { executeBuild, cancelBuild } from '../services/buildService';

export function useBuildProcess(): BuildProcessReturn {
  const settings = ref<Settings>({
    projectPath: null,
    buildDir: null,
    cubeIdeExePath: null,
    workspacePath: null,
    projectName: null,
    configName: null,
  });

  const buildConfig = ref<LocalBuildConfig>({
    projectPath: '',
    buildDir: '',
    workspacePath: '',
    cubeIdeExePath: '',
    settings: {},
    cleanBuild: false,
    projectName: null,
    configName: null,
    customConsoleArgs: null,
    cancelled: false
  });

  const buildStatus = ref<BuildStatusType>('idle');
  const buildMessages = ref<BuildMessage[]>([]);
  const buildLogs = ref<string[]>([]);
  const currentStdout = ref('');
  const isCancelling = ref(false);

  // Only one listener should be active at a time
  let unlistenCancel: (() => void) | null = null;

  async function build() {
    // Validate required fields
    const requiredFields: Array<keyof Settings> = ['projectPath', 'buildDir', 'cubeIdeExePath', 'workspacePath'];
    const missingFields = requiredFields.filter(field => !settings.value[field]);
    
    if (missingFields.length > 0) {
      buildStatus.value = 'error';
      buildMessages.value.push({
        type: 'error',
        text: `Missing required fields: ${missingFields.join(', ')}`,
      });
      return;
    }

    buildStatus.value = 'building';
    buildMessages.value = [];

    try {
      const config: BuildConfig = {
        projectPath: settings.value.projectPath!,
        buildDir: settings.value.buildDir!,
        cubeIdeExePath: settings.value.cubeIdeExePath!,
        workspacePath: settings.value.workspacePath!,
        projectName: settings.value.projectName || undefined,
        configName: settings.value.configName || undefined,
        cleanBuild: buildConfig.value.cleanBuild,
        cancelled: false,
        customConsoleArgs: buildConfig.value.customConsoleArgs || undefined,
        settings: Object.fromEntries(
          Object.entries(buildConfig.value.settings)
            .filter(([_, value]) => value !== null)
            .map(([key, value]) => [key, value])
        ),
      };
      
      const result = await executeBuild(config);
      // Если сборка была отменена через invoke (build_project вернул cancel)
      if (
        typeof result.result === 'string' &&
        result.result.toLowerCase().includes('cancelled')
      ) {
        buildStatus.value = 'idle';
        buildMessages.value.push({
          type: 'success',
          text: 'Build cancelled by user'
        });
        isCancelling.value = false;
        return;
      }
      buildStatus.value = result.success ? 'success' : 'error';
      
      buildMessages.value.push({
        type: result.success ? 'success' : 'error',
        text: result.result,
      });
    } catch (error) {
      buildStatus.value = 'error';
      buildMessages.value.push({
        type: 'error',
        text: `Build error: ${error instanceof Error ? error.message : String(error)}`,
      });
    }
  }

  async function handleCancel() {
    if (!isCancelling.value && buildStatus.value === 'building') {
      try {
        console.log('[DEBUG] Cancel initiated');
        isCancelling.value = true;

        // Set up listener first
        const unlisten = await listen<boolean>('build-cancelled', () => {
          console.log('[DEBUG] Received build-cancelled event');
          resetAfterCancel();
          unlisten();
        });

        console.log('[DEBUG] Cancel listener set up, sending cancel command');
        
        // Send cancel command
        await cancelBuild();
        console.log('[DEBUG] Cancel command sent');

        // Fallback timer
        const timeoutId = setTimeout(() => {
          console.log('[DEBUG] Cancel timeout triggered');
          if (isCancelling.value) {
            console.log('[DEBUG] Forcing cancel reset');
            resetAfterCancel();
            unlisten();
          }
        }, 1500);

        // Clean up timeout on event
        listen('build-cancelled', () => {
          console.log('[DEBUG] Clearing timeout');
          clearTimeout(timeoutId);
        });

      } catch (error) {
        console.error('[DEBUG] Cancel error:', error);
        resetAfterCancel();
      }
    }
  }

  function resetAfterCancel() {
    console.log('[DEBUG] Resetting state after cancel');
    if (!isCancelling.value) {
      console.log('[DEBUG] Cancel already reset, skipping');
      return;
    }
    
    buildStatus.value = 'idle';
    currentStdout.value = '';
    isCancelling.value = false;
    
    buildMessages.value = [{
      type: 'success',
      text: 'Build cancelled by user'
    }];
    
    console.log('[DEBUG] Cancel reset complete');
  }

  return {
    settings,
    buildConfig,
    buildStatus,
    buildMessages,
    buildLogs,
    currentStdout,
    isCancelling,
    build,
    cancelBuild: handleCancel,
  };
}