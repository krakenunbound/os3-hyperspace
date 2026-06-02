# OS/3 Hyperspace — Windows dev guide

Run and iterate on the shell prototype while Redox integration is in progress.

**Related:** [PHASES.md](PHASES.md) · [smart-objects.md](smart-objects.md) · [persistence.md](persistence.md) · [redox-roadmap.md](redox-roadmap.md)

Implemented in this build (and latest iteration): resize handles (canvas + inspector), basic Link navigation + target setting in inspector, demo Link prewired, accent header polish on objects, dynamic starfield for hyperspace immersion, Link as glowing portals, Agents with neural glow for liveness.

Still documented targets: full object linking polish (visual connections), Redox VM, local inference, deeper Smart Object behaviors (nesting, reactivity).

---

## Prerequisites

- [Rust](https://rustup.rs/) stable (`rustc`, `cargo`)
- Windows 10/11 with GPU drivers for the native window backend (`eframe`)

```powershell
rustc --version
cargo --version
```

---

## Commands

| Command | Action |
|---------|--------|
| `cargo run -p hyperspace-shell` | Launch the shell |
| `cargo build --workspace` | Build all crates |
| `cargo test --workspace` | Run tests |
| `.\scripts\dev.ps1 run` | Same as `cargo run` (default) |
| `.\scripts\dev.ps1 test` | Run tests |

First build downloads dependencies (~430 crates); subsequent builds are fast.

---

## UI layout

```
┌─────────────────────────────────────────────────────────────┐
│ Top bar: title · dimension tabs · + Dimension · Save · HUD  │
├──────────┬──────────────────────────────────────┬───────────┤
│ Left HUD │ Infinite canvas (grid, objects)      │ Inspector │
│ Controls │                                      │ (when     │
│ Spawn    │                          [minimap]   │ selected) │
├──────────┴──────────────────────────────────────┴───────────┤
│ Status: dimension name · zoom %                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Controls

### Canvas

| Input | Action |
|-------|--------|
| Scroll wheel | Zoom toward cursor |
| Middle-drag | Pan |
| Space + drag | Pan |
| Double-click (empty) | Create Note at cursor (always Note; Link kind not double-click creatable today) |
| Drag object | Move (snaps to 20px grid) |
| Drag corner handles (selected) | Resize object (snaps size to grid on release) |
| Canvas background | Dynamic starfield (layered, zoom/pan reactive for hyperspace depth/immersion) |
| Click object | Select → Inspector opens (all kinds including Link) |
| Click Agent | Select + invoke stub agent |
| Click Link | Select + if target set, navigate to that dimension (Link renders as glowing portal) |
| Click Agent | Select + invoke stub; Agent renders with neural glow for "liveness" |
| Click empty canvas | Deselect |

### Keyboard

| Key | Action |
|-----|--------|
| **Ctrl+S** | Save workspace |
| **Delete** | Remove selected object (only when not typing in Inspector) |
| **Escape** | Clear selection |

### Top bar

| Control | Action |
|---------|--------|
| **Home / Work / …** | Switch dimension |
| **+ Dimension** | Create dimension (optional name field) |
| **Save** | Write workspace to disk |
| **Toggle HUD** | Show/hide left panel |
| **●** (amber dot) | Unsaved changes indicator |

### Left HUD

- **Spawn** — create Note, App, Folder, Agent, or Link at origin
- **AI Runtime** — ping stub agent or ask about active dimension
- **Hyperspace FS** — sync in-memory store, show object count

### Inspector (right, when selected)

- Edit title and body
- Edit size (or drag handles on canvas)
- For **Link**: choose target dimension from other dims (or clear)
- **Delete object**

---

## Persistence

Workspace file:

```
%APPDATA%\os3-hyperspace\workspace.json
```

- Loads automatically on startup
- Saves on **Ctrl+S**, **Save** button, or exit (if dirty)
- Missing/invalid file → demo Home + Work dimensions

Details: [persistence.md](persistence.md)

Backup:

```powershell
Copy-Item "$env:APPDATA\os3-hyperspace\workspace.json" "$env:USERPROFILE\Desktop\hyperspace-backup.json"
```

---

## Workspace crates

| Crate | Role |
|-------|------|
| `hyperspace-core` | Types: dimensions, viewport, Smart Objects |
| `hyperspace-shell` | Runnable UI (this binary) |
| `hyperspace-ai` | Agent trait + stub |
| `hyperspace-fs` | Object store + JSON persistence |

---

## Troubleshooting

| Issue | Try |
|-------|-----|
| Window doesn't open | Update GPU drivers; run `cargo run -p hyperspace-shell` from repo root |
| Blank canvas | Reset workspace: delete `%APPDATA%\os3-hyperspace\workspace.json` and restart |
| Build errors after pull | `cargo build --workspace` — check Rust is stable |

---

## Next steps toward Redox

1. [Redox VM setup](redox-roadmap.md)
2. Fork orbital + storage components
3. Port shell from eframe → native Orbital app

Track progress in [PHASES.md](PHASES.md).
