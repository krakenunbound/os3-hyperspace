# Changelog

All notable changes to OS/3 Hyperspace will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

See [docs/DEVELOPMENT-LOG.md](docs/DEVELOPMENT-LOG.md) for *exhaustive* session-by-session implementation details, design rationales, code pointers, test instructions, limitations, and resumption guides. This CHANGELOG is the high-level public summary.

## [Unreleased]

_Nothing yet — see the roadmap in [docs/ux-vision.md](docs/ux-vision.md) (next up: spring-animated camera + dimension cross-fades; exposing the command registry to the AI)._

## [0.2.0] - 2026-06-03 (Command Palette + bug-fix & UX pass)

> Headline: a **Command Palette (⌘K)** — the keyboard-complete spine of the shell — plus a
> round of real input/layout **bug fixes** that make manipulation actually work. See
> [docs/ux-vision.md](docs/ux-vision.md) for the design direction ("macOS beauty, Linux power").

### Added
- **Command Palette (⌘K / Ctrl+K)** — Spotlight-style fuzzy launcher (`crates/hyperspace-shell/src/palette.rs`):
  - Searchable registry of commands: spawn any object (at the view centre), new workspace,
    fit view, save, toggle side panel, ask/ping the local agent, delete selection, and
    "Go to Workspace: …" per dimension.
  - Lightweight fuzzy matcher (word-start / consecutive-run bonuses); ↑/↓ to move (wraps),
    ↵ to run, Esc / click-outside to dismiss; dim backdrop + centered glass card + auto-focus.
  - Data-only `CommandAction` enum the app interprets (no callbacks fighting `&mut self`),
    deliberately shaped so the AI can later *act* through the same registry.
  - Discoverable via the dock's **⌘ Commands** button and the top-bar **Spawn** button.
- **Fit view to content** (`F` key or dock button) — frames the whole active dimension
  (`fit_viewport_to_content`, unit-tested). First step toward the planned fluid camera.
- **Create workspaces (Dimensions) from the UI** — top-bar "New workspace…" field + ＋
  (Enter also submits). Previously only possible in code.
- **Resize cursors** — hovering a selected object's corner shows directional resize cursors.
- Resize handles for all Smart Objects (corner drag on canvas + DragValue editors in Inspector). Sizes snap on release. Min size enforcement. Works across Note/App/Folder/Agent/Link.
- Basic cross-dimension object linking via `Link` Smart Objects:
  - `link_target: Option<SmartObjectId>` field on SmartObject (persisted, optional for compat).
  - HUD spawn button for Link.
  - Inspector target picker (lists other dimensions, sets/clears target).
  - Click-to-activate navigation (switches active dimension if targeted).
  - Demo workspace now includes a pre-wired "Link to Work" example.
- Small visual polish: kind-colored accent header strip + top line on every object card for more futuristic card-like appearance.
- Full project skeleton now tracked in git + pushed to GitHub (previously only minimal docs skeleton was committed; source was on disk but untracked).
- **"Best OS" visual iteration** (see docs/DEVELOPMENT-LOG.md top entry for full self-reflection "how can we make the best os?" + "are we there yet?"):
  - Dynamic layered starfield background in canvas (world-anchored, zoom/pan reactive density/size/alpha for true hyperspace depth and immersion).
  - Link objects render as glowing portals/wormholes (concentric rings + event horizon core using accent color).
  - Agent objects render with inner neural glow + bright core to convey "alive" AI liveness.
  - These directly advance immersion, Smart Object "smartness", and the inspirational futuristic UI vision.

- **Major GUI Modernization** (see docs/DEVELOPMENT-LOG.md "2026-06-03 GUI Modernization" entry responding directly to user feedback + the high-end futuristic reference image provided):
  - Deep space + vibrant neon theme (much darker cosmic palette, purple/cyan/magenta accents, modern spacing/typography, glass-like panel fills).
  - Per-kind unicode icons (📝🚀📁🧠🌀) + small kind badges on object headers for instant recognition.
  - Premium glassmorphic Smart Object "cards": soft drop shadows, layered inset content rects for depth, strong multi-layer neon outer glow when selected, rich headers with icon + title + badge.
  - Richer cosmic background: existing starfield + new large low-alpha purple/cyan nebula clouds (directly evokes the beautiful nebulae, light streaks, and depth in the reference image).
  - Modern OS top menubar with Workspaces / Objects / System / AI sections (inspired by the reference top bar).
  - Polished left "HYPERDRIVE" and right "INSPECTOR" glass side panels with better section headers and icon usage.
  - New glassy bottom dock/taskbar with quick actions and "Local-first" branding (completes the full modern desktop silhouette).
  - Subtler futuristic grid + glass-themed minimap/overlay.
  - All changes dramatically improve "modern and attractive" feel while preserving the core infinite canvas + Smart Objects vision.

