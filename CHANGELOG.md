# Changelog

All notable changes to OS/3 Hyperspace will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

See [docs/DEVELOPMENT-LOG.md](docs/DEVELOPMENT-LOG.md) for *exhaustive* session-by-session implementation details, design rationales, code pointers, test instructions, limitations, and resumption guides. This CHANGELOG is the high-level public summary.

## [Unreleased]

### Added
- Resize handles for all Smart Objects (corner drag on canvas + DragValue editors in Inspector). Sizes snap on release. Min size enforcement. Works across Note/App/Folder/Agent/Link.
- Basic cross-dimension object linking via `Link` Smart Objects:
  - `link_target: Option<SmartObjectId>` field on SmartObject (persisted, optional for compat).
  - HUD spawn button for Link.
  - Inspector target picker (lists other dimensions, sets/clears target).
  - Click-to-activate navigation (switches active dimension if targeted).
  - Demo workspace now includes a pre-wired "Link to Work" example.
- Small visual polish: kind-colored accent header strip + top line on every object card for more futuristic card-like appearance.
- Full project skeleton now tracked in git + pushed to GitHub (previously only minimal docs skeleton was committed; source was on disk but untracked).

### Changed
- Canvas input now prioritizes resize handles on selected objects before object dragging.
- Event system extended: `CanvasEvent::Resized` and `CanvasEvent::LinkActivate`.
- HUD controls text and Inspector updated for new interactions.
- Demo content bodies mention new features (resize hint, link usage).
- `with_demo_content()` now wires cross-dim Link example.

### Documentation
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
[0.1.0]: #010---2026-06-03-skeleton--initial-features