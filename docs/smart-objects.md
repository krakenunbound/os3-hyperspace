# Smart Objects

Smart Objects are typed entities on the Hyperspace canvas. They are first-class citizens — not just files, windows, or icons.

**Implementation:** `crates/hyperspace-core/src/object.rs` (kinds + `SmartObject` incl. `link_target`)  
**Rendering & interaction (desktop prototype):** `crates/hyperspace-shell/src/canvas.rs` (full resize + LinkActivate), `app.rs` (inspector + events + spawn + pending nav)

---

## Object kinds

| Kind | Color (accent) | Default size | Purpose |
|------|----------------|--------------|---------|
| **Note** | Amber | 280×180 | Freeform text, reminders, instructions |
| **App** | Blue | 240×160 | Launches or embeds applications (future) |
| **Folder** | Green | 220×140 | Maps to Hyperspace FS paths (future) |
| **Agent** | Purple | 260×190 | Local-first AI agent entry point |
| **Link** | Red | 200×120 | Cross-dimension links (basic: target, spawn, inspector setter, click-nav, demo prewire; see impl notes below) |

Each object has:

- `id` — stable UUID
- `title` — display name
- `body` — multiline content
- `position` — world coordinates (`WorldPoint`)
- `size` — width/height in world units (`WorldSize`)

---

## Interaction (shell prototype)

| Action | Result |
|--------|--------|
| Click object | Select; opens **Inspector** (right panel) |
| Click Agent | Select + invoke stub AI runtime |
| Click Link | Select + if target set, cross-dimension navigation |
| Click empty canvas | Deselect |
| Drag object | Move; snaps to 20px grid |
| Drag corner (selected) | Resize; snaps size lightly to grid |
| Canvas background | Dynamic layered starfield (zoom/pan reactive "hyperspace" depth for immersion) |
| Double-click empty canvas | Spawn new **Note** at cursor |
| Spawn buttons (left HUD) | Create object at world origin (0, 0) — now includes Link |
| Inspector edits | Update title/body/size; for Link set target dim; marks workspace dirty |
| Delete key | Remove selected object (when not typing) |
| Delete button (Inspector) | Remove selected object |

Selected objects render with a white border and brighter fill.

---

## Future behavior (not yet implemented)

### App

- Double-click or explicit "Open" launches a Redox orbital app or host process
- Embedded preview for supported app types

### Folder

- Body shows path summary (`hs://<dimension-id>/projects/...`)
- Double-click navigates into folder contents on canvas or in a file view

### Agent

- Persistent conversation thread per agent object
- Runs inference locally via `hyperspace-ai` (no cloud required)
- Can act on other Smart Objects in the same dimension
- Visual liveness: Inner glow + bright neural core to signal "alive" AI (part of making objects feel smart and immersive toward "best OS").

### Link

- Points to another dimension (or future object/URL)
- Click navigates/activates when target set
- Visual: Renders as glowing portal/wormhole (concentric rings + core, per Link accent) for "best OS" magical multidimensional feel. Basic cross-dim linking implemented (target picker in Inspector, HUD spawn, click-to-nav if set, demo pre-linked example, stored in persistence). Next polish: actual drawn connections/lines to targets, richer targets.

---

## Serialization

Smart Objects serialize as part of `HyperspaceState` — see [persistence.md](persistence.md).

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "kind": "Note",
  "title": "Welcome to OS/3 Hyperspace",
  "body": "Scroll to zoom...",
  "position": { "x": 120.0, "y": 140.0 },
  "size": { "width": 280.0, "height": 180.0 }
}
```
