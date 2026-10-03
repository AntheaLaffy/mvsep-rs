# Changelog

## Unreleased

- Check desktop and API crate version consistency in CI and release workflows.
- Provide independent version update commands without altering transitive dependencies.

## Desktop 1.2.3 / API crate 0.1.1

- Follow `LC_ALL`, `LC_MESSAGES`, then `LANG` for startup language selection; preserve saved preferences and fall back to English for unsupported locales.
- Use an English startup window title and default README.
- Include the root `.DirIcon` required by AppImageHub validation.
- Replace Chinese endpoint reference and Rust API documentation with English; fix two upload examples' form key types.
- Replace retired rewrite skills with reusable engineering skills and focused MVSEP guidance.

These entries describe the prepared source changes; publication is tracked by the corresponding registry versions and GitHub release.
