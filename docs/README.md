# OS/3 Hyperspace Documentation

Living documentation for the project. Updated as each phase lands.

**Index entry point.** See also root [../README.md](../README.md), [../ARCHITECTURE.md](../ARCHITECTURE.md), and [../TODO.md](../TODO.md).

## Phase tracker (source of truth)

| Doc | Purpose |
|-----|---------|
| [PHASES.md](PHASES.md) | Phases 0–4 with status tables (✅ / ⬜), exit criteria per phase, and suggested next build order. |

Current: Phase 0 (prototype running) with early Phase 2 shell features done.

## Concept specs

| Doc | Purpose |
|-----|---------|
| [smart-objects.md](smart-objects.md) | Smart Object types, interaction, future behavior (Note, App, Folder, Agent, Link) |
| [persistence.md](persistence.md) | JSON schema, paths, save triggers, future Hyperspace FS |
| [ai-runtime.md](ai-runtime.md) | Stub today, local inference plan |
| [redox-roadmap.md](redox-roadmap.md) | VM setup, forks, migration path |

## Updated root docs

| Doc | Purpose |
|-----|---------|
| [../README.md](../README.md) | Status table, doc links, update convention |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Target vs. current layers, crate map, data flow |
| [../TODO.md](../TODO.md) | Short checklist linking to phase + concept docs |
| [dev-windows.md](dev-windows.md) | UI layout diagram, full controls, troubleshooting (Windows prototype) |

## Conventions going forward

When something ships, update in this order:

1. `docs/PHASES.md` + `TODO.md` — check off items
2. Relevant concept doc (smart-objects, persistence, etc.)
3. `ARCHITECTURE.md` if crate boundaries change
4. `dev-windows.md` if controls or workflow change

See [PHASES.md](PHASES.md) for the detailed tracker and next targets (object linking, Redox VM, resize handles, local inference).
