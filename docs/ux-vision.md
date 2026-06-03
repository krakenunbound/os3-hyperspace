# UX Vision — "macOS beauty, Linux power (or better)"

> North star: a desktop that feels as **calm, fluid, and beautiful** as macOS, but is as
> **open, scriptable, and powerful** as Linux — reimagined around an **infinite spatial
> canvas** and **local-first AI**, not a stack of overlapping windows.

This doc is the design spine for the shell. It separates the **principles** (won't change
often) from the **roadmap** (what to build next). Implementation status is tracked in
[PHASES.md](PHASES.md) and [../TODO.md](../TODO.md).

---

## 1. Design principles

1. **Calm by default, powerful on demand.** A near-empty canvas at rest. Power (command
   palette, terminal, scripting, AI) is one keystroke away but never in your face.
2. **One spatial model.** Everything lives on the same zoomable canvas as a *Smart Object*.
   No mode switching between "files", "windows", and "apps" — just objects at coordinates.
3. **Direct manipulation.** Drag, resize, zoom, link. Every object behaves consistently;
   affordances (handles, cursors, hover states) tell you what's grabbable.
4. **Motion with meaning.** Animation communicates *where things went* (zoom-to-fit,
   dimension transitions), never decoration for its own sake. Target 60fps, respect a
   "reduce motion" setting.
5. **Local-first & yours.** AI runs on-device; state is plain, inspectable JSON today and
   an open FS tomorrow. No mandatory cloud, no lock-in. This is the "Linux power" half.
6. **Keyboard-complete.** Anything you can do with the mouse you can do from the keyboard
   and (later) from a script. Discoverable via a single palette.

---

## 2. The macOS-grade polish checklist

What makes macOS *feel* premium, mapped to concrete shell work:

| macOS quality | What it really is | Our plan |
|---|---|---|
| Depth & materials | Layered translucency, soft shadows, vibrancy | ✅ glass cards + glow (done); ⬜ true blur/vibrancy behind panels |
| Fluidity | Spring animations, momentum scroll/zoom | ⬜ animated zoom-to-fit & dimension cross-fade; momentum pan |
| Precision | Pixel-snapped text, consistent radii & spacing | ⬜ unify corner radii via theme tokens; pixel-snap labels |
| Affordance | Cursors change to match action | ✅ resize cursors on handles (done) |
| Spotlight | One box to find/launch/do anything | ✅ **command palette** (⌘K) — fuzzy search, keyboard-complete |
| Mission Control | Zoom out to see everything | 🟡 minimap + **Fit to content** (F) done; ⬜ animated overview |
| Quietness | Restraint in color & chrome | 🟡 tune neon down a notch for long sessions; light theme |

---

## 3. Signature interactions (the roadmap)

Ordered by leverage. Each is a self-contained, shippable slice.

### 3.1 Command Palette (⌘K / Ctrl+K) — ✅ shipped
A single fuzzy-searchable box, Spotlight-style, that can: spawn any object (at the view
centre), switch/create dimensions, run AI prompts, fit view, save, toggle panels, delete
selection. The keyboard-complete backbone — every new feature registers one `Command` and
is instantly discoverable. Implemented in `crates/hyperspace-shell/src/palette.rs` using a
data-only `CommandAction` enum the app interprets (no callbacks fighting the borrow checker).
**Next within this slice:** recent/frequent ordering, inline arguments (e.g. "rename →"),
and exposing the same registry to the AI so it can *act* through the palette.

### 3.2 Fluid camera
- Animated **zoom-to-fit** and **zoom-to-selection** (ease-out spring) instead of a jump.
- **Momentum pan** and smooth pinch/scroll zoom (already smooth-ish; add inertia).
- **Cross-dimension transition**: a quick zoom-through/fade when following a Link, so the
  "multidimensional" model is felt, not just stated.

### 3.3 Living Smart Objects
- Real window chrome behaviors: minimize (collapse to header), maximize (zoom-to-object),
  close (done). Currently min/max are drawn but inert.
- Inline editing on the canvas (double-click title/body) instead of only the Inspector.
- Type-specific content: Folder shows real children; App launches; Agent is a chat surface.

### 3.4 The AI layer as a first-class citizen
- A persistent, dockable **Copilot** that can see the current dimension and *act* (spawn,
  arrange, link) via the same command registry — AI and user share one command vocabulary.
- Per-object "ask" affordance (the Agent object is the seed of this).

### 3.5 Theming & accessibility ("Linux power, humane defaults")
- Theme **tokens** (colors, radii, spacing, motion) in one place; ship dark + light + a
  calmer "low-neon" variant. User-overridable (the Linux half: it's *your* desktop).
- Reduce-motion and high-contrast modes; full keyboard navigation; scalable text.

---

## 4. What just landed (first iteration)

Concrete steps toward the vision, shipped alongside this doc:

- **Command Palette (⌘K)** — Spotlight-style fuzzy launcher; the keyboard-complete spine (see §3.1).
- **Resize & close actually work** — fixed a coordinate-space bug where hit zones were
  offset from where handles/buttons were drawn (see DEVELOPMENT-LOG).
- **Resize cursors** appear when hovering a selected object's corner (affordance).
- **Fit to content** (`F` or the dock button) frames the whole dimension — the seed of the
  fluid-camera work.
- **Create workspaces (Dimensions)** from the top bar — the spatial model is now expandable
  from the UI, not just code.
- **Dock no longer occludes** the minimap/zoom overlay (panel ordering fix).

---

## 5. Non-goals (for now)

- Pixel-faithful macOS cloning — we borrow its *qualities*, not its look.
- Overlapping stacked windows as the primary model — the canvas is the model.
- Cloud-required features — local-first stays the default.
