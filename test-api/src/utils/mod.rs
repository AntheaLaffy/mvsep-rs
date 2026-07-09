//! 工具函数模块
//!
//! 提供路径处理、控制台输出等通用工具函数。

pub mod console;
pub mod paths;

use std::path::PathBuf;

/// 获取数据目录路径
///
/// 根据操作系统返回合适的数据目录：
/// - Unix: 当前目录 `.`
/// - Windows: `%APPDATA%\mvsep-tester`
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
/// 默认路径为数据目录下的 `mvsep.db`。
///
/// # 返回
///
/// `PathBuf` - 数据库文件路径
pub fn db_path() -> PathBuf {
    data_dir().join("mvsep.db")
}
