pub mod crawl_config;
pub mod crawl_result;
pub mod link_analysis;
pub mod project;
pub mod schedule;
pub mod seo;

pub use crawl_config::{CrawlConfig, ProxyConfig, SiteAuth};
pub use crawl_result::{
    ChangedUrl, ComparePageResult, CompareResult, CrawlProgress, CrawlResult, CrawlSnapshot,
    DashboardStats, DuplicateGroup, DuplicateGroupUrl, IssueCount, KeywordAggregate, PageDetail,
    PageLink, PaginatedResults, RedirectHop, RedirectRecord, ResultsFilter, SiteTreeFullNode,
    SiteTreeNode, SiteTreeStreamNode, SnapshotStats, StatusBucket, UrlFieldDiff,
};
pub use link_analysis::{AnchorAgg, AnchorQuality, DomainAgg, LinkAnalysis};
pub use project::{CreateProjectRequest, Project, RenameProjectRequest};
pub use schedule::{CreateScheduledJobRequest, ScheduledJob, UpdateScheduledJobRequest};
pub use seo::{GradeCount, SeoCategoryAvg, SeoIssueCount, SeoOverview};
