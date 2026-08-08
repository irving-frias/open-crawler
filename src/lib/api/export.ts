import { invoke } from '@tauri-apps/api/core';

export function exportFull(
  projectId: string,
  filePath: string,
  format: 'xlsx' | 'csv'
): Promise<void> {
  return invoke<void>('export_full', { projectId, filePath, format });
}
