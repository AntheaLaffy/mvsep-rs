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
