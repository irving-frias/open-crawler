import { invoke } from '@tauri-apps/api/core';
import type { AnchorAgg, DomainAgg, LinkAnalysis } from './types';

export function getLinkAnalysis(projectId: string): Promise<LinkAnalysis> {
  return invoke<LinkAnalysis>('get_link_analysis', { projectId });
}

export function getProjectHasLinks(projectId: string): Promise<boolean> {
  return invoke<boolean>('get_project_has_links', { projectId });
}

export function getOrphanPagesPage(
  projectId: string,
  page: number,
  pageSize: number
): Promise<[string[], number]> {
  return invoke<[string[], number]>('get_orphan_pages_page', { projectId, page, pageSize });
}

export function getDeadEndPagesPage(
  projectId: string,
  page: number,
  pageSize: number
): Promise<[string[], number]> {
  return invoke<[string[], number]>('get_dead_end_pages_page', { projectId, page, pageSize });
}

export function getTopAnchorsPage(
  projectId: string,
  page: number,
  pageSize: number
): Promise<[AnchorAgg[], number]> {
  return invoke<[AnchorAgg[], number]>('get_top_anchors_page', { projectId, page, pageSize });
}

export function getExternalDomainsPage(
  projectId: string,
  page: number,
  pageSize: number
): Promise<[DomainAgg[], number]> {
  return invoke<[DomainAgg[], number]>('get_external_domains_page', {
    projectId,
    page,
    pageSize,
  });
}
