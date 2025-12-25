# 🦴🐻 Demo: Signing Capability (LoamSpine + BearDog)

**Goal**: Demonstrate inter-primal signing integration  
**Time**: 5 minutes  
**Prerequisites**: BearDog CLI signer at `../bins/beardog-cli-signer`

---

## 🎯 What You'll Learn

- Inter-primal communication (LoamSpine ↔ BearDog)
- Signing entries with BearDog's cryptographic capabilities
- Storing signed entries in LoamSpine
- Verifying signatures with public keys
- Capability-based discovery (conceptual)

---

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Inter-Primal Architecture

```
┌─────────────┐         ┌─────────────┐
│  LoamSpine  │────────▶│   BearDog   │
│   (Ledger)  │  Sign   │  (Signer)   │
│             │ Request │             │
└─────────────┘         └─────────────┘
      │                        │
      │  Hash: a1b2c3d4...    │
      │─────────────────────▶│
      │                        │
      │         ◀──────────────┤
      │  Signature: f9e8d7...  │
      │                        │
      ↓                        
Commit signed entry
(hash + signature + public key)
```

### 2. Signing Workflow

```
1. Create Entry → 2. Compute Hash → 3. Sign with BearDog
                                            ↓
                                   4. Commit to LoamSpine
                                            ↓
                              5. Verify Signature Independently
```

### 3. Signed vs Unsigned Entries

**Signed Entry** (critical operations):
- Financial transactions
- Permission changes
- Audit events
- Requires BearDog signature

**Unsigned Entry** (non-critical):
- Read queries
- Logs
- Metadata
- No signature needed

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine + 🐻 BearDog: Signing Capability Demo
================================================================

Step 1: Checking prerequisites...
✓ BearDog CLI signer found

Step 2: Preparing environment...
✓ Environment ready

Step 3: Generating BearDog keypair...
✓ Keypair generated
   Key path: /tmp/beardog-demo-keys/demo-key.pem
   Public key: ed25519:a1b2c3d4e5f6...

Step 4: Starting LoamSpine server...
✓ LoamSpine running (PID: 12345)

Step 5: Creating session for signed entries...
✓ Session created
   Session ID: 550e8400-e29b-41d4-a716-446655440000

Step 6: Signing entry with BearDog...
   Content: {"action":"transfer","amount":100,"from":"alice","to":"bob"}
   Hash: a1b2c3d4e5f67890...
   Requesting signature from BearDog...
✓ Entry signed by BearDog
   Signature: f9e8d7c6b5a43210...

Step 7: Committing signed entry to LoamSpine...
✓ Signed entry committed
   Entry hash: a1b2c3d4e5f67890...

Step 8: Verifying signature through BearDog...
   Retrieved data: {"action":"transfer","amount":100,"from":"alice","to":"bob"}
✓ Signature verified by BearDog

Step 9: Committing unsigned entry for comparison...
✓ Unsigned entry committed
   Entry hash: b2c3d4e5f6a1b2c3...

================================================================
  Demo Complete!
================================================================

Key Principles:
  - BearDog provides signing capability
  - LoamSpine stores signed entries
  - Public key enables independent verification
  - No hardcoded dependencies between primals
  - Capability discovered at runtime

Gap discovered:
  ⚠️  Need capability-based discovery mechanism
      - How does LoamSpine find BearDog?
      - Answer: Through Songbird orchestrator!
```

---

## 🎓 Learning Points

### Pattern 1: Generate Keypair

```bash
# BearDog generates ed25519 keypair
beardog-cli-signer generate-key \
    --output /tmp/demo-key.pem \
    --key-type ed25519

# Extract public key
PUBLIC_KEY=$(beardog-cli-signer show-public-key \
    --key /tmp/demo-key.pem)
```

### Pattern 2: Sign Entry Hash

```bash
# Compute hash of entry content
CONTENT='{"action":"transfer","amount":100}'
HASH=$(echo -n "$CONTENT" | sha256sum | awk '{print $1}')

# Sign hash with BearDog
SIGNATURE=$(beardog-cli-signer sign \
    --key /tmp/demo-key.pem \
    --message "$HASH" \
    --format base64)
```

**Why sign hash instead of content?**:
- Fixed size (32 bytes)
- Efficient
- Standard practice

### Pattern 3: Commit Signed Entry

```bash
# Include signature in entry metadata
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "commit_entry",
    "params": {
      "session_id": "'$SESSION_ID'",
      "entry": {
        "hash": "'$HASH'",
        "content_type": "application/json",
        "data": "'$(echo -n "$CONTENT" | base64)'",
        "metadata": {
          "signed": true,
          "signature": "'$SIGNATURE'",
          "public_key": "'$PUBLIC_KEY'",
          "signer": "beardog",
          "signature_algorithm": "ed25519"
        }
      }
    },
    "id": 2
  }'
```

### Pattern 4: Verify Signature

```bash
# Anyone with public key can verify
beardog-cli-signer verify \
    --public-key "$PUBLIC_KEY" \
    --message "$HASH" \
    --signature "$SIGNATURE"

# Output: VALID or INVALID
```

**No need for BearDog after signing**:
- Public key is enough
- Verification is cryptographic
- No online service needed

---

## 💡 Key Concepts

### ed25519 Signatures

**Why ed25519?**:
- Fast signing and verification
- Small signatures (64 bytes)
- High security (128-bit equivalent)
- Deterministic (same input = same signature)

**Security Properties**:
```
Private Key → Sign → Signature
Public Key → Verify → ✓ or ✗

