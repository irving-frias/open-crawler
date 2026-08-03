export interface Project {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface PageLink {
  from_url: string;
  to_url: string;
  config_id: string;
  project_id: string;
  link_type: string;
  anchor_text?: string | null;
  is_follow: boolean;
}

export interface SemanticIssue {
  issue_type: string;
  severity: string;
  element?: string | null;
  message?: string | null;
  css_selector?: string | null;
  xpath?: string | null;
  snippet?: string | null;
  line?: number | null;
  column?: number | null;
}

export interface CrawlResult {
  id: string;
  config_id: string;
  project_id: string;
  url: string;
  status_code?: number | null;
  title?: string | null;
  meta_description?: string | null;
  h1?: string | null;
  canonical?: string | null;
  size_bytes?: number | null;
  load_time_ms?: number | null;
  is_indexable?: boolean | null;
  depth: number;
  parent_url?: string | null;
  crawl_timestamp: string;
  links: PageLink[];
  html_lang?: string | null;
  hreflang_json?: string | null;
  semantic_issues_json?: string | null;
  html_body?: string | null;
  readability_score?: number | null;
  content_hash?: string | null;
  duplicate_group_id?: number | null;
  keywords_json?: string | null;
  og_json?: string | null;
  pagespeed_score?: number | null;
  pagespeed_json?: string | null;
}

export interface CrawlProgress {
  project_id: string;
  urls_crawled: number;
  urls_queued: number;
  current_url: string;
  errors: number;
  elapsed_secs: number;
}

export interface PaginatedResults {
  items: CrawlResult[];
  total: number;
  page: number;
  page_size: number;
}

export interface IssueCount {
  issue_type: string;
  severity: string;
  count: number;
}

export interface SiteTreeNode {
  url: string;
  title: string | null;
  status_code: number | null;
  depth: number;
  has_children: boolean;
  issue_count: number;
}

export interface StatusBucket {
  status: number;
  count: number;
}

export interface DashboardStats {
  total_pages: number;
  indexed_pages: number;
  broken_pages: number;
  avg_load_ms: number;
  avg_size_bytes: number;
  avg_readability?: number | null;
  duplicate_count: number;
  missing_title_count: number;
  missing_description_count: number;
  missing_h1_count: number;
  status_distribution: StatusBucket[];
  top_issues: IssueCount[];
}

export interface CrawlSnapshot {
  id: string;
  project_id: string;
  snapshot_time: string;
  total_pages: number;
  indexed_pages: number;
  broken_pages: number;
  avg_load_ms: number;
  avg_size_bytes: number;
  avg_readability?: number | null;
}

export interface SnapshotStats {
  total_pages: number;
  indexed_pages: number;
  broken_pages: number;
  avg_load_ms: number;
  avg_size_bytes: number;
  avg_readability?: number | null;
}

export interface UrlFieldDiff {
  field: string;
  before?: string | null;
  after?: string | null;
}

export interface ChangedUrl {
  url: string;
  diffs: UrlFieldDiff[];
}

export interface CompareResult {
  new_urls: string[];
  removed_urls: string[];
  changed_urls: ChangedUrl[];
  unchanged_count: number;
  before: SnapshotStats;
  after: SnapshotStats;
}

export interface DuplicateGroupUrl {
  url: string;
  title: string | null;
  status_code: number | null;
}

export interface DuplicateGroup {
  id: number;
  size: number;
  urls: DuplicateGroupUrl[];
}

export interface KeywordAggregate {
  keyword: string;
  count: number;
  pages: number;
}

export interface PageSpeedData {
  score?: number | null;
  fcp?: string | null;
  lcp?: string | null;
  cls?: string | null;
  tbt?: string | null;
  speed_index?: string | null;
  error?: string | null;
}

export interface ProxyConfig {
  url: string;
  username?: string | null;
  password?: string | null;
}

export interface CrawlConfig {
  id?: string | null;
  project_id?: string | null;
  seed_urls: string[];
  max_depth?: number;
  respect_robots?: boolean;
  render_js?: boolean;
  concurrency?: number;
  delay_ms?: number;
  max_body_size?: number;
  same_origin_only?: boolean;
  check_sitemap?: boolean;
  max_crawl_time_secs?: number;
  check_semantics?: boolean;
  include_patterns?: string[];
  exclude_patterns?: string[];
  custom_headers?: [string, string][];
  request_timeout_ms?: number;
  proxy?: ProxyConfig | null;
}

export interface ResumableCrawlInfo {
  session_id: string;
  pages_crawled: number;
  errors: number;
  elapsed_secs: number;
  queue_remaining: number;
}

export type ProjectStats = Record<string, number | string>;
export type SettingsMap = Record<string, string>;

export interface ExportProgress {
  stage: string;
  processed: number;
  total: number;
  percent: number;
}

export interface PageDetail {
  page: CrawlResult;
  links: PageLink[];
}
