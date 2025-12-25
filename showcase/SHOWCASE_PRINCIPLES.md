# 🎯 Showcase Principles
## No Mocks, Real Capabilities Only

**Last Updated**: December 23, 2025

---

## 🏆 Core Principle

**The showcase demonstrates REAL capabilities, not aspirations.**

### ✅ What We Do:
- Demonstrate features that are actually implemented
- Use real APIs and real Rust code
- Check for capability availability before running
- Exit gracefully when capabilities are missing
- Provide clear "what you need" instructions

### ❌ What We DON'T Do:
- Mock or simulate features that don't exist
- Generate fake API responses
- Pretend capabilities are available when they're not
- Use placeholder data
- Run demos that can't actually work

---

## 🎯 Why This Matters

### Aligns with Architecture
LoamSpine's capability-based architecture is built on **runtime discovery**:
- Primals discover each other at runtime
- No hardcoded dependencies
- Graceful degradation when services unavailable

**The showcase should demonstrate this principle, not violate it!**

### Builds Trust
- Users see what actually works
- Honest about current state
- Clear about what's implemented vs planned
- Transparent about requirements

### Forces Quality
- Can't hide behind mocks
- Must implement real features
- Demos show actual value
- Gaps are obvious and must be addressed

---

## 📋 Demo Requirements

### Every Demo Must:

1. **Check Prerequisites**
   ```rust
   // Check that LoamSpine compiles and tests pass
   // cargo test --lib
   ```

2. **Use Real Code**
   ```rust
   use loam_spine_core::{Spine, SpineBuilder, Did};
   
   let owner = Did::new("did:key:z6MkOwner");
   let spine = SpineBuilder::new(owner)
       .with_name("Demo Spine")
       .build()?;
   
   // This is REAL spine creation, not a mock
   ```

3. **Verify Capability Exists**
   ```rust
   // Check if signing capability is available
   if let Some(signer) = registry.get_signer().await {
       // Use real signer
   } else {
       println!("Signing capability not available");
       println!("Demo will continue without signatures");
   }
   ```

4. **Exit Gracefully When Missing**
   ```bash
   if [ ! -f "../crates/loam-spine-core/src/lib.rs" ]; then
       echo "LoamSpine not found at expected path"
       echo "Run from: loamSpine/showcase/"
       exit 1
   fi
   ```

5. **Document What's Needed**
   ```bash
   echo "Prerequisites:"
   echo "  • Rust 1.75+ installed"
   echo "  • cargo build successful"
   echo "  • cargo test passing"
   ```

---

## 🔍 Capability Checking Pattern

### Standard Check Template (Bash)

```bash
#!/bin/bash
# Demo: Feature Name
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOAMSPINE_ROOT="$SCRIPT_DIR/../.."

echo "🦴 LoamSpine Demo: Feature Name"
echo "================================"
echo ""

# 1. Check LoamSpine builds
echo "→ Checking LoamSpine builds..."
if ! (cd "$LOAMSPINE_ROOT" && cargo check --quiet 2>/dev/null); then
    echo "✗ LoamSpine does not compile"
    echo ""
    echo "Fix build errors first:"
    echo "  cd $LOAMSPINE_ROOT"
    echo "  cargo build"
    echo ""
    exit 1
fi
echo "✓ LoamSpine compiles"

# 2. Check tests pass
echo "→ Running quick test..."
if ! (cd "$LOAMSPINE_ROOT" && cargo test --lib --quiet 2>/dev/null); then
    echo "✗ Tests failing"
    echo ""
    echo "Fix tests first:"
    echo "  cd $LOAMSPINE_ROOT"
    echo "  cargo test"
    echo ""
    exit 1
fi
echo "✓ Tests passing"

# 3. Run the actual demo
echo ""
echo "Running demo with REAL capabilities..."
cd "$LOAMSPINE_ROOT"
cargo run --example demo_name
```

### Standard Check Template (Rust)

```rust
//! Demo: Feature Name
//! 
//! This demo shows real LoamSpine capabilities.

use loam_spine_core::{
    Spine, SpineBuilder, Did, EntryType,
    LoamSpineResult,
};

fn main() -> LoamSpineResult<()> {
    println!("🦴 LoamSpine Demo: Feature Name");
    println!("================================\n");
    
    // Create real spine
    let owner = Did::new("did:key:z6MkDemoOwner");
    let spine = SpineBuilder::new(owner.clone())
        .with_name("Demo Spine")
        .build()?;
    
    println!("✓ Created spine: {}", spine.id());
    
    // Real operations here...
    
    Ok(())
}
```

---

## 📊 Current Showcase Status

### Implemented & Demonstrable ✅
Features that are **actually implemented** and can be demonstrated:

- [x] Spine creation with SpineBuilder
- [x] Entry creation and appending
- [x] All 15+ entry type variants
- [x] Certificate minting
- [x] Certificate transfer
- [x] Certificate loans with terms
- [x] Inclusion proof generation
- [x] Provenance proof generation
- [x] Backup export (binary + JSON)
- [x] Backup import and verification
- [x] InMemory storage backend
- [x] Sled storage backend
- [x] tarpc RPC service (18 methods)
- [x] JSON-RPC 2.0 server
- [x] Capability-based discovery registry

### Planned / Awaiting Integration ⏳
Features that are **designed but awaiting other primals**:

- [ ] Real RhizoCrypt session commits (traits ready)
- [ ] Real SweetGrass braid commits (traits ready)
- [ ] Real BearDog signing (capability registry ready)
- [ ] Songbird service discovery
- [ ] NestGate payload storage

---

## 🎉 Benefits of This Approach

### For Development
- ✅ Forces implementation of real features
- ✅ Identifies gaps immediately
- ✅ Prioritizes work based on demo needs
- ✅ No technical debt from mocks

### For Users
- ✅ See what actually works
- ✅ Understand requirements clearly
- ✅ Know what's implemented vs planned
- ✅ Can trust the demos

### For Quality
- ✅ Honest assessment of current state
- ✅ Clear roadmap emerges naturally
- ✅ No hiding behind fake data
- ✅ Real capabilities or nothing

---

## 📝 Demo Exit Codes

### Standard Exit Codes

```bash
exit 0  # Success
exit 1  # Error (build failure, missing dependency)
exit 2  # Configuration error
```

**Important**: All demos should work! If a capability is missing, 
the demo should clearly state requirements rather than fail silently.

---

**Principle**: If it's not implemented, the demo should say so clearly. No faking it!

🦴 **LoamSpine: Real capabilities, not aspirations.**

