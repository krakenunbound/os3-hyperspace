# TODO

Short checklist — see **[docs/PHASES.md](docs/PHASES.md)** for full phase details and notes.

## Phase 0: Setup

- [ ] Spin up Redox desktop in VM → [redox-roadmap.md](docs/redox-roadmap.md)
- [ ] Fork Redox relevant repos
- [ ] Set up Redox dev environment
- [x] Clone repo + Rust workspace on Windows
- [x] Runnable shell prototype (`hyperspace-shell`)
- [x] Windows dev docs + script

## Phase 1: Foundation

- [ ] Harden kernel (Redox fork)
- [ ] Hyperspace FS on Redox (replace JSON-only persistence)
- [ ] Local inference behind `AgentRuntime` → [ai-runtime.md](docs/ai-runtime.md)
- [x] Core types crate (`hyperspace-core`)
- [x] In-memory object store (`hyperspace-fs`)
- [x] JSON workspace persistence → [persistence.md](docs/persistence.md)
- [x] Agent runtime trait + stub (`hyperspace-ai`)

## Phase 2: Shell MVP

- [x] Infinite zoomable canvas
- [x] Dimension switching + create new
- [x] Smart Objects → [smart-objects.md](docs/smart-objects.md)
- [x] Selection + inspector panel
- [x] Grid snap + minimap
- [x] Persistent layout restore
- [ ] Object linking across dimensions
- [ ] Resize Smart Objects
- [ ] Native Redox orbital shell

## Phase 3+: Compatibility, Polish, Release

- [ ] App/Folder Smart Objects launch real programs/paths
- [ ] Redox VM integration tests
- [ ] Theme system + accessibility pass
- [ ] Installer / ISO pipeline

---

**Docs convention:** When something ships, update in this order:  
`docs/PHASES.md` + `TODO.md` (check off) → relevant concept doc (smart-objects.md etc.) → `ARCHITECTURE.md` (if boundaries change) → `docs/dev-windows.md` (if controls change).  

See [docs/README.md](docs/README.md) and [docs/PHASES.md](docs/PHASES.md) for the authoritative structure and tracker.

Contribute via PRs!
