# OS/3 Hyperspace — Windows dev guide

Run and iterate on the shell prototype while Redox integration is in progress.

**Related:** [PHASES.md](PHASES.md) · [smart-objects.md](smart-objects.md) · [persistence.md](persistence.md) · [redox-roadmap.md](redox-roadmap.md)

Implemented (major GUI modernization): Deep space neon theme, rich starfield + nebulae background, premium glassmorphic Smart Object cards (shadows, layered bodies, icons + kind badges, neon glow on select, portal Links, glowing Agents), modern top menubar (Workspaces/Objects/System/AI sections), polished glass HUD + Inspector panels, bottom dock/taskbar. See DEVELOPMENT-LOG "GUI Modernization" entry and the high-end reference image provided by user.

Previous: resize, basic linking, etc.

Still documented targets: visual link connections, richer object content, Redox native shell, deeper AI behaviors.

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
| Canvas background | Dynamic starfield + nebulae + glowing energy streaks (layered, zoom/pan reactive for hyperspace depth/immersion) |
| Click object | Select → Inspector opens (all kinds including Link). Selected shows window chrome in header. |
| Click Agent | Select + invoke stub agent |
| Click Link | Select + if target set, navigate to that dimension (Link renders as glowing portal) |
| Click Agent | Select + invoke stub; Agent renders with neural glow for "liveness" |
| Click empty canvas | Deselect |
| Click close button (X on selected header) | Delete the object (real window control feel) |

### Keyboard

| Key | Action |
|-----|--------|
| **⌘K / Ctrl+K** | Open/close the **Command Palette** (Spotlight-style: search + run any command) |
| **F** | Fit the view to the active dimension's content |
| **Ctrl+S** | Save workspace |
| **Delete** | Remove selected object (only when not typing in Inspector) |
| **Escape** | Clear selection (or close the palette if open) |

#### Command Palette (⌘K)

The fastest way to do anything. Open with ⌘K/Ctrl+K (or the dock's **⌘ Commands** button),
type to fuzzy-search, **↑/↓** to move, **↵** to run, **Esc** or click-outside to dismiss.
Commands include: spawn any object (at the view centre), new workspace, fit view, save,
toggle side panel, ping/ask the local agent, delete selected, and "Go to Workspace: …" for
each other dimension. New features should register here first — it's the keyboard-complete
spine of the shell (see [ux-vision.md](ux-vision.md)).

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
