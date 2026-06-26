//! 数据库文件路径工具。
//!
//! 三个数据库的路径：
//! - [`db_path`] — `./mvsep.db`（算法缓存）
//! - [`tasks_db_path`] — `./tasks.db`（任务追踪）
//! - [`user_config_path`] — `./user_config.db`（用户配置）

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

pub fn user_config_path() -> PathBuf {
    data_dir().join("user_config.db")
}

pub fn tasks_db_path() -> PathBuf {
    data_dir().join("tasks.db")
}

pub fn ensure_data_dir() -> anyhow::Result<()> {
    let dir = data_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}
