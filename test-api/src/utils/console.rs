//! 控制台工具函数
//!
//! 提供控制台初始化和颜色支持。

/// 初始化控制台
///
/// 在 Windows 上启用虚拟终端以支持颜色输出。
/// 在 Unix 系统上无需特殊处理。
pub fn init() {
    #[cfg(windows)]
    {
        if let Err(e) = colored::control::set_virtual_terminal(true) {
            eprintln!("Warning: Failed to enable virtual terminal: {:?}", e);
        }
    }

    #[cfg(not(windows))]
    let _ = ();
}
