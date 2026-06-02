# Smart Objects

Smart Objects are typed entities on the Hyperspace canvas. They are first-class citizens — not just files, windows, or icons.

**Implementation:** `crates/hyperspace-core/src/object.rs` (kinds + `SmartObject` incl. `Link`)  
**Rendering & interaction (desktop prototype):** `crates/hyperspace-shell/src/canvas.rs`, `app.rs` (basic creation, move, inspector; double-click always spawns Note; HUD spawn omits Link today)

---

## Object kinds

| Kind | Color (accent) | Default size | Purpose |
|------|----------------|--------------|---------|
| **Note** | Amber | 280×180 | Freeform text, reminders, instructions |
| **App** | Blue | 240×160 | Launches or embeds applications (future) |
| **Folder** | Green | 220×140 | Maps to Hyperspace FS paths (future) |
| **Agent** | Purple | 260×190 | Local-first AI agent entry point |
| **Link** | Red | 200×120 | Cross-dimension or external links (next target; kind defined + partial UI support today) |

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
| Click empty canvas | Deselect |
| Drag object | Move; snaps to 20px grid |
| Double-click empty canvas | Spawn new **Note** at cursor |
| Spawn buttons (left HUD) | Create object at world origin (0, 0) |
| Inspector edits | Update title/body; marks workspace dirty |
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

### Link (next documented target)

- Points to another dimension, object, or URL
- Click navigates or opens target
- Current: `ObjectKind::Link` exists in core and can be created via code paths / inspector; no cross-dimension navigation, no special Link UI, no spawn button in left HUD yet (see [PHASES.md](PHASES.md) Phase 2 todo and "What to build next")

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
