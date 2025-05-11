import { Ref } from 'vue';

export interface BuildConfig {
  project_path: string;
  build_dir: string;
  cube_ide_exe_path: string;
  workspace_path: string;
  project_name?: string;
  config_name?: string;
  clean_build: boolean;
  cancelled: boolean;
  custom_console_args?: string;
  settings: Record<string, any>;
}

export interface BuildSettingsConfig {
  build_settings: Array<{
    id: string;
    label: string;
    field_type: 'range' | 'select' | 'checkbox_group';
    description?: string;
    format?: string;
    validation?: { min: number; max: number };
    options?: Array<{
      label: string;
      value: string;
      define?: string;
      description?: string;
    }>;
    min_selected?: number;
  }>;
}

export interface Settings {
  projectPath: string | null;
  buildDir: string | null;
  cubeIdeExePath: string | null;
  workspacePath: string | null;
  projectName: string | null;
  configName: string | null;
}

export interface BuildMessage {
  type: 'success' | 'error';
  text: string;
}

export interface BuildProcessReturn {
  settings: Ref<Settings>;
  buildConfig: Ref<LocalBuildConfig>;  // Changed from BuildConfig to LocalBuildConfig
  buildStatus: Ref<BuildStatusType>;
  buildMessages: Ref<BuildMessage[]>;  // Changed from string[] to BuildMessage[]
  buildLogs: Ref<string[]>;
  currentStdout: Ref<string>;
  isCancelling: Ref<boolean>;
  build: () => Promise<void>;
  cancelBuild: () => Promise<void>;
}

export interface LocalBuildConfig {
  settings: Record<string, string | string[]>;
  cleanBuild: boolean;
  projectName: string | null;
  configName: string | null;
  customConsoleArgs: string | null;
}

export interface BuildResult {
  result: string;
  logs: string[];
  stages: string[];
  success: boolean;
}

export type BuildStatusType = 'idle' | 'building' | 'success' | 'error' | 'cancelled';