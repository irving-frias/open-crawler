use serde::{Deserialize, Serialize};

/// Aggregate link metrics for a project, computed from `page_links`.
///
/// Everything is derived with plain SQL over the enriched link rows
/// (`rel_tokens`, `is_sponsored`, `is_ugc`, `is_internal`), so the panel does
/// not need the raw link graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkAnalysis {
    pub total_links: usize,
    pub internal_links: usize,
    pub external_links: usize,
    pub self_links: usize,
    pub followed_links: usize,
    pub nofollow_links: usize,
    pub sponsored_links: usize,
    pub ugc_links: usize,
    pub unique_internal_targets: usize,
    pub internal_pages: usize,
    pub orphan_count: usize,
    pub orphan_pages: Vec<String>,
    pub dead_end_count: usize,
    pub dead_end_pages: Vec<String>,
    pub top_anchors: Vec<AnchorAgg>,
    pub anchor_quality: AnchorQuality,
    pub external_domains: Vec<DomainAgg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorAgg {
    pub anchor: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgg {
    pub domain: String,
    pub count: usize,
    pub nofollow: usize,
    pub sponsored: usize,
    pub ugc: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorQuality {
    pub descriptive: usize,
    pub generic: usize,
    pub url_anchors: usize,
    pub empty: usize,
}
