# Resources

This file records the high-trust resources and reusable ideas that guide mvsep-rs rewrite work. It follows the `teach` skill's resource-log idea: record where knowledge comes from, what it is used for, and what not to import.

## Project Truth

| Resource | Use For | Do Not Use For |
|---|---|---|
| `docs/mission.md` | Project mission, goals and non-goals | Detailed implementation choices |
| `docs/architecture/backend-rewrite.md` | The accepted mvsep-rs rewrite architecture | Replacing current architecture without an ADR |
| `manifest/rewrite-status.yaml` | Current migration batch state | Free-form notes or wish lists |
| `rewrite-records/` | Non-obvious rewrite lessons and source-boundary decisions | Runtime state or task tracking |
| `Note.md` | Durable style, UX and engineering preferences | Formal architecture decisions |
| `CONTEXT.md` | Canonical domain terms | Implementation plans |

## External And Skill Resources

| Resource | Use For | Borrow | Do Not Borrow |
|---|---|---|---|
| `teach` skill | Stateful workspace design | Mission, resources, notes, records, minimum scoped progress | Teaching lessons, HTML lesson outputs, learning-specific pedagogy |
| `py2rs` skill | Software engineering discipline for rewrites | Behavior-first migration, reversible batches, manifest state, role-separated review, quality gates | Python/Rust directory architecture, py/rs runtime router, script-as-unit model, Python-specific dependency alignment |
| Tauri 2 official docs | Command/state/event/capability details | API usage and security model | Third-party examples as authority |
| Tailwind official docs | Tailwind v3 current config and future v4 upgrade planning | Version-specific build and design-token guidance | Mixing Tailwind v4 upgrade into backend batches |
| Local tests and code | Actual current behavior | Fixture and contract truth | Unverified assumptions about remote API shapes |

## Resource Rules

- Read project truth before external resources.
- Record a new `rewrite-records/` entry when a source boundary or migration lesson changes future behavior.
- If py2rs is referenced, explicitly state which principle is being borrowed and which py2rs architecture detail is not being used.
- If teach is referenced, translate learning concepts into engineering equivalents: lesson -> minimum migration slice, learning record -> rewrite record, resources -> project resources, mission -> rewrite mission.
