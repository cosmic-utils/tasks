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
//! use tasks::migrations::run_migration;
//! use tasks::services::store::Store;
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

use crate::services::store::Store;
use crate::{Error, Result};
use std::path::Path;

/// Run the migration from old storage format to new storage format
///
/// This function:
/// 1. Opens the new store at `new_base_dir`
/// 2. Reads the old storage from `old_base_dir`
/// 3. Converts all lists and tasks to the new format
/// 4. Flattens nested sub-tasks into separate task files with parent_id references
///
/// # Arguments
///
/// * `old_base_dir` - Path to the old storage directory (contains `lists/` and `tasks/` subdirectories)
/// * `new_base_dir` - Path to the new storage directory (will contain `lists.ron` and `{list_id}/` directories)
///
/// # Returns
///
/// A `MigrationReport` containing statistics about the migration
///
/// # Errors
///
/// Returns an error if:
/// - The new store cannot be opened
/// - Files cannot be read or parsed
/// - Files cannot be written to the new location
pub fn run_migration(
    old_base_dir: impl AsRef<Path>,
    new_base_dir: impl AsRef<Path>,
) -> Result<MigrationReport> {
    let store = Store::open(new_base_dir)?;
    let migrator = Migrator::new(old_base_dir, store);
    migrator.migrate()
}

/// Check if migration is needed by looking for old storage structure
///
/// Returns `true` if the old storage format is detected (lists/ and tasks/ directories exist)
pub fn needs_migration(old_base_dir: impl AsRef<Path>) -> bool {
    // Check if migration has already been completed
    let marker_file = old_base_dir.as_ref().join("migrated");
    !marker_file.exists()
}

/// Perform the migration if needed, returning an error if migration fails
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
