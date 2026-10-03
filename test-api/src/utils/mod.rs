//! Utility module
//!
//! Provides common utilities for paths and console output.

pub mod console;
pub mod paths;

use std::path::PathBuf;

/// Get the data directory path
///
/// Returns the data directory for the operating system:
/// - Unix: Current directory `.`
/// - Windows: `%APPDATA%\mvsep-tester`
///
/// # Returns
///
/// `PathBuf` - Data directory path
pub fn data_dir() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from(".")
    }

    #[cfg(windows)]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mvsep-tester")
    }
}

/// Get the main database path
///
/// The default path is `mvsep.db` in the data directory.
///
/// # Returns
///
/// `PathBuf` - Database file path
pub fn db_path() -> PathBuf {
    data_dir().join("mvsep.db")
}
