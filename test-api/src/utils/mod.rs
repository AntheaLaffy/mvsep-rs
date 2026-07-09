pub mod console;
pub mod paths;

use std::path::PathBuf;

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

pub fn db_path() -> PathBuf {
    data_dir().join("mvsep.db")
}
