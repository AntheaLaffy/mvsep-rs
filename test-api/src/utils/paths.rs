//! Database file path utilities
//!
//! Provides path functions for the three databases:
//!
//! - [`db_path`] — `./mvsep.db` (Algorithm cache)
//! - [`tasks_db_path`] — `./tasks.db` (Task tracking)
//! - [`user_config_path`] — `./user_config.db` (User configuration)
//!
//! The data directory depends on the operating system:
//! - Unix: Current directory `.`
//! - Windows: `%APPDATA%\mvsep-tester`

use std::path::PathBuf;

/// Get the data directory path
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
/// # Returns
///
/// `PathBuf` - Main database file path (`mvsep.db`)
pub fn db_path() -> PathBuf {
    data_dir().join("mvsep.db")
}

/// Get the user configuration database path
///
/// # Returns
///
/// `PathBuf` - User configuration database file path (`user_config.db`)
pub fn user_config_path() -> PathBuf {
    data_dir().join("user_config.db")
}

/// Get the task database path
///
/// # Returns
///
/// `PathBuf` - Task database file path (`tasks.db`)
pub fn tasks_db_path() -> PathBuf {
    data_dir().join("tasks.db")
}

/// Ensure the data directory exists
///
/// Creates the data directory if it does not exist.
///
/// # Returns
///
/// `anyhow::Result<()>` - Success or error
pub fn ensure_data_dir() -> anyhow::Result<()> {
    let dir = data_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}
