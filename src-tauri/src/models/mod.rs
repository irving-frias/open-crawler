pub mod crawl_config;
pub mod crawl_result;
pub mod project;

pub use crawl_config::{CrawlConfig, ProxyConfig, SiteAuth};
pub use crawl_result::{
    ChangedUrl, CompareResult, CrawlProgress, CrawlResult, CrawlSnapshot, DashboardStats,
    DuplicateGroup, DuplicateGroupUrl, IssueCount, KeywordAggregate, PageDetail, PageLink,
    PaginatedResults, ResultsFilter, SiteTreeFullNode, SiteTreeNode, SnapshotStats, StatusBucket,
    UrlFieldDiff,
};
pub use project::{CreateProjectRequest, Project, RenameProjectRequest};
