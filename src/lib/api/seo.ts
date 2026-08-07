import { invoke } from '@tauri-apps/api/core';
import type { SeoAuditResult, SeoOverview } from './types';

export function getSeoAudit(projectId: string, url: string): Promise<SeoAuditResult> {
  return invoke<SeoAuditResult>('get_seo_audit', { projectId, url });
}

export function runSeoAudit(pageId: string): Promise<SeoAuditResult> {
  return invoke<SeoAuditResult>('run_seo_audit', { pageId });
}

export function getSeoOverview(projectId: string): Promise<SeoOverview> {
  return invoke<SeoOverview>('get_seo_overview', { projectId });
}
