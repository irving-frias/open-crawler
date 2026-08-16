import { invoke } from '@tauri-apps/api/core';
import type { DashboardStats, DuplicateGroup, KeywordAggregate } from './types';

export function getDashboardStats(projectId: string): Promise<DashboardStats> {
  return invoke<DashboardStats>('get_dashboard_stats', { projectId });
}

export function getDuplicateGroups(projectId: string): Promise<DuplicateGroup[]> {
  return invoke<DuplicateGroup[]>('get_duplicate_groups', { projectId });
}

export function getDuplicateGroupsPage(
  projectId: string,
  page: number,
  pageSize: number
): Promise<[DuplicateGroup[], number]> {
  return invoke<[DuplicateGroup[], number]>('get_duplicate_groups_page', {
    projectId,
    page,
    pageSize,
  });
}

export function getProjectKeywords(
  projectId: string,
  limit?: number | null
): Promise<KeywordAggregate[]> {
  return invoke<KeywordAggregate[]>('get_project_keywords', { projectId, limit: limit ?? null });
}

export function getProjectKeywordsPage(
  projectId: string,
  page: number,
  pageSize: number
): Promise<[KeywordAggregate[], number]> {
  return invoke<[KeywordAggregate[], number]>('get_project_keywords_page', {
    projectId,
    page,
    pageSize,
  });
}
