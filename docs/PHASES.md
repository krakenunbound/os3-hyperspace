# Project Phases

**Current phase:** Phase 0 (prototype running) with early Phase 2 shell features done.

**Last updated:** 2026-06-03 (v0.2.0 — Command Palette ⌘K + input/layout bug-fix & UX pass)

This is the source of truth for project progress (Phases 0–4 with status tables ✅ / ⬜, exit criteria, and suggested build order). [TODO.md](../TODO.md) is a short mirror of this file. See [README.md](README.md) (in docs/) for the documentation index.

**Update convention:** When something ships, update `docs/PHASES.md` + `TODO.md` first (check off), then the relevant concept doc, then `ARCHITECTURE.md` (if crate boundaries changed), then `dev-windows.md` (if controls/workflow changed). See [README.md](README.md) (in docs/) for the full numbered list and structure.

---

## Phase 0: Setup

**Goal:** Dev environment and a runnable prototype on the host OS (Windows first).

| Item | Status | Notes |
|------|--------|-------|
| Git repo + README | ✅ Done | [github.com/krakenunbound/os3-hyperspace](https://github.com/krakenunbound/os3-hyperspace) |
| Rust Cargo workspace | ✅ Done | Four crates under `crates/` |
| Windows dev workflow | ✅ Done | [dev-windows.md](dev-windows.md), [scripts/dev.ps1](../scripts/dev.ps1) |
| Runnable shell prototype | ✅ Done | `cargo run -p hyperspace-shell` |
| Redox desktop in VM | ⬜ Todo | See [redox-roadmap.md](redox-roadmap.md) |
| Fork Redox repos | ⬜ Todo | Kernel, orbital, relevant libs |
| Redox dev environment | ⬜ Todo | Cross-build + VM test loop |

**Exit criteria:** Redox VM running; Hyperspace crates building in host + target paths identified.

---

## Phase 1: Foundation

**Goal:** Core types and service boundaries that survive the move to Redox.

| Item | Status | Notes |
|------|--------|-------|
| `hyperspace-core` types | ✅ Done | Dimensions, viewport, Smart Objects |
| `hyperspace-fs` in-memory store | ✅ Done | `ObjectStore` trait |
| `hyperspace-fs` JSON persistence | ✅ Done | [persistence.md](persistence.md) |
| `hyperspace-ai` trait + stub | ✅ Done | [ai-runtime.md](ai-runtime.md) |
| Hyperspace FS on Redox | ⬜ Todo | Replace JSON with native storage |
| AI runtime (local inference) | ⬜ Todo | Plug model behind `AgentRuntime` |
| Kernel hardening (Redox fork) | ⬜ Todo | Security, drivers, stability |

**Exit criteria:** Workspace persists on Redox; agent trait backed by a real local model.

---

## Phase 2: Shell MVP

**Goal:** Infinite canvas shell with dimensions and Smart Objects — first on desktop, then native on Redox.

| Item | Status | Notes |
|------|--------|-------|
| Infinite zoomable canvas | ✅ Done | Scroll zoom, pan, grid |
| Dimension switching | ✅ Done | Top-bar tabs + create new |
| Smart Objects | ✅ Done | [smart-objects.md](smart-objects.md) |
| Object selection + inspector | ✅ Done | Right panel edit |
| Grid snap | ✅ Done | 20px world grid |
| Minimap | ✅ Done | Bottom-right overview |
| Persistent layout | ✅ Done | Auto-load/save JSON |
| Canvas immersion & object liveness visuals | ✅ Major step + polish | Deep space theme, layered starfield + nebulae + glowing energy streaks, premium glassmorphic cards with icons/badges/shadows/glows + interactive window chrome (close button works on selected), richer demo content (terminal, about, folders), modern top bar/HUDs/dock (see DEVELOPMENT-LOG "Let's rock" entry + user reference image). Still more polish possible. |
| Native Redox orbital shell | ⬜ Todo | Replace eframe/egui layer |
| Object linking across dimensions | 🟡 Partial | `Link` kind + link_target + HUD spawn + Inspector picker + click-to-navigate + demo prewire + persistence. Polish (arrows, object targets, feedback) todo. See [smart-objects.md](smart-objects.md) |
| Resize Smart Objects | ✅ Done | Drag corner handles on selection (live + snap); Inspector size DragValues; min sizes, works on all kinds incl Links. **Hit-testing bug fixed in 0.2.0** (was offset by panel origin); resize cursors on hover |
| Command Palette (⌘K) | ✅ Done (0.2.0) | Spotlight-style fuzzy launcher; keyboard-complete command registry (`palette.rs`). The spine new features register into. See [ux-vision.md](ux-vision.md) §3.1 |
| Fit view to content | ✅ Done (0.2.0) | `F` / dock / palette; centroid-frames the dimension (`fit_viewport_to_content`) |

**Exit criteria:** Same UX on Redox as desktop prototype; layouts portable between both.

---

## Phase 3: Compatibility

**Goal:** Run existing software inside Hyperspace.

| Item | Status | Notes |
|------|--------|-------|
| POSIX / Linux compatibility layer | ⬜ Todo | Via Redox + additions |
| Windows app bridges | ⬜ Todo | Long-term |
| Terminal / shell integration | ⬜ Todo | `App` objects launch real programs |
| Folder → filesystem mount | ⬜ Todo | `Folder` objects map to Hyperspace FS paths |

**Exit criteria:** Launch terminal and at least one external app from a Smart Object.

---

## Phase 4: Polish & Release

**Goal:** Production-quality UX and distributable OS image.

| Item | Status | Notes |
|------|--------|-------|
| Theme system | ⬜ Todo | Beyond current dark prototype theme |
| Accessibility pass | ⬜ Todo | Keyboard nav, contrast, screen readers |
| Onboarding flow | ⬜ Todo | First-run dimension setup |
| Redox VM integration tests | ⬜ Todo | CI smoke tests |
| Installer / ISO pipeline | ⬜ Todo | Bootable Hyperspace image |

**Exit criteria:** Bootable ISO; new user can reach a working dimension in under 5 minutes.

---

## What to build next (suggested order)

Next documented targets (resize + basic linking landed):

1. Polish object linking (visual links/arrows, richer targets like objects/URLs, activation UX) — see [smart-objects.md](smart-objects.md)
2. Redox VM + fork plan ([redox-roadmap.md](redox-roadmap.md))
3. Local inference behind `AgentRuntime` — see [ai-runtime.md](ai-runtime.md)
4. Deeper Smart Object behaviors, theme/a11y, more canvas polish

(These drive the Phase 0 exit + Phase 1/2 progress. See updated dev-windows.md for current controls.)
