import { invoke } from '@tauri-apps/api/core';
import type { AlertSettings, TestAlertResult } from './types';

export function getAlertSettings(projectId: string): Promise<AlertSettings> {
  return invoke<AlertSettings>('get_alert_settings', { projectId });
}

export function saveAlertSettings(settings: AlertSettings): Promise<AlertSettings> {
  return invoke<AlertSettings>('save_alert_settings', { settings });
}

export function testAlertNotification(projectId: string | null): Promise<TestAlertResult> {
  return invoke<TestAlertResult>('test_alert_notification', { projectId });
}
