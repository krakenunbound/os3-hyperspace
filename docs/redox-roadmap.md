# Redox Roadmap

OS/3 Hyperspace targets **Redox OS** as its microkernel foundation. The current eframe/egui prototype runs on Windows/macOS/Linux so shell UX can iterate before Redox integration.

Redox VM setup and relevant forks are documented next targets (see [PHASES.md](PHASES.md) "What to build next" and Phase 0/1 todos).

**Phase:** 0 → 1 transition (VM setup + forks are next documented targets alongside object linking and resize)

---

## Why Redox

- Written in **Rust** — same language as Hyperspace crates
- **Microkernel** design — isolation, security, stability
- **Open source** — no vendor lock-in
- Active community and documented build process

---

## Step 1: Redox in a VM (Phase 0)

### Prerequisites

- VirtualBox, VMware, or QEMU
- ~20 GB disk, 4+ GB RAM for VM
- Rust toolchain (already installed for Hyperspace)

### Getting Redox running

1. Follow [Redox build instructions](https://doc.redox-os.org/book/building-redox.html)
2. Build the ISO or use a published image
3. Boot in VM; confirm desktop (Orbital) loads
4. Document your host-specific setup in this file when done

### Success check

- [ ] Redox boots to desktop in VM
- [ ] Network works (optional, for git fetch)
- [ ] Can copy files host ↔ VM

---

## Step 2: Identify forks (Phase 0–1)

Likely Redox components to fork or extend:

| Component | Hyperspace use |
|-----------|----------------|
| **kernel** | Hardening, Hyperspace-specific syscalls (if needed) |
| **orbital** | Native windowing — replace eframe layer |
| **redoxfs** / storage | Backing store for Hyperspace FS |
| **libextra** / IPC patterns | Shell ↔ service communication |

Not all forks are needed on day one. Start with orbital + a Hyperspace daemon crate.

---

## Step 3: Port crates to Redox (Phase 1–2)

Migration order:

1. **`hyperspace-core`** — already `#![no_std]`-friendly types; verify on Redox target
2. **`hyperspace-fs`** — add Redox-backed `ObjectStore` impl
3. **`hyperspace-ai`** — run inference in a Redox userland service
4. **`hyperspace-shell`** — new binary using Orbital instead of eframe

The desktop prototype stays in-repo for fast iteration. Same core types, different window backend.

```
Desktop (now)          Redox (target)
─────────────────      ─────────────────
eframe / egui    →     Orbital toolkit
JsonWorkspaceStore →   Hyperspace FS daemon
in-process agent   →   IPC to ai-runtime service
```

---

## Step 4: CI and integration tests (Phase 4)

- Redox VM in CI (QEMU headless) for smoke tests
- Boot → launch shell → create object → persist → reboot → verify

---

## Host development loop (until Redox shell exists)

Continue daily work on Windows:

```powershell
cargo run -p hyperspace-shell   # UX iteration
cargo test --workspace          # core + fs tests
```

Parallel track: Redox VM for kernel/service experiments.

See [PHASES.md](PHASES.md) for checklist items tied to each phase.
