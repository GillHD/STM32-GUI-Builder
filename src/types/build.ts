import type { Ref } from 'vue';

export type BuildStatus = 'idle' | 'building' | 'success' | 'error' | 'cancelled';

export interface BuildSettings {
  projectPath: string | null;
  buildDir: string | null;
  cubeIdeExePath: string | null;
  workspacePath: string | null;
  projectName: string | null;
  configName: string | null;
}

export interface BuildSettingOption {
  value: string;
  label: string;
  define?: string;
}

export type BuildFieldType = 'range' | 'select' | 'checkbox_group' | 'text' | 'number';

export interface BuildSettingBase {
  id: string;
  name: string;
  value: string; // Added this field for file naming
  field_type: BuildFieldType;
  description: string;
  define?: string;
  options?: BuildSettingOption[];
  validation?: {
    min: number;
    max: number;
    format: string;
  };
  exclusive?: boolean;
  min_selected?: number;
}

export type BuildSetting = BuildSettingBase;

export interface BuildSettingsConfig {
  version: string;
  build_settings: BuildSetting[];
}

export interface BuildConfig {
  project_path: string | null;
  build_dir: string | null;
  cube_ide_exe_path: string | null;
  workspace_path: string | null;
  project_name: string | null;
  config_name: string | null;
  clean_build: boolean;
  cancelled: boolean;
  custom_console_args?: string | null;
  settings: { [key: string]: string | string[] | number[] };
}

export interface LocalBuildConfig {
  settings: { [key: string]: string | string[] };
  cleanBuild: boolean;
  projectName?: string | null;
  configName?: string | null;
  customConsoleArgs?: string | null;
}

export interface BuildMessage {
  type: 'success' | 'error';
  text: string;
}

export interface BuildProcessReturn {
  settings: Ref<BuildSettings>;
  buildConfig: Ref<LocalBuildConfig>;
  buildStatus: Ref<BuildStatus>;
  buildMessages: Ref<BuildMessage[]>;
  buildLogs: Ref<string[]>;
  currentStdout: Ref<string>;
  isCancelling: Ref<boolean>;
  build: () => Promise<void>;
  cancelBuild: () => Promise<void>;
}