//! 数据库文件路径工具
//!
//! 提供三个数据库的路径获取函数：
//!
//! - [`db_path`] — `./mvsep.db`（算法缓存）
//! - [`tasks_db_path`] — `./tasks.db`（任务追踪）
//! - [`user_config_path`] — `./user_config.db`（用户配置）
//!
//! 数据目录根据操作系统不同：
//! - Unix: 当前目录 `.`
//! - Windows: `%APPDATA%\mvsep-tester`

use std::path::PathBuf;

/// 获取数据目录路径
///
/// # 返回
///
/// `PathBuf` - 数据目录路径
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

/// 获取主数据库路径
///
/// # 返回
///
/// `PathBuf` - 主数据库文件路径（`mvsep.db`）
pub fn db_path() -> PathBuf {
    data_dir().join("mvsep.db")
}

/// 获取用户配置数据库路径
///
/// # 返回
///
/// `PathBuf` - 用户配置数据库文件路径（`user_config.db`）
pub fn user_config_path() -> PathBuf {
    data_dir().join("user_config.db")
}

/// 获取任务数据库路径
///
/// # 返回
///
/// `PathBuf` - 任务数据库文件路径（`tasks.db`）
pub fn tasks_db_path() -> PathBuf {
    data_dir().join("tasks.db")
}

/// 确保数据目录存在
///
/// 如果数据目录不存在则创建。
///
/// # 返回
///
/// `anyhow::Result<()>` - 成功或错误
pub fn ensure_data_dir() -> anyhow::Result<()> {
    let dir = data_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}
