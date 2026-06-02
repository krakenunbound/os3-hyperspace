# Workspace Persistence

The shell prototype saves your full workspace — all dimensions, viewports, and Smart Objects — to a JSON file on disk.

**Implementation:** `crates/hyperspace-fs/src/file.rs` (`JsonWorkspaceStore`)

---

## File locations

| OS | Path |
|----|------|
| **Windows** | `%APPDATA%\os3-hyperspace\workspace.json` |
| **macOS** | `~/Library/Application Support/os3-hyperspace/workspace.json` |
| **Linux** | `~/.local/share/os3-hyperspace/workspace.json` |

If the file is missing or invalid, the shell starts with demo content (Home + Work dimensions).

---

## Save triggers

| Trigger | Behavior |
|---------|----------|
| **Startup** | Load existing file, or demo content |
| **Ctrl+S** / **Save** button | Write immediately |
| **Exit with unsaved changes** | Auto-save on quit |
| **Dirty indicator** | Orange ● in top bar when changes exist since last save |

The in-memory `ObjectStore` syncs on explicit "Sync to in-memory store" and after canvas edits that mark the workspace dirty.

---

## JSON schema

Top-level structure (`HyperspaceState`):

```json
{
  "dimensions": [ /* Dimension[] */ ],
  "active_dimension": "uuid-of-active-dimension"
}
```

Each **Dimension**:

```json
{
  "id": "uuid",
  "name": "Home",
  "viewport": {
    "pan_x": 0.0,
    "pan_y": 0.0,
    "zoom": 1.0
  },
  "objects": [ /* SmartObject[] — see smart-objects.md */ ]
}
```

There is no explicit schema version field yet. Breaking changes will add a `version` key when Redox-native storage lands (or when object linking / new Smart Object fields land in Phase 2).

---

## Future: Hyperspace FS on Redox

The JSON store is a **development stand-in**. Target architecture:

```
hs://<dimension-id>/<path>/object-id
```

- Dimensions mount as navigable namespaces
- Smart Objects may map to files, symlinks, or composite records
- `JsonWorkspaceStore` remains for export/import and desktop dev builds

See [ARCHITECTURE.md](../ARCHITECTURE.md) and [PHASES.md](PHASES.md) Phase 1.

Persistence will evolve with next targets (object linking may add link metadata; Redox replaces the JSON stand-in).

---

## Manual backup

Copy your workspace file before risky experiments:

```powershell
Copy-Item "$env:APPDATA\os3-hyperspace\workspace.json" "$env:USERPROFILE\Desktop\hyperspace-backup.json"
```

Restore by placing the file back at the default path and restarting the shell.
