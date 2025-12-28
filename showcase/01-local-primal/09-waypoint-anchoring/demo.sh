#!/bin/bash
set -e

# Demo: Waypoint Anchoring - Spine Slices and Temporal Ranges
# Demonstrates LoamSpine's waypoint system for temporary slice anchoring

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_NAME="09-waypoint-anchoring"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🦴 LoamSpine Demo: Waypoint Anchoring${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "📍 Waypoints allow temporary spine slices to 'visit' other spines:"
echo "  • SliceAnchor: Slice arrives at waypoint"
echo "  • SliceOperation: Operations performed on slice"
echo "  • SliceDeparture: Slice returns to origin"
echo ""
echo "Real-world use cases:"
echo "  • Certificate lending (game rentals, museum loans)"
echo "  • Temporary data access (research collaborations)"
echo "  • Cross-spine operations (audit, analysis)"
echo ""

# Create a simple waypoint demonstration
cat > /tmp/waypoint_demo.rs << 'EOF'
use loam_spine_core::{Spine, SpineBuilder, Entry, EntryType};
use loam_spine_core::types::{Did, ContentHash};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏗️  Creating two spines: Origin and Waypoint\n");
    
    let alice_did = Did::new("did:key:z6MkAlice");
    let bob_did = Did::new("did:key:z6MkBob");
    
    // 1. Alice's origin spine
    let mut origin_spine = SpineBuilder::new(alice_did.clone())
        .with_name("Alice's Game Library")
        .build()?;
    
    println!("✅ Origin spine created: {}", origin_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Owner: {}", alice_did);
    println!("   ID: {}\n", origin_spine.id);
    
    // 2. Bob's waypoint spine
    let mut waypoint_spine = SpineBuilder::new(bob_did.clone())
        .with_name("Bob's Gaming Session")
        .build()?;
    
    println!("✅ Waypoint spine created: {}", waypoint_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Owner: {}", bob_did);
    println!("   ID: {}\n", waypoint_spine.id);
    
    // 3. Create a slice ID (represents a game certificate)
    let slice_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
    println!("🎮 Creating game slice: {}\n", slice_id);
    
    // 4. SliceAnchor: Game arrives at Bob's spine
    println!("📍 STEP 1: SliceAnchor - Game arrives at waypoint");
    let anchor_entry = Entry::new(
        waypoint_spine.height,
        Some(waypoint_spine.tip),
        bob_did.clone(),
        EntryType::SliceAnchor {
            slice_id,
            origin_spine: origin_spine.id,
            origin_entry: origin_spine.genesis,
        },
    ).with_spine_id(waypoint_spine.id);
    
    waypoint_spine.append(anchor_entry)?;
    println!("   ✅ Game anchored at waypoint");
    println!("   From: {} (Alice's Library)", origin_spine.id);
    println!("   To: {} (Bob's Session)\n", waypoint_spine.id);
    
    // 5. SliceOperation: Bob plays the game
    println!("🎯 STEP 2: SliceOperation - Game is played");
    let play_entry = Entry::new(
        waypoint_spine.height,
        Some(waypoint_spine.tip),
        bob_did.clone(),
        EntryType::SliceOperation {
            slice_id,
            operation: "play_session".to_string(),
        },
    ).with_spine_id(waypoint_spine.id);
    
    waypoint_spine.append(play_entry)?;
    println!("   ✅ Game session recorded");
    println!("   Operation: play_session");
    println!("   Duration: 2 hours\n");
    
    // 6. Another operation
    let achievement_entry = Entry::new(
        waypoint_spine.height,
        Some(waypoint_spine.tip),
        bob_did.clone(),
        EntryType::SliceOperation {
            slice_id,
            operation: "achievement_unlocked".to_string(),
        },
    ).with_spine_id(waypoint_spine.id);
    
    waypoint_spine.append(achievement_entry)?;
    println!("   ✅ Achievement recorded");
    println!("   Operation: achievement_unlocked\n");
    
    // 7. SliceDeparture: Game returns to origin
    println!("🚀 STEP 3: SliceDeparture - Game returns home");
    let departure_entry = Entry::new(
        waypoint_spine.height,
        Some(waypoint_spine.tip),
        bob_did.clone(),
        EntryType::SliceDeparture {
            slice_id,
            reason: "Session complete".to_string(),
        },
    ).with_spine_id(waypoint_spine.id);
    
    waypoint_spine.append(departure_entry)?;
    println!("   ✅ Game departed waypoint");
    println!("   Reason: Session complete");
    println!("   Operations: 2 (play + achievement)\n");
    
    // 8. Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📊 Waypoint Session Summary:\n");
    println!("Origin Spine:");
    println!("   Name: {}", origin_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Height: {} entries", origin_spine.height);
    println!("   Owner: {}\n", alice_did);
    
    println!("Waypoint Spine:");
    println!("   Name: {}", waypoint_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Height: {} entries", waypoint_spine.height);
    println!("   Owner: {}", bob_did);
    println!("   Slice journey:");
    println!("     1. Anchor (slice arrives)");
    println!("     2. Operation (play_session)");
    println!("     3. Operation (achievement_unlocked)");
    println!("     4. Departure (slice returns)\n");
    
    println!("💡 Key Insight:");
    println!("   The slice's journey is permanently recorded at the waypoint,");
    println!("   while ownership remains with the origin spine. This enables:");
    println!("     • Game rentals with usage tracking");
    println!("     • Museum artwork loans with viewing records");
    println!("     • Research data access with audit trails");
    println!("     • Certificate lending with operation history\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Running waypoint demo...${NC}"
echo ""

cd "${SCRIPT_DIR}/../../../.."
rustc --edition 2021 /tmp/waypoint_demo.rs \
  -L target/debug/deps \
  --extern loam_spine_core=target/debug/libloam_spine_core.rlib \
  --extern uuid=target/debug/deps/libuuid-*.rlib \
  -o /tmp/waypoint_demo 2>&1 || {
    echo "Building dependencies first..."
    cargo build --lib
    rustc --edition 2021 /tmp/waypoint_demo.rs \
      -L target/debug/deps \
      --extern loam_spine_core=target/debug/libloam_spine_core.rlib \
      --extern uuid=target/debug/deps/libuuid-*.rlib \
      -o /tmp/waypoint_demo 2>&1
}

/tmp/waypoint_demo
rm /tmp/waypoint_demo /tmp/waypoint_demo.rs

echo ""
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 What you learned:"
echo "  ✅ Waypoint pattern (anchor → operate → depart)"
echo "  ✅ SliceAnchor for slice arrival"
echo "  ✅ SliceOperation for waypoint activities"
echo "  ✅ SliceDeparture for slice return"
echo "  ✅ Permanent journey record at waypoint"
echo ""
echo "💡 Key concepts:"
echo "   • Slices can temporarily 'visit' other spines"
echo "   • Operations are recorded at the waypoint"
echo "   • Ownership stays with origin spine"
echo "   • Full audit trail of slice journey"
echo ""
echo "🎯 Next:"
echo "   See 10-recursive-spines/ for spine composition patterns"
echo ""

