import { ref } from 'vue';
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
    settings: {},
    cleanBuild: false,
    projectName: null,
    configName: null,
    customConsoleArgs: null
  });

  const buildStatus = ref<BuildStatusType>('idle');
  const buildMessages = ref<BuildMessage[]>([]);
  const buildLogs = ref<string[]>([]);
  const currentStdout = ref('');
  const isCancelling = ref(false);

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
        project_path: settings.value.projectPath!,
        build_dir: settings.value.buildDir!,
        cube_ide_exe_path: settings.value.cubeIdeExePath!,
        workspace_path: settings.value.workspacePath!,
        project_name: settings.value.projectName || undefined,
        config_name: settings.value.configName || undefined,
        clean_build: buildConfig.value.cleanBuild,
        cancelled: false,
        custom_console_args: buildConfig.value.customConsoleArgs || undefined,
        settings: Object.fromEntries(
          Object.entries(buildConfig.value.settings)
            .filter(([_, value]) => value !== null) // Filter out null values
            .map(([key, value]) => [key, value])
        ),
      };
      
      const result = await executeBuild(config);
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
      isCancelling.value = true;
      await cancelBuild();
      buildStatus.value = 'cancelled';
      isCancelling.value = false;
    }
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