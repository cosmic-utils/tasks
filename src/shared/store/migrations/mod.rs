//! Migration module for converting from old storage format to new storage format
//!
//! This module provides functionality to migrate from the old task storage system
//! (with nested sub-tasks in directories) to the new flat storage system
//! (with parent_id references).
//!
//! # Old Format
//! - Lists stored as individual `.ron` files in `{base_dir}/lists/`
//! - Tasks stored as `.ron` files in `{base_dir}/tasks/{list_id}/` with nested sub-task directories
//! - List IDs and Task IDs were String-based UUIDs
//! - Sub-tasks stored in the parent task's `sub_tasks` Vec
//!
//! # New Format
//! - Lists stored in a single registry file `{base_dir}/lists.ron`
//! - Tasks stored as individual `.ron` files in `{base_dir}/{list_id}/`
//! - List IDs and Task IDs are proper Uuid types
//! - Sub-tasks are separate task files with `parent_id` references
//!
//! # Usage
//!
//! ```no_run
//! use tasks::shared::store::migrations::run_migration;
//! use tasks::shared::store::Store;
//!
//! let old_dir = "/path/to/old/storage";
//! let new_dir = "/path/to/new/storage";
//!
//! match run_migration(old_dir, new_dir) {
//!     Ok(report) => {
//!         tracing::info!("Migration successful!");
//!         tracing::info!("Lists migrated: {}", report.lists_migrated);
//!         tracing::info!("Tasks migrated: {}", report.tasks_migrated);
//!     }
//!     Err(e) => tracing::error!("Migration failed: {}", e),
//! }
//! ```

mod migrate;
mod models;

pub use migrate::{MigrationReport, Migrator};

use crate::shared::store::Store;
use crate::{Error, Result};
use std::path::Path;

pub fn run_migration(
    old_base_dir: impl AsRef<Path>,
    new_base_dir: impl AsRef<Path>,
) -> Result<MigrationReport> {
    let store = Store::open(new_base_dir)?;
    let migrator = Migrator::new(old_base_dir, store);
    migrator.migrate()
}

pub fn needs_migration(old_base_dir: impl AsRef<Path>) -> bool {
    let marker_file = old_base_dir.as_ref().join("migrated");
    !marker_file.exists()
}

pub fn migrate(old_base_dir: std::path::PathBuf, new_base_dir: &std::path::Path) -> Result<()> {
    if needs_migration(&old_base_dir) {
        match run_migration(&old_base_dir, &new_base_dir) {
            Ok(report) => {
                if !report.errors.is_empty() {
                    tracing::error!("\n⚠ Errors encountered ({}):", report.errors.len());
                    for error in &report.errors {
                        tracing::error!("    - {}", error);
                    }
                    return Err(Error::MigrationFailed(format!(
                        "{} errors during migration",
                        report.errors.len()
                    )));
                }
            }
            Err(err) => {
                tracing::error!("Migration failed: {}", err);
                return Err(Error::MigrationFailed(err.to_string()));
            }
        }
    }
    Ok(())
}
