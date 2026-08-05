//! Portable app-data packages (`.ocproj`) for sharing projects between devices.
//!
//! A package is a ZIP containing a clean SQLite snapshot of the app database
//! (`open-crawler.db`) plus a `manifest.json` with metadata and a SHA-256
//! checksum. Export prunes unselected projects and can strip secrets/HTML;
//! import validates and merges projects back into the local database.

pub mod commands;
pub mod desktop_share;
pub mod obex;
pub mod package;
pub mod server;
