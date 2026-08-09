import { invoke } from '@tauri-apps/api/core';
import type {
  CrawlResult,
  IssueCount,
  PageDetail,
  PaginatedResults,
  SiteGraph,
  SiteGraphEdgePage,
  SiteTreeFullNode,
  SiteTreeNode,
} from './types';

export interface GetResultsArgs {
  projectId: string;
  page: number;
  pageSize: number;
  semanticIssueType?: string | null;
  search?: string | null;
  statusFilter?: number[] | null;
  severityFilter?: string[] | null;
  domainFilter?: string | null;
  depthFilter?: number | null;
  missingTitle?: boolean | null;
  duplicateTitle?: boolean | null;
  noindexOnly?: boolean | null;
  is404?: boolean | null;
}

export function getResults(args: GetResultsArgs): Promise<PaginatedResults> {
  return invoke<PaginatedResults>('get_results', {
    projectId: args.projectId,
    page: args.page,
    pageSize: args.pageSize,
    semanticIssueType: args.semanticIssueType ?? null,
    search: args.search ?? null,
    statusFilter: args.statusFilter ?? null,
    severityFilter: args.severityFilter ?? null,
    domainFilter: args.domainFilter ?? null,
    depthFilter: args.depthFilter ?? null,
    missingTitle: args.missingTitle ?? null,
    duplicateTitle: args.duplicateTitle ?? null,
    noindexOnly: args.noindexOnly ?? null,
    is404: args.is404 ?? null,
  });
}

export function getPageDetail(pageId: string): Promise<PageDetail> {
  return invoke<PageDetail>('get_page_detail', { pageId });
}

export function recrawlPage(pageId: string): Promise<CrawlResult> {
  return invoke<CrawlResult>('recrawl_page', { pageId });
}

export function getSemanticIssueCounts(projectId: string): Promise<IssueCount[]> {
  return invoke<IssueCount[]>('get_semantic_issue_counts', { projectId });
}

export function getSiteTree(
  projectId: string,
  url?: string | null,
  limit?: number | null
): Promise<SiteTreeNode[]> {
  return invoke<SiteTreeNode[]>('get_site_tree', {
    projectId,
    url: url ?? null,
    limit: limit ?? null,
  });
}

export function getSiteTreeFull(projectId: string): Promise<SiteTreeFullNode[]> {
  return invoke<SiteTreeFullNode[]>('get_site_tree_full', { projectId });
}

export function getSiteGraph(projectId: string): Promise<SiteGraph> {
  return invoke<SiteGraph>('get_site_graph', { projectId });
}

export function getSiteGraphEdges(
  projectId: string,
  offset: number,
  limit: number
): Promise<SiteGraphEdgePage> {
  return invoke<SiteGraphEdgePage>('get_site_graph_edges', { projectId, offset, limit });
}
