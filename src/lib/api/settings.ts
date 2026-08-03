import { invoke } from '@tauri-apps/api/core';
import type { SettingsMap } from './types';

export function getSettings(): Promise<SettingsMap> {
  return invoke<SettingsMap>('get_settings');
}

export function saveSettings(settings: SettingsMap): Promise<void> {
  return invoke<void>('save_settings', { settings });
}
