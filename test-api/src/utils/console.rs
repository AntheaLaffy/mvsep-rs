pub fn init() {
    #[cfg(windows)]
    {
        if let Err(e) = colored::control::set_virtual_terminal(true) {
            eprintln!("Warning: Failed to enable virtual terminal: {}", e);
        }
    }

    #[cfg(windows)]
    unsafe {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            if let Some(lib) = winapi::shared::minwindef::HMODULE::from_raw(1) {
                let _ = lib;
            }
        });
    }

    #[cfg(not(windows))]
    let _ = ();
}
