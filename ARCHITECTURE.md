# Architecture

High-level design for OS/3 Hyperspace — target system and current prototype.

**Phase tracker (source of truth):** [docs/PHASES.md](docs/PHASES.md)  
**Docs index:** [docs/README.md](docs/README.md)  
See root [README.md](README.md) for update convention when features ship.

---

## Target layers

```
┌─────────────────────────────────────────────────────────┐
│  5. User Polish — themes, a11y, onboarding              │
├─────────────────────────────────────────────────────────┤
│  4. Compatibility — POSIX, Linux, Windows bridges       │
├─────────────────────────────────────────────────────────┤
│  3. Hyperspace Shell — canvas, Smart Objects, dims      │
├─────────────────────────────────────────────────────────┤
│  2. Core Services — Hyperspace FS, AI runtime, sessions  │
├─────────────────────────────────────────────────────────┤
│  1. Microkernel — Redox fork, drivers, isolation        │
└─────────────────────────────────────────────────────────┘
```

---

## Current prototype (desktop)

Layer 3 runs on Windows/macOS/Linux today. Layers 2 are partially stubbed; layer 1 is future Redox work.

```
┌──────────────────────────────────────────────────────────┐
│  hyperspace-shell                                        │
│  eframe/egui · canvas · inspector · minimap · HUD        │
├────────────────────┬─────────────────────────────────────┤
│  hyperspace-core   │  Shared types (no UI dependency)    │
│  · HyperspaceState │  Dimension, Viewport, SmartObject   │
├────────────────────┼─────────────────────────────────────┤
│  hyperspace-fs     │  hyperspace-ai                      │
│  · ObjectStore     │  · AgentRuntime trait               │
│  · InMemoryStore   │  · StubModel (dev)                  │
│  · JsonWorkspace   │                                     │
└────────────────────┴─────────────────────────────────────┘
         │                              │
         ▼                              ▼
   workspace.json                  (future: local LLM)
   %APPDATA%/os3-hyperspace/
```

---

## Crate responsibilities

| Crate | Layer | Role |
|-------|-------|------|
| `hyperspace-core` | 2–3 | Pure types: dimensions, viewport transforms, Smart Objects, errors |
| `hyperspace-fs` | 2 | Storage abstractions; JSON persistence today, Redox FS later |
| `hyperspace-ai` | 2 | Agent trait boundary; stub today, local inference later |
| `hyperspace-shell` | 3 | Windowing, input, rendering, panels — **replaceable** with Orbital on Redox |

`hyperspace-shell` is the only crate that depends on eframe/egui. Core + services stay portable.

---

## Key concepts

### Dimensions

Independent zoomable workspaces with their own viewport and objects. Users switch contexts (Home, Work, …) via top-bar tabs. See [docs/smart-objects.md](docs/smart-objects.md).

### Smart Objects

Typed canvas entities: Note, App, Folder, Agent, Link. Serialized inside each dimension. Current: basic Link kind supported in types/inspector. Future/next: App launches, Folder mounts, Link crosses dimensions (documented target).

### Viewport

Camera in world space (`pan_x`, `pan_y`, `zoom`). `Viewport::screen_to_world` / `world_to_screen` keep objects anchored during pan/zoom. Zoom clamped to 8%–800%.

### Hyperspace FS

Address scheme (target): `hs://<dimension-id>/path/...`

| Implementation | Phase | Location |
|----------------|-------|----------|
| `InMemoryObjectStore` | 1 ✅ | Dev sync / tests |
| `JsonWorkspaceStore` | 1 ✅ | [persistence.md](docs/persistence.md) |
| Redox-native store | 1 ⬜ | Future daemon |

### AI Runtime

`AgentRuntime` trait isolates the shell from inference backend (stub today). Local inference is a documented next target. [docs/ai-runtime.md](docs/ai-runtime.md)

---

## Data flow (save/load)

```
Startup
  JsonWorkspaceStore::load_or_default()
    → HyperspaceState
    → InMemoryObjectStore::from_state()

Edit canvas / inspector
  → mutate HyperspaceState (dirty flag)

Save (Ctrl+S / exit)
  → JsonWorkspaceStore::save(&state)
```

---

## Redox migration path

1. Keep `hyperspace-core`, `hyperspace-fs`, `hyperspace-ai` target-agnostic
2. Add Redox `ObjectStore` + IPC agent service
3. New `hyperspace-shell` binary using Orbital (or fork orbital)
4. Retain desktop eframe build for fast UX dev on Windows

Details: [docs/redox-roadmap.md](docs/redox-roadmap.md)

---

## Dependency graph

```
hyperspace-shell
  ├── hyperspace-core
  ├── hyperspace-fs ──► hyperspace-core
  └── hyperspace-ai ──► hyperspace-core

hyperspace-fs
  └── hyperspace-core
```

No circular dependencies. Shell is the composition root.