Anyone can verify with public key
Only private key holder can sign
```

### Capability-Based Discovery

**Problem**: How does LoamSpine find BearDog?

**Solution**: Songbird orchestrator!

```bash
# LoamSpine queries Songbird
curl http://localhost:8082/api/v1/discover \
  -d '{"capability": "digital-signature"}'

# Songbird responds
{
  "services": [
    {
      "name": "beardog",
      "endpoint": "http://localhost:7000",
      "capabilities": ["digital-signature", "key-management"]
    }
  ]
}
```

**Benefits**:
- No hardcoded endpoints
- Services discover each other at runtime
- Dynamic ecosystem
- Primal sovereignty

### Signed Entry Metadata

```json
{
  "hash": "a1b2c3d4e5f67890...",
  "content_type": "application/json",
  "data": "eyJhY3Rpb24iOiJ0cmFuc2ZlciJ9",
  "metadata": {
    "signed": true,
    "signature": "f9e8d7c6b5a43210...",
    "public_key": "ed25519:a1b2c3d4...",
    "signer": "beardog",
    "signature_algorithm": "ed25519",
    "signed_at": "2025-12-25T10:30:00Z"
  }
}
```

**Fields**:
- `signed`: Boolean flag
- `signature`: Base64-encoded signature
- `public_key`: Public key for verification
- `signer`: Which primal signed it
- `signature_algorithm`: Algorithm used

---

## 🔍 Advanced: Multi-Signature Entries

For critical operations, require multiple signatures:

```json
{
  "hash": "a1b2c3d4...",
  "metadata": {
    "signed": true,
    "signatures": [
      {
        "signer": "beardog-alice",
        "public_key": "ed25519:alice...",
        "signature": "sig_alice..."
      },
      {
        "signer": "beardog-bob",
        "public_key": "ed25519:bob...",
        "signature": "sig_bob..."
      }
    ],
    "threshold": 2,  // Require 2 out of 2
    "policy": "all-required"
  }
}
```

**Use cases**:
- Financial transactions (2-of-3 approval)
- Admin operations (majority vote)
- Escrow (buyer + seller + arbiter)

---

## 📊 Use Cases

### Use Case 1: Financial Transaction

```bash
# Alice transfers 100 tokens to Bob
TRANSACTION='{"from":"alice","to":"bob","amount":100}'

# Sign with Alice's private key (via BearDog)
SIGNATURE=$(sign_with_beardog "$TRANSACTION" alice_key)

# Commit to LoamSpine
loamspine_commit_signed "$TRANSACTION" "$SIGNATURE"

# Anyone can verify Alice authorized this
verify_signature "$TRANSACTION" "$SIGNATURE" alice_public_key
```

### Use Case 2: Audit Event

```bash
# Admin deletes user account
EVENT='{"action":"delete_user","user_id":"12345","admin":"eve"}'

# Sign with admin key
SIGNATURE=$(sign_with_beardog "$EVENT" admin_key)

# Commit to LoamSpine
loamspine_commit_signed "$EVENT" "$SIGNATURE"

# Audit: Verify admin "eve" authorized deletion
verify_signature "$EVENT" "$SIGNATURE" eve_public_key
```

### Use Case 3: Contract Execution

```bash
# Smart contract execution
CONTRACT='{"contract":"0xabc","method":"transfer","args":[...]}'

# Sign with contract owner key
SIGNATURE=$(sign_with_beardog "$CONTRACT" owner_key)

# Commit to LoamSpine
loamspine_commit_signed "$CONTRACT" "$SIGNATURE"

# Blockchain verifies signature before execution
```

---

## 🔧 Gap Discovered: Capability Discovery

This demo reveals a critical need:

### How Primals Discover Each Other

**Current**: Hardcoded endpoint `../bins/beardog-cli-signer`

**Needed**: Runtime discovery via Songbird

```rust
// LoamSpine discovers signing capability
let songbird = SongbirdClient::new("http://localhost:8082");
let signers = songbird.discover_capability("digital-signature").await?;

// Get BearDog endpoint
let beardog = signers.first().ok_or("No signer available")?;

// Use BearDog for signing
let signature = beardog.sign(&hash).await?;
```

**Benefits**:
- No hardcoded dependencies
- BearDog can move/restart
- Multiple signers possible (load balancing)
- Primal sovereignty maintained

---

## 🔧 Troubleshooting

### Error: "BearDog CLI signer not found"

**Cause**: Binary not available

**Solution**:
```bash
# Build BearDog from source
cd ../../phase1/BearDog
cargo build --release --bin beardog-cli-signer

# Copy to bins directory
cp target/release/beardog-cli-signer ../../phase2/loamSpine/bins/
```

### Error: "Signature verification failed"

**Cause**: Hash mismatch or corrupted signature

**Solution**:
```bash
# Recompute hash
echo -n "$CONTENT" | sha256sum

# Check for trailing newlines (common mistake!)
echo -n vs echo

# Verify base64 encoding is correct
echo "$SIGNATURE" | base64 -d | xxd
```

---

## ➡️ Next Steps

After completing this demo:
- **Next Demo**: `04-storage-capability` — NestGate integration
- **Implement**: Capability discovery via Songbird
- **Deep Dive**: `../../specs/INTEGRATION_SPECIFICATION.md`
- **Related**: Gap #3 in `INTEGRATION_GAPS.md`

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ BearDog keypair generated
- ✅ Entry signed with BearDog
- ✅ Signed entry committed to LoamSpine
- ✅ Signature verified successfully
- ✅ Unsigned entry for comparison

---

**Updated**: December 25, 2025  
**Gap Discovered**: Capability-based discovery mechanism needed  
**Principle**: Inter-primal collaboration through capabilities

🦴🐻 **LoamSpine + BearDog: Secure, signed ledger entries**
