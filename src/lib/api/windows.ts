import { invoke } from '@tauri-apps/api/core';

export function openProjectWindow(projectId: string, title: string): Promise<void> {
  return invoke<void>('open_project_window', { projectId, title });
}

export function closeProjectWindow(projectId: string): Promise<void> {
  return invoke<void>('close_project_window', { projectId });
}

export function listOpenProjectWindows(): Promise<string[]> {
  return invoke<string[]>('list_open_project_windows');
}

export function isProjectWindow(): Promise<boolean> {
  return invoke<boolean>('is_project_window');
}
