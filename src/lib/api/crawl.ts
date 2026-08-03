import { invoke } from '@tauri-apps/api/core';
import type { CrawlConfig, CrawlProgress, ResumableCrawlInfo } from './types';

export function startCrawl(projectId: string, config: CrawlConfig): Promise<void> {
  return invoke<void>('start_crawl', { projectId, config });
}

export function stopCrawl(projectId: string): Promise<void> {
  return invoke<void>('stop_crawl', { projectId });
}

export function getCrawlStatus(projectId: string): Promise<CrawlProgress | null> {
  return invoke<CrawlProgress | null>('get_crawl_status', { projectId });
}

export function getRunningCrawls(): Promise<string[]> {
  return invoke<string[]>('get_running_crawls');
}

export function checkResumableCrawl(projectId: string): Promise<ResumableCrawlInfo | null> {
  return invoke<ResumableCrawlInfo | null>('check_resumable_crawl', { projectId });
}

export function getLastCrawlConfig(projectId: string): Promise<CrawlConfig | null> {
  return invoke<CrawlConfig | null>('get_last_crawl_config', { projectId });
}

export function isMobile(): Promise<boolean> {
  return invoke<boolean>('is_mobile');
}

export function getFavicon(url: string): Promise<string | null> {
  return invoke<string | null>('get_favicon', { url });
}
