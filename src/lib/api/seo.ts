import { invoke } from '@tauri-apps/api/core';
import type { FixSuggestion, SeoAuditResult, SeoOverview, SeoAuditProgress } from './types';

export function getSeoAudit(projectId: string, url: string): Promise<SeoAuditResult> {
  return invoke<SeoAuditResult>('get_seo_audit', { projectId, url });
}

export function runSeoAudit(pageId: string): Promise<SeoAuditResult> {
  return invoke<SeoAuditResult>('run_seo_audit', { pageId });
}

export function getSeoOverview(projectId: string): Promise<SeoOverview> {
  return invoke<SeoOverview>('get_seo_overview', { projectId });
}

export function runSeoAuditAll(projectId: string): Promise<SeoAuditProgress> {
  return invoke<SeoAuditProgress>('run_seo_audit_all', { projectId });
}

export function getSeoAuditStatus(projectId: string): Promise<SeoAuditProgress | null> {
  return invoke<SeoAuditProgress | null>('get_seo_audit_status', { projectId });
}

export function stopSeoAudit(projectId: string): Promise<void> {
  return invoke<void>('stop_seo_audit', { projectId });
}

export interface SuggestFixPayload {
  checkId: string;
  checkMessage: string;
  checkGuidance: string;
  elementSnippet?: string | null;
  language: string;
}

export function suggestFix(payload: SuggestFixPayload): Promise<FixSuggestion> {
  return invoke<FixSuggestion>('suggest_fix', { ...payload });
}
