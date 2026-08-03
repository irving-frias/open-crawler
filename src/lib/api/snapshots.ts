import { invoke } from '@tauri-apps/api/core';
import type { CompareResult, CrawlSnapshot } from './types';

export function listCrawlSnapshots(projectId: string): Promise<CrawlSnapshot[]> {
  return invoke<CrawlSnapshot[]>('list_crawl_snapshots', { projectId });
}

export function compareCrawls(snapshotA: string, snapshotB: string): Promise<CompareResult> {
  return invoke<CompareResult>('compare_crawls', { snapshotA, snapshotB });
}
