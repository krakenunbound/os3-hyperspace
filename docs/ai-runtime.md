# AI Runtime

OS/3 Hyperspace is **AI-native** with a **local-first** policy: agents run on your machine, not in the cloud, unless you explicitly opt in.

**Implementation:** `crates/hyperspace-ai/`

Local inference behind the `AgentRuntime` trait is one of the next documented targets (see [PHASES.md](PHASES.md)).

---

## Current state (Phase 1 stub; real local inference pending as documented target)

The shell ships with a deterministic stub model for UI development:

```rust
pub trait AgentRuntime {
    fn ping(&self) -> Result<String>;
    fn complete(&self, message: AgentMessage) -> Result<AgentReply>;
}
```

`LocalAgentRuntime` uses `StubModel` — canned responses that confirm the hook is wired. No inference engine is bundled yet.

### Shell integration

| UI action | Behavior |
|-----------|----------|
| **Ping local agent** (left HUD) | Returns stub online message |
| **Ask about this dimension** | Stub describes the active dimension name |
| **Click Agent Smart Object** | Stub responds about that object |

---

## Target state (Phase 1+; local inference is a next documented target)

```
┌─────────────────────────────────────┐
│  hyperspace-shell                   │
│    Agent Smart Objects              │
├─────────────────────────────────────┤
│  hyperspace-ai                      │
│    AgentRuntime trait               │
│    ├─ StubModel (dev/tests)         │
│    └─ LocalModel (llama.cpp, etc.)  │
├─────────────────────────────────────┤
│  Redox service (future)             │
│    sandboxed inference process      │
└─────────────────────────────────────┘
```

Planned properties:

- **Local by default** — model weights on disk, no network required
- **Per-agent context** — each Agent object maintains its own thread
- **Object-aware** — agents can read metadata from Smart Objects in the same dimension (with permission)
- **Replaceable backend** — swap stub for llama.cpp, ONNX, or Redox-native runtime behind the same trait

---

## Adding a real backend (sketch)

1. Implement `AgentRuntime` in a new module (e.g. `local_llm.rs`)
2. Wire model path via config or env var
3. Swap `LocalAgentRuntime::new()` in `hyperspace-shell` to use the new backend
4. Keep `StubModel` for CI and offline UI tests

No API changes to the shell should be required — the trait boundary exists for this migration.

---

## Privacy principles

- Agent objects never send data off-device unless a future "cloud bridge" is explicitly enabled
- Conversation history stored with workspace / Hyperspace FS, user-owned
- Agent invoke is always user-initiated in the prototype (no background polling)
