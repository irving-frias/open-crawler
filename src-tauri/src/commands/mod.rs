//! Command registry.
//!
//! The actual `#[tauri::command]` implementations live in
//! `crate::features::<domain>::commands`, grouped by feature. This module
//! re-exports them all so `lib.rs` can register the handler list from a single
//! place.

pub use crate::features::analytics::commands::*;
pub use crate::features::app::commands::*;
pub use crate::features::crawl::commands::*;
pub use crate::features::export::commands::*;
pub use crate::features::pagespeed::commands::*;
pub use crate::features::projects::commands::*;
pub use crate::features::results::commands::*;
pub use crate::features::settings::commands::*;
pub use crate::features::snapshots::commands::*;
