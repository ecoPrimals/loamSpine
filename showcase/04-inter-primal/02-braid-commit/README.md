# 🦴 Demo: Braid Commit (Multi-Entry Session)

**Goal**: Demonstrate multi-entry sessions with Merkle tree construction  
**Time**: 5 minutes  
**Prerequisites**: LoamSpine built (`cargo build`)

---

## 🎯 What You'll Learn

- Creating sessions with multiple entries (braids)
- Committing entries in sequence
- Merkle tree construction from multiple hashes
- Cryptographic proof of entry inclusion and order
- Session integrity verification

---

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Braiding Concept

A **braid** is a session with multiple entries woven together:

```
Entry 1 → Entry 2 → Entry 3 → Entry 4
   ↓         ↓         ↓         ↓
   └─────────┴─────────┴─────────┘
                  │
            Merkle Root
    (cryptographic proof)
```

### 2. Merkle Tree Construction

With 4 entries, LoamSpine builds this tree:

```
            Merkle Root
             /        \
            /          \
       Hash(1,2)    Hash(3,4)
        /    \        /    \
       /      \      /      \
    Entry1  Entry2  Entry3  Entry4
```

**Properties**:
- Any change to any entry changes the root
- Root proves all entries + their order
- Efficient proofs (O(log n))

### 3. Entry Types

This demo commits 4 different entry types:
1. **User action**: User interaction event
2. **System response**: System validation/processing
3. **State change**: Application state update
4. **Final result**: Completion status

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine: Braid Commit Demo (Multi-Entry)
================================================================

Step 1: Preparing environment...
✓ Environment ready

Step 2: Starting LoamSpine server...
✓ LoamSpine running (PID: 12345)

Step 3: Creating braid session (4 entries expected)...
✓ Braid session created
   Session ID: 550e8400-e29b-41d4-a716-446655440000
   Expected entries: 4

Step 4: Committing multiple entries to braid...
   Entry 1/4: User action
   ✓ Entry 1 committed: a1b2c3d4e5f6...
   Entry 2/4: System response
   ✓ Entry 2 committed: b2c3d4e5f6a1...
   Entry 3/4: State change
   ✓ Entry 3 committed: c3d4e5f6a1b2...
   Entry 4/4: Final result
   ✓ Entry 4 committed: d4e5f6a1b2c3...

✓ All 4 entries committed to braid

Step 5: Finalizing braid (computing Merkle tree)...
✓ Braid finalized
   Merkle Root: f9e8d7c6b5a43210...
   Entry Count: 4

              MERKLE TREE STRUCTURE
                    
                  Merkle Root
              f9e8d7c6b5a43210...
                    /    \
                   /      \
              Hash(1,2)  Hash(3,4)
               /    \      /    \
              /      \    /      \
          Entry1  Entry2 Entry3  Entry4
          a1b2c3d4 b2c3d4e5 c3d4e5f6 d4e5f6a1

Step 6: Verifying braid integrity...
   Retrieving all entries and verifying...
   ✓ Entry 1 verified
   ✓ Entry 2 verified
   ✓ Entry 3 verified
   ✓ Entry 4 verified

✓ Braid integrity verified

================================================================
  Demo Complete!
================================================================

Why Braiding Matters:
  - Multiple entries form a cryptographic proof
  - Merkle root proves all entries + order
  - Any change breaks the root hash
  - Efficient verification (log n proofs)
```

---

## 🎓 Learning Points

### Pattern 1: Multi-Entry Session Creation

```bash
# Manifest declares expected entry count
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "create_session",
    "params": {
      "manifest": {
        "expected_entries": 4,
        "metadata": {
          "demo": "braid-commit",
          "description": "Multi-entry session"
        }
      }
    },
    "id": 1
  }'
```

**Why declare entry count?**:
- Pre-allocates Merkle tree structure
- Validates completeness on finalization
- Detects missing or extra entries

### Pattern 2: Sequential Entry Commits

```bash
# Commit entries in sequence
for i in {1..4}; do
  curl -X POST http://localhost:8080/rpc \
    -H "Content-Type: application/json" \
    -d '{
      "jsonrpc": "2.0",
      "method": "commit_entry",
      "params": {
        "session_id": "'$SESSION_ID'",
        "entry": {
          "hash": "'$(compute_hash $i)'",
          "content_type": "application/json",
          "data": "'$(encode_data $i)'",
          "metadata": {
            "sequence": '$i'
          }
        }
      },
      "id": '$((i+1))'
    }'
done
```

**Best practice**: Use `sequence` metadata to track order

### Pattern 3: Merkle Tree Finalization

```bash
# After all entries committed, finalize
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "finalize_session",
    "params": {
      "session_id": "'$SESSION_ID'"
    },
    "id": 10
  }'

