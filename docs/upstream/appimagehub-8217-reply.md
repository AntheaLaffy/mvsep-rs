Suggested reply for https://github.com/AppImage/appimage.github.io/pull/8217#issuecomment-5899932516

Thank you! MVSEP is useful internationally, and the application already includes English, Chinese and Japanese translations, as well as an English README: https://github.com/AntheaLaffy/mvsep-rs/blob/main/README.en.md

I have prepared a change to detect the system locale in the order `LC_ALL`, `LC_MESSAGES`, then `LANG`, with the WebView language as a fallback when none is set. Chinese and Japanese locales use their respective translations, and all other locales (including `C` and `POSIX`) default to English. An explicitly saved language preference continues to take precedence. The initial native window title is now English as well.

The frontend build, locale detection checks, startup preference checks and Rust locale precedence test pass. This change still needs to be included in a new AppImage release before the catalog can test the updated binary.
