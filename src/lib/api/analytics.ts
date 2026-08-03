import { invoke } from '@tauri-apps/api/core';
import type { DashboardStats, DuplicateGroup, KeywordAggregate } from './types';

export function getDashboardStats(projectId: string): Promise<DashboardStats> {
  return invoke<DashboardStats>('get_dashboard_stats', { projectId });
}

export function getDuplicateGroups(projectId: string): Promise<DuplicateGroup[]> {
  return invoke<DuplicateGroup[]>('get_duplicate_groups', { projectId });
}

export function getProjectKeywords(projectId: string, limit?: number | null): Promise<KeywordAggregate[]> {
  return invoke<KeywordAggregate[]>('get_project_keywords', { projectId, limit: limit ?? null });
}