# Response includes Merkle root and count
{
  "jsonrpc": "2.0",
  "result": {
    "merkle_root": "f9e8d7c6b5a43210...",
    "entry_count": 4
  },
  "id": 10
}
```

**Validation**: LoamSpine checks that `entry_count` matches `expected_entries`

---

## 💡 Key Concepts

### Merkle Proofs

To prove Entry 2 is in the braid, you only need:
```
Entry 2 hash
  +
Entry 1 hash (sibling)
  +
Hash(3,4) (parent's sibling)
  =
Can recompute Merkle Root
```

**Efficiency**: Prove any entry with log₂(n) hashes
- 4 entries → 2 hashes needed
- 1024 entries → 10 hashes needed
- 1 million entries → 20 hashes needed

### Content-Addressed Braiding

Each entry is identified by its content hash:
```rust
// Content determines hash
let content = b"User clicked button";
let hash = sha256(content);

// Same content = same hash (deduplication)
// Different content = different hash (integrity)
```

### Cryptographic Guarantees

The Merkle root provides:
1. **Inclusion**: Proves entry is in the braid
2. **Order**: Proves entry's position
3. **Integrity**: Detects any tampering
4. **Completeness**: Proves all entries present

---

## 📊 Use Cases

### Use Case 1: Audit Trail

```
Action Log Braid:
  Entry 1: User login
  Entry 2: Permission check
  Entry 3: Data access
  Entry 4: Logout

Merkle root proves:
  - All actions occurred
  - Actions in this order
  - No actions omitted
```

### Use Case 2: Multi-Party Transaction

```
Transaction Braid:
  Entry 1: Initiator proposes
  Entry 2: Reviewer approves
  Entry 3: System validates
  Entry 4: Transaction commits

Merkle root proves:
  - All parties participated
  - Correct sequence followed
  - No steps skipped
```

### Use Case 3: Distributed Computation

```
Computation Braid:
  Entry 1: Task assigned
  Entry 2: Intermediate result 1
  Entry 3: Intermediate result 2
  Entry 4: Final computation

Merkle root proves:
  - All steps completed
  - Results are authentic
  - Computation is verifiable
```

---

## 🔍 Advanced: Merkle Proof Verification

To verify Entry 2 without downloading the whole braid:

```bash
# Get Merkle proof for Entry 2
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "get_merkle_proof",
    "params": {
      "session_id": "'$SESSION_ID'",
      "entry_hash": "'$ENTRY_2_HASH'"
    },
    "id": 11
  }'

# Response includes proof path
{
  "jsonrpc": "2.0",
  "result": {
    "proof": [
      "a1b2c3d4...",  // Entry 1 hash (sibling)
      "c3d4e5f6..."   // Hash(3,4) (parent's sibling)
    ],
    "position": 1  // Entry 2 is at index 1
  },
  "id": 11
}

# Verify locally
merkle_root_computed = hash(
    hash(Entry1Hash, Entry2Hash),  // Left subtree
    Hash34                          // Right subtree
)

assert merkle_root_computed == merkle_root_from_session
```

---

## 🔧 Troubleshooting

### Error: "Entry count mismatch"

**Cause**: Finalized before committing all expected entries

**Solution**:
```bash
# Check manifest
echo "Expected: $EXPECTED_ENTRIES"
echo "Committed: $ACTUAL_ENTRIES"

# Commit remaining entries before finalizing
```

### Error: "Merkle computation failed"

**Cause**: Invalid hash or corrupted entry

**Solution**:
```bash
# Recompute hash for each entry
for content in "${CONTENTS[@]}"; do
    echo -n "$content" | sha256sum
done

# Verify each entry was committed successfully
```

---

## 📈 Performance Characteristics

| Entries | Merkle Depth | Proof Size | Verification Time |
|---------|--------------|------------|-------------------|
| 4       | 2            | 2 hashes   | ~1ms             |
| 16      | 4            | 4 hashes   | ~2ms             |
| 256     | 8            | 8 hashes   | ~4ms             |
| 4,096   | 12           | 12 hashes  | ~6ms             |

**Scalability**: Logarithmic proof size and verification time

---

## ➡️ Next Steps

After completing this demo:
- **Next Demo**: `03-signing-capability` — Integration with BearDog for signed entries
- **Deep Dive**: `../../specs/DATA_MODEL.md` — Merkle tree implementation
- **Advanced**: `../../specs/PROOFS.md` — Merkle proof generation and verification

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ Session created with 4 expected entries
- ✅ All 4 entries committed successfully
- ✅ Merkle root computed on finalization
- ✅ All entries verified through Merkle proofs
- ✅ Entry count matches expected count

---

**Updated**: December 25, 2025  
**Principle**: Cryptographic proofs for multi-entry integrity

🦴 **LoamSpine: Braiding verifiable history**
