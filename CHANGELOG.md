# Changelog

## Desktop 1.3.1

- Bundle a `.DirIcon` that actually resolves: Tauri CLI 2.12.3 creates it as a relative symlink, whereas 2.10.1 pointed it at an absolute path on the build machine, which AppImageHub rejected with `FATAL: .DirIcon is missing`.
- Drop the `bundle.linux.appimage.files` `.DirIcon` entry: custom files land under the AppDir's `usr/` tree, so it never reached the root the validator looks at.

## Unreleased

- Discontinue the AUR source-build package in favor of the prebuilt `mvsep-gui-bin` AppImage package.
- Skip CI for documentation-only changes.

## Desktop 1.3.0 / API crate 0.1.1

- Add optional desktop notifications when a separation task finishes or fails while the app is in the background, with a settings toggle and a test action.
- Check desktop and API crate version consistency in CI and release workflows.
- Provide independent version update commands without altering transitive dependencies.
- Follow `LC_ALL`, `LC_MESSAGES`, then `LANG` for startup language selection; preserve saved preferences and fall back to English for unsupported locales.
- Use an English startup window title and default README.
- Include the root `.DirIcon` required by AppImageHub validation.
- Replace Chinese endpoint reference and Rust API documentation with English; fix two upload examples' form key types.
- Replace retired rewrite skills with reusable engineering skills and focused MVSEP guidance.

These entries describe the prepared source changes; publication is tracked by the corresponding registry versions and GitHub release.
