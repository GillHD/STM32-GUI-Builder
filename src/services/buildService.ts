import { invoke } from '@tauri-apps/api/core';
import type { BuildResult, BuildConfig } from '../types/index';

export async function executeBuild(config: BuildConfig): Promise<BuildResult> {
  return await invoke<BuildResult>('build_project', { config });
}

export async function cancelBuild(): Promise<void> {
  await invoke('cancel_build');
}

export async function loadBuildSettings(): Promise<any> {
  try {
    const schema = await invoke<any>('load_build_settings_schema');
    
    // Validate schema format
    if (!schema.build_settings?.every((setting: any) => 
      setting.format && 
      typeof setting.format === 'string' &&
      ['number', 'string', 'string[]'].includes(setting.format)
    )) {
      throw new Error('Invalid schema format. Each setting must have a valid format field (number/string/string[])');
    }
    
    return schema;
  } catch (error) {
    throw new Error(`Failed to load build settings: ${error}`);
  }
}

export function formatTimestamp(): string {
  const now = new Date();
  return `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;
}