- **"Let's rock" GUI Polish follow-up** (see docs/DEVELOPMENT-LOG.md top "Let's rock" entry):
  - Cinematic background: added 5 glowing purple energy light ribbons/streaks with soft multi-pass glow + parallax. Combined with stars + nebulae for intense space energy matching the reference image.
  - Interactive window titlebar chrome on selected Smart Objects: drawn [−] [□] [X] controls (close is red-tinted). Clicking the X area now deletes the object (real close button behavior).
  - Richer demo content in objects so they look like the windows in the reference: Terminal with shell output + prompt, Projects with file listing, new System/About with logo block and specs, updated Welcome note.
  - Updated docs and log with full details.

### Changed
- The Command Palette (⌘K) is now the primary way to act in the shell; scattered buttons
  funnel into one registry. `handle_shortcuts` lets the palette own the keyboard while open.
- Canvas input now prioritizes resize handles on selected objects before object dragging.
- Event system extended: `CanvasEvent::Resized` and `CanvasEvent::LinkActivate`.
- HUD controls text and Inspector updated for new interactions.
- Demo content bodies mention new features (resize hint, link usage).
- `with_demo_content()` now wires cross-dim Link example.

### Documentation
- New [docs/ux-vision.md](docs/ux-vision.md): the UX design spine — principles, a macOS-polish
  checklist mapped to concrete work, and a leverage-ordered roadmap.
- [docs/DEVELOPMENT-LOG.md](docs/DEVELOPMENT-LOG.md): two new dated entries (bug-fix pass; Command Palette).
- [docs/dev-windows.md](docs/dev-windows.md): ⌘K / F controls + a Command Palette section.
- Massive update per project convention (see docs/DEVELOPMENT-LOG.md:2026-06-03 entry for the full "document the hell out of it" process).
- [docs/PHASES.md](docs/PHASES.md): Status tables, next targets, dates, notes.
- [docs/smart-objects.md](docs/smart-objects.md): Interaction table, impl notes, Link section, header.
- [docs/dev-windows.md](docs/dev-windows.md): Controls, HUD, Inspector, "implemented in this build" banner.
- [TODO.md](../TODO.md): Checkboxes + notes.
- New [docs/DEVELOPMENT-LOG.md](docs/DEVELOPMENT-LOG.md): Verbose dated sessions with code locations (file:line), decisions, test steps, limitations, how to resume.
- This CHANGELOG created.
- Code-level rustdoc + inline comments added in canvas.rs, app.rs, object.rs, lib.rs, dimension.rs.
- Root README / ARCHITECTURE lightly synced for status.
- Commits use verbose messages referencing the log.

### Fixed / Technical
- **Resize handles & close button now actually work.** They were hit-tested in *global*
  screen coords but compared against a *canvas-local* pointer, so the grab/click zones were
  offset from the drawn handles by the panel origin (~260px×40px with the HUD shown). Now
  hit-tested in canvas-local space (new `object_local_rect` helper).
- **Close zone no longer swallows minimize/maximize.** The old 50px-wide control zone covered
  all three chrome buttons; narrowed to a tight box around the × glyph only.
- **Bottom dock no longer occludes the minimap + zoom overlay.** It was created *inside* the
  `CentralPanel` and painted over the canvas; now declared as a sibling panel *before* it.
- **Dock "New Note" spawns at the visible canvas centre** (`-pan`) instead of off-screen.
- **Wired up the dead `new_dimension_name` field** → the create-workspace UI above.
- **clippy:** trimmed excessive f32 literal precision in the starfield hash.
- Borrow checker workarounds for dimension mutation during Link nav (pending state collector, modeled after agent prompts).
- No breaking changes to public types or persistence format (new field is optional + defaulted).

See the git history (c7a0756 + dcd9950) and DEVELOPMENT-LOG for exact diffs + resumption steps.

## [0.1.0] - 2026-06-03 (skeleton + initial features)

### Added
- Initial runnable desktop prototype (eframe/egui on Windows):
  - Infinite zoomable/pannable canvas with grid.
  - Multiple Dimensions with tab switching + creation.
  - 5 Smart Object kinds (Note, App, Folder, Agent, Link) with color accents, inspector, drag-move (20px snap), selection, delete, double-click spawn (Notes), minimap, HUD controls, AI stub integration, JSON persistence (auto load/save on dirty/Ctrl+S/exit).
- Full Rust Cargo workspace: hyperspace-core (types, viewport math, state), hyperspace-shell (UI), hyperspace-fs (store + JsonWorkspaceStore), hyperspace-ai (AgentRuntime trait + StubModel).
- Comprehensive living docs (PHASES.md as source of truth with tables/exit criteria/next order, concept specs, dev guide, architecture, TODO mirror).
- scripts/dev.ps1, .gitignore, MIT license.
- GitHub skeleton initially minimal; full source + docs committed and pushed in this period.

### Notes
- This establishes the "Phase 0 running + early Phase 2" baseline.
- Everything before the Unreleased section was part of the initial skeleton establishment + first feature milestone (resize + linking).

For exhaustive details of the 2026-06-03 work, see [docs/DEVELOPMENT-LOG.md](docs/DEVELOPMENT-LOG.md).

[Unreleased]: #unreleased
[0.2.0]: #020---2026-06-03-command-palette--bug-fix--ux-pass
[0.1.0]: #010---2026-06-03-skeleton--initial-features