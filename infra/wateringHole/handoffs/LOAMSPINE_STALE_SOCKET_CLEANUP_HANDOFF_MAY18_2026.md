<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Stale Socket Cleanup Response

**Date**: May 18, 2026
**From**: loamSpine team
**To**: primalSpring, wetSpring, downstream consumers
**Re**: Stale socket detection + cleanup (HIGH priority upstream ask)

---

## Audit checklist

| Item | Status | Details |
|------|--------|---------|
| `unlink()` before `bind()` | **PASS** | Unconditional `remove_file` with `NotFound` silenced (TOCTOU-safe). `uds.rs:99-111`. |
| Clean up on shutdown | **PASS** | `Drop` impl removes socket; binary removes symlinks + PID file. `uds.rs:51-54`, `main.rs:414-427`. |
| PID file | **NEW** | `loamspine.pid` written alongside socket after bind. Consumers can `kill(pid, 0)` for instant liveness. |

---

## What changed

### TOCTOU-safe pre-bind cleanup

Before (had TOCTOU race):
```rust
if path.exists() {
    std::fs::remove_file(&path)?;
}
```

After (unconditional, race-free):
```rust
match std::fs::remove_file(&path) {
    Ok(()) => debug!("Removed stale socket"),
    Err(e) if e.kind() == NotFound => {}
    Err(e) => return Err(e),
}
```

### PID file

On UDS bind success, `loamspine.pid` is written to the same directory as the
socket. Contains the PID as a decimal string. Consumers can check
`kill(pid, 0)` (no signal sent) for O(1) liveness without connect overhead.

File is removed on graceful shutdown alongside the socket and symlinks.

---

## Socket lifecycle summary

| Phase | Behavior |
|-------|----------|
| Startup | `create_dir_all(parent)` → `remove_file(path)` (ignore NotFound) → `UnixListener::bind` → write `loamspine.pid` |
| Running | Accept loop with BTSP/JSON-RPC dispatch |
| Shutdown | `stop()` → accept loop exits → `Drop` removes socket → binary removes symlinks + PID file |
| Crash | Socket persists (no guaranteed unlink); PID file persists but stale PID detectable via `kill(pid, 0)` |

---

## Files touched

- `crates/loam-spine-api/src/jsonrpc/uds.rs` — TOCTOU-safe pre-bind cleanup
- `bin/loamspine-service/main.rs` — PID file write after bind, cleanup on shutdown
