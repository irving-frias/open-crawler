import { invoke } from '@tauri-apps/api/core';
import type { ComparePageResult, CrawlSnapshot } from './types';

export function listCrawlSnapshots(projectId: string): Promise<CrawlSnapshot[]> {
  return invoke<CrawlSnapshot[]>('list_crawl_snapshots', { projectId });
}

export function compareCrawlsPage(
  snapshotA: string,
  snapshotB: string,
  section: 'new' | 'removed' | 'changed',
  page: number,
  pageSize: number
): Promise<ComparePageResult> {
  return invoke<ComparePageResult>('compare_crawls_page', {
    snapshotA,
    snapshotB,
    section,
    page,
    pageSize,
  });
}
