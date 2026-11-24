import { invoke } from '@tauri-apps/api/core';

export interface SitemapExport {
  files: string[];
  total_urls: number;
}

export function exportFull(
  projectId: string,
  filePath: string,
  format: 'xlsx' | 'csv'
): Promise<void> {
  return invoke<void>('export_full', { projectId, filePath, format });
}

export function exportSitemap(projectId: string, filePath: string): Promise<SitemapExport> {
  return invoke<SitemapExport>('export_sitemap', { projectId, filePath });
}
