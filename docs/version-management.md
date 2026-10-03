# Version management

The desktop app (`mvsep-gui`) and API crate (`mvsep-api-tester`) have independent versions. Use stable Semantic Versioning: patch for compatible fixes or documentation releases, minor for compatible features, and major for breaking changes. For the API crate's `0.x` series, a minor increment may introduce breaking changes.

## Check and update

From the repository root (Python 3.11 or later):

```bash
npm run version:check
npm run version:bump -- desktop 1.2.3
npm run version:bump -- api 0.1.1
```

The desktop command synchronizes `package.json`, the root entries in `package-lock.json`, `src-tauri/tauri.conf.json`, and the desktop package entries in `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`. The API command updates only its own manifest and lockfile entries. Neither command changes transitive dependency versions or publishes anything. Bun's lockfile does not currently store the root version.

The checker rejects inconsistent versions; the updater rejects invalid versions and non-increasing updates before writing. Review the resulting diff and record the release in `CHANGELOG.md`.

## Publish in dependency order

1. Run the checks appropriate to the changed code, including API doctests for public documentation.
2. If the API crate changed, publish it first with `cargo publish --manifest-path test-api/Cargo.toml`. Wait for registry availability before updating the desktop's dependency requirement and running `cargo update --manifest-path src-tauri/Cargo.toml -p mvsep-api-tester`.
3. Verify the desktop package and publish with `cargo publish --manifest-path src-tauri/Cargo.toml`.
4. Commit and push the final version changes and lockfiles. Tag the desktop release as `v<desktop-version>`; the existing tag workflow publishes the installers.
5. Verify the release assets and CI results before claiming release completion or replying with artifact links.

CI checks manifest/lockfile agreement. Release CI also checks that a pushed tag matches the desktop version. Manual release dispatch checks version consistency but does not substitute for verifying the intended tag and release assets.

Publication, tag pushes, and external replies require the user's authorization. Once authorized, complete the concrete release and verification without adding redundant approval steps.
