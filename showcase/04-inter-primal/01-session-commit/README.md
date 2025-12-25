# 🦴 Demo: Session Commit (Local Single-Entry)

**Goal**: Demonstrate basic session creation and single entry commit  
**Time**: 3 minutes  
**Prerequisites**: LoamSpine built (`cargo build`)

---

## 🎯 What You'll Learn

- Creating a session with LoamSpine
- Committing an entry to a session
- Verifying entries through their hash
- Finalizing sessions with Merkle proofs
- Real JSON-RPC interaction (no mocks)

---

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Session Lifecycle

```
CREATE → COMMIT → VERIFY → FINALIZE
   │        │        │         │
   ↓        ↓        ↓         ↓
Session   Entry    Hash    Merkle Root
  ID      Hash   Verified   Computed
```

### 2. Entry Structure

Each entry contains:
- **Hash**: SHA-256 of the content
- **Content Type**: MIME type (e.g., `text/plain`)
- **Data**: Base64-encoded content
- **Metadata**: Custom key-value pairs

### 3. JSON-RPC Interaction

All operations use JSON-RPC 2.0:
- `create_session` → Session ID
- `commit_entry` → Entry hash
- `get_entry` → Entry data
- `finalize_session` → Merkle root

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine: Session Commit Demo
================================================================

Step 1: Preparing environment...
✓ Environment ready

Step 2: Starting LoamSpine server...
✓ LoamSpine running (PID: 12345)
   TARP endpoint: http://localhost:9001
   JSON-RPC endpoint: http://localhost:8080

Step 3: Creating session...
✓ Session created
   Session ID: 550e8400-e29b-41d4-a716-446655440000

Step 4: Committing entry to session...
✓ Entry committed
   Entry Hash: a1b2c3d4e5f67890...
   Content: Hello from LoamSpine Session Demo!

Step 5: Verifying entry...
✓ Entry verified successfully
   Retrieved: Hello from LoamSpine Session Demo!

Step 6: Finalizing session...
✓ Session finalized
   Merkle Root: f9e8d7c6b5a43210...

================================================================
  Demo Complete!
================================================================

What we demonstrated:
  ✅ Created a session with LoamSpine
  ✅ Committed a single entry with metadata
  ✅ Verified the entry through its hash
  ✅ Finalized the session with Merkle root
  ✅ Used real JSON-RPC calls (no mocks)

Next steps:
  - Try: 02-braid-commit (multi-entry session)
  - Learn: How braiding works with multiple entries
```

---

## 🎓 Learning Points

### Pattern 1: Session Creation

```bash
# JSON-RPC call
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "create_session",
    "params": {
      "manifest": {
        "expected_entries": 1,
        "metadata": {
          "demo": "session-commit",
          "timestamp": "2025-12-25T10:30:00Z"
        }
      }
    },
    "id": 1
  }'

# Response
{
  "jsonrpc": "2.0",
  "result": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "id": 1
}
```

### Pattern 2: Entry Commit

```bash
# Prepare content
CONTENT="Hello from LoamSpine!"
HASH=$(echo -n "$CONTENT" | sha256sum | awk '{print $1}')
DATA=$(echo -n "$CONTENT" | base64)

# JSON-RPC call
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "commit_entry",
    "params": {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "entry": {
        "hash": "'$HASH'",
        "content_type": "text/plain",
        "data": "'$DATA'",
        "metadata": {
          "author": "demo-user"
        }
      }
    },
    "id": 2
  }'
```

### Pattern 3: Entry Verification

```bash
# Retrieve entry
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "get_entry",
    "params": {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "entry_hash": "a1b2c3d4e5f67890..."
    },
    "id": 3
  }'

# Decode and verify
echo "$RESPONSE" | jq -r '.result.entry.data' | base64 -d
# Output: Hello from LoamSpine!
```

### Pattern 4: Session Finalization

```bash
# Finalize (computes Merkle root)
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "finalize_session",
    "params": {
      "session_id": "550e8400-e29b-41d4-a716-446655440000"
    },
    "id": 4
  }'

# Response includes Merkle root
{
  "jsonrpc": "2.0",
  "result": {
    "merkle_root": "f9e8d7c6b5a43210...",
    "entry_count": 1
  },
  "id": 4
}
```

---

## 💡 Key Concepts

### Session Manifest

The manifest declares session properties:
```json
{
  "expected_entries": 1,      // How many entries expected
  "metadata": {               // Custom metadata
    "demo": "session-commit",
    "timestamp": "2025-12-25T10:30:00Z"
  }
}
```

### Content Hashing

LoamSpine uses SHA-256 for content addressing:
```rust
// Simplified
let content = b"Hello from LoamSpine!";
let hash = sha256(content);
// hash: a1b2c3d4e5f67890...
```

**Why?**: Content-addressed storage ensures:
- Deduplication (same content = same hash)
- Integrity (tampered content changes hash)
- Verification (anyone can recompute hash)

### Merkle Trees

When a session is finalized, LoamSpine computes a Merkle tree:

```
        Merkle Root
           /    \
          /      \
     Hash(AB)  Hash(CD)
       /  \      /  \
      A    B    C    D
   (entries in session)
```

**Benefits**:
- Compact proof of all entries
- Efficient verification
- Tamper-evident

---

## 📊 Storage Internals

After this demo, check the storage directory:

```bash
ls -l /tmp/loamspine-demo-session/

# You'll see:
sessions/550e8400-e29b-41d4-a716-446655440000/
  ├── manifest.json       # Session metadata
  ├── entries/
  │   └── a1b2c3d4...    # Entry content
  └── merkle.json        # Merkle tree structure
```

**Storage Format**: CBOR (Compact Binary Object Representation)
- More efficient than JSON
- Preserves types
- Smaller on disk

---

## 🔧 Troubleshooting

### Error: "Failed to create session"

**Cause**: LoamSpine not running or port conflict

**Solution**:
```bash
# Check if LoamSpine is running
ps aux | grep loam-spine-cli

# Check if port is in use
lsof -i :8080

# Kill existing instance
pkill -f loam-spine-cli
```

### Error: "Failed to commit entry"

**Cause**: Invalid session ID or hash mismatch

**Solution**:
```bash
# Verify session ID
echo $SESSION_ID

# Recompute hash
echo -n "Hello from LoamSpine!" | sha256sum
```

### Error: "Entry verification failed"

**Cause**: Content was tampered or encoding issue

**Solution**:
- Ensure base64 encoding is correct
- Check that hash matches content
- Verify no trailing newlines in content

---

## ➡️ Next Steps

After completing this demo:
- **Next Demo**: `02-braid-commit` — Multi-entry sessions with braiding
- **Deep Dive**: `../../specs/DATA_MODEL.md` — Session and entry structure
- **Advanced**: `03-signing-capability` — Integration with BearDog signing

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ Session created with valid UUID
- ✅ Entry committed with hash
- ✅ Entry retrieved successfully
- ✅ Merkle root computed on finalization
- ✅ Storage directory contains session files

---

**Updated**: December 25, 2025  
**Principle**: Real capabilities, no mocks

🦴 **LoamSpine: Persistent, verifiable ledger**
