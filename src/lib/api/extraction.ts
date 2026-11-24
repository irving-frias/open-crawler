import { invoke } from '@tauri-apps/api/core';
import type { CustomExtraction } from './types';

export function getCustomExtractions(projectId: string): Promise<CustomExtraction[]> {
  return invoke<CustomExtraction[]>('get_custom_extractions', { projectId });
}

export function saveCustomExtractions(
  projectId: string,
  rules: CustomExtraction[]
): Promise<CustomExtraction[]> {
  return invoke<CustomExtraction[]>('save_custom_extractions', { projectId, rules });
}

export function testCustomExtraction(
  rule: CustomExtraction,
  url: string
): Promise<string | number | null> {
  return invoke<string | number | null>('test_custom_extraction', { rule, url });
}

export function recomputeCustomExtractions(projectId: string): Promise<number> {
  return invoke<number>('recompute_custom_extractions', { projectId });
}
