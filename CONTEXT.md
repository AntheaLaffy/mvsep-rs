# Context

This file is the project glossary. It records canonical terms only, not implementation plans.

## Terms

**MVSep**

The remote music separation platform used by this project.

**Separation Task**

A user request to upload an audio file, run a selected separation algorithm and later download output files.

**Algorithm Cache**

The local copy of available MVSep algorithms, groups, fields and output format metadata.

**Output Format**

The requested file format and bit depth for separated results.

**Preset**

A saved combination of algorithm, options, output format and demo flag.

**Old Backend**

The backend behavior currently embedded in the Tauri desktop application.

**Rewritten Backend**

The Rust backend capability being extracted and stabilized from `test-api`.

**Backend Gateway**

The TypeScript adapter at `src/app/backend/gateway.ts`. It is the only frontend module that should import Tauri JavaScript APIs, call `invoke`, or call `listen`.

**Canonical Backend Store**

The rewritten backend persistence layer for migrated domains. When backend state and legacy frontend-local state disagree for a migrated domain, the backend store is authoritative unless a migration record explicitly says otherwise.

**Migration Batch**

A small, independently testable step that moves one capability from old backend behavior to rewritten backend behavior.

**Review Gate**

A required review checkpoint before a migration batch can be promoted.
