//! Console utilities
//!
//! Provides console initialization and color support.

/// Initialize the console
///
/// Enables virtual terminal processing on Windows for color output.
/// No special handling is needed on Unix systems.
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
