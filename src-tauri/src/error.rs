use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Crawl error: {0}")]
    Crawl(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Engine not initialized")]
    EngineNotInitialized,

    #[error("Engine already running")]
    EngineAlreadyRunning,

    #[error("PageSpeed API error: {0}")]
    Pagespeed(String),

    #[error("Archive error: {0}")]
    Archive(#[from] zip::result::ZipError),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
