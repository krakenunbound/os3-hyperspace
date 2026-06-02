# OS/3 Hyperspace

The AI-native, multidimensional operating system. Spiritual successor to OS/2 Warp.

Built on [Redox OS](https://www.redox-os.org/) (Rust microkernel) for maximum stability, security, and openness.

## Vision

- Infinite zoomable workspaces (**Dimensions**)
- **Smart Objects** — typed entities on the canvas, not just files or windows
- **Local-first AI agents** — on-device by default
- Better than Windows 11 / macOS in usability, robustness, and freedom

## Project status

| Phase | Focus | Status |
|-------|-------|--------|
| **0** | Setup + desktop prototype | ✅ Prototype running on Windows; Redox VM + forks pending |
| **1** | Core services + Redox foundation | 🟡 JSON persistence + AI stub done; Redox / local inference pending |
| **2** | Shell MVP | 🟡 Resize handles + basic Link navigation landed (see docs/DEVELOPMENT-LOG.md + PHASES.md); more polish + native Redox pending |
| **3+** | Compatibility, polish, release | ⚪ Not started |

Full tracker: **[docs/PHASES.md](docs/PHASES.md)**

## Quick start (Windows)

```powershell
git clone https://github.com/krakenunbound/os3-hyperspace.git
cd os3-hyperspace
cargo run -p hyperspace-shell
```

Controls, persistence paths, and dev scripts: **[docs/dev-windows.md](docs/dev-windows.md)**

## What's in the repo

```
crates/
  hyperspace-core/    Dimensions, viewport math, Smart Object types
  hyperspace-shell/   Infinite-canvas UI prototype (eframe/egui)
  hyperspace-ai/      Local agent runtime trait + stub model
  hyperspace-fs/      Object store + JSON workspace persistence
docs/                 Phase tracker, architecture notes, concept specs
scripts/dev.ps1       run | build | check | test
```

## Documentation

See **[docs/README.md](docs/README.md)** for the full index (phase tracker, concept specs, dev guide).

Quick links:

| Doc | Description |
|-----|-------------|
| [docs/README.md](docs/README.md) | Documentation index + full structure |
| [docs/PHASES.md](docs/PHASES.md) | Phase tracker (source of truth) — status tables, exit criteria, next order |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Target vs. current layers, crate map, data flow |
| [docs/smart-objects.md](docs/smart-objects.md) | Smart Object types, interaction, future behavior |
| [docs/persistence.md](docs/persistence.md) | JSON schema, paths, save triggers |
| [docs/ai-runtime.md](docs/ai-runtime.md) | Stub today, local inference plan |
| [docs/redox-roadmap.md](docs/redox-roadmap.md) | VM setup, forks, migration path |
| [TODO.md](TODO.md) | Short checklist linking to phases + concepts |
| [docs/dev-windows.md](docs/dev-windows.md) | UI layout, controls, troubleshooting (Windows) |

**Update convention (when something ships):**  
1. `docs/PHASES.md` + `TODO.md` — check off items  
2. Relevant concept doc  
3. `ARCHITECTURE.md` if crate boundaries change  
4. `dev-windows.md` if controls or workflow change

## License

MIT — see [LICENSE](LICENSE).
