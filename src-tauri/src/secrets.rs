use crate::db::CrawlRepo;
use crate::error::AppError;

/// Service name used for OS keychain entries.
const KEYRING_SERVICE: &str = "open-crawler";

/// Keys that hold sensitive API credentials. Kept in the OS keychain on
/// desktop; on mobile (no keyring backend) they live in the settings table.
pub const SECRET_KEYS: [&str; 2] = ["pagespeed_api_key", "ai_api_key"];

pub fn is_secret_key(key: &str) -> bool {
    SECRET_KEYS.contains(&key)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn keyring_get(key: &str) -> Result<Option<String>, AppError> {
    match keyring::Entry::new(KEYRING_SERVICE, key) {
        Ok(entry) => match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Crawl(format!("keyring read failed: {e}"))),
        },
        Err(e) => Err(AppError::Crawl(format!("keyring init failed: {e}"))),
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn keyring_set(key: &str, value: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| AppError::Crawl(format!("keyring init failed: {e}")))?;
    entry
        .set_password(value)
        .map_err(|e| AppError::Crawl(format!("keyring write failed: {e}")))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn keyring_delete(key: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| AppError::Crawl(format!("keyring init failed: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Crawl(format!("keyring delete failed: {e}"))),
    }
}

/// Reads a secret. On desktop the OS keychain is the source of truth, falling
/// back to the settings table for values written before keyring support (or
/// when no keyring backend is available). On mobile the settings table is used.
pub fn get(repo: &CrawlRepo, key: &str) -> Result<Option<String>, AppError> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        match keyring_get(key) {
            Ok(Some(_)) => keyring_get(key),
            Ok(None) => repo.get_setting(key),
            Err(_) => repo.get_setting(key),
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        repo.get_setting(key)
    }
}

/// Writes a secret. On desktop it goes to the OS keychain and the plain-text
/// settings row is cleared; on mobile (no keyring backend) it is stored in the
/// settings table.
pub fn set(repo: &CrawlRepo, key: &str, value: &str) -> Result<(), AppError> {
    if value.is_empty() {
        return delete(repo, key);
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        match keyring_set(key, value) {
            Ok(()) => {
                let _ = repo.set_setting(key, "");
                Ok(())
            }
            Err(_) => repo.set_setting(key, value),
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        repo.set_setting(key, value)
    }
}

/// Deletes a secret from the keyring (desktop) and clears the settings row.
pub fn delete(repo: &CrawlRepo, key: &str) -> Result<(), AppError> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let _ = keyring_delete(key);
    }
    repo.set_setting(key, "")
}

/// True when a secret exists (in the keyring or the settings table).
pub fn has(repo: &CrawlRepo, key: &str) -> Result<bool, AppError> {
    Ok(get(repo, key)?.map(|v| !v.is_empty()).unwrap_or(false))
}
