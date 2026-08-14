import { invoke } from '@tauri-apps/api/core';
import type { LinkAnalysis } from './types';

export function getLinkAnalysis(projectId: string): Promise<LinkAnalysis> {
  return invoke<LinkAnalysis>('get_link_analysis', { projectId });
}

export function getProjectHasLinks(projectId: string): Promise<boolean> {
  return invoke<boolean>('get_project_has_links', { projectId });
}
