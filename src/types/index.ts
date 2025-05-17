import { Ref } from 'vue';

export interface BuildConfig {
  projectPath: string;
  buildDir: string;
  cubeIdeExePath: string;
  workspacePath: string;
  projectName?: string;
  configName?: string;
  cleanBuild: boolean;
  cancelled: boolean;
  customConsoleArgs?: string;
  settings: Record<string, any>;
}

export interface BuildSettingsConfig {
  build_settings: Array<{
    id: string;
    label: string;
    value: string; // Added this field for file naming
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
  externalSettingsPath?: string | null;
}

export interface BuildMessage {
  type: 'success' | 'error' | 'info';  // Add 'info' type
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
  projectPath: string;
  buildDir: string;
  workspacePath: string;
  cubeIdeExePath: string;
  projectName?: string | null;
  configName?: string | null;
  cleanBuild: boolean;
  customConsoleArgs?: string | null;
  settings: Record<string, string | string[]>;
  cancelled?: boolean;
}

export interface BuildResult {
  result: string;
  logs: string[];
  stages: string[];
  success: boolean;
}

export type BuildStatusType = 'idle' | 'building' | 'success' | 'error' | 'cancelled';