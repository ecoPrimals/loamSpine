#!/bin/bash
set -e

# Demo: Recursive Spines - Spine Composition Patterns
# Demonstrates how spines can reference other spines

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_NAME="10-recursive-spines"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🦴 LoamSpine Demo: Recursive Spines${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "🔗 Recursive spines enable powerful composition patterns:"
echo "  • Spine-to-spine references"
echo "  • Cross-spine proofs"
echo "  • Hierarchical organization"
echo "  • Distributed ledgers"
echo ""
echo "Real-world use cases:"
echo "  • Project spine → sub-project spines"
echo "  • Organization spine → team spines"
echo "  • Research program → experiment spines"
echo "  • Portfolio spine → individual asset spines"
echo ""

# Create demonstration
cat > /tmp/recursive_demo.rs << 'EOF'
use loam_spine_core::{Spine, SpineBuilder, Entry, EntryType};
use loam_spine_core::types::{Did, ContentHash};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏗️  Creating hierarchical spine structure\n");
    
    let org_did = Did::new("did:key:z6MkOrganization");
    let team_a_did = Did::new("did:key:z6MkTeamA");
    let team_b_did = Did::new("did:key:z6MkTeamB");
    
    // 1. Organization spine (root)
    let mut org_spine = SpineBuilder::new(org_did.clone())
        .with_name("Research Organization")
        .build()?;
    
    println!("✅ Root spine created: {}", org_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Owner: {}", org_did);
    println!("   ID: {}\n", org_spine.id);
    
    // 2. Team A spine (child)
    let mut team_a_spine = SpineBuilder::new(team_a_did.clone())
        .with_name("Team A - ML Research")
        .build()?;
    
    println!("✅ Child spine A created: {}", team_a_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Owner: {}", team_a_did);
    println!("   ID: {}\n", team_a_spine.id);
    
    // 3. Team B spine (child)
    let mut team_b_spine = SpineBuilder::new(team_b_did.clone())
        .with_name("Team B - Systems Research")
        .build()?;
    
    println!("✅ Child spine B created: {}", team_b_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Owner: {}", team_b_did);
    println!("   ID: {}\n", team_b_spine.id);
    
    // 4. Teams do work (create entries)
    println!("🔬 Teams conduct research...\n");
    
    let team_a_work = Entry::new(
        team_a_spine.height,
        Some(team_a_spine.tip),
        team_a_did.clone(),
        EntryType::GenericData {
            data_type: "research_result".to_string(),
            content_hash: ContentHash::generate_from_str("team_a_ml_model"),
            metadata: serde_json::json!({
                "title": "Novel ML Architecture",
                "status": "complete"
            }).to_string().into_bytes().into(),
        },
    ).with_spine_id(team_a_spine.id);
    
    team_a_spine.append(team_a_work)?;
    println!("   ✅ Team A: Research result added");
    
    let team_b_work = Entry::new(
        team_b_spine.height,
        Some(team_b_spine.tip),
        team_b_did.clone(),
        EntryType::GenericData {
            data_type: "research_result".to_string(),
            content_hash: ContentHash::generate_from_str("team_b_system_design"),
            metadata: serde_json::json!({
                "title": "Distributed System Design",
                "status": "complete"
            }).to_string().into_bytes().into(),
        },
    ).with_spine_id(team_b_spine.id);
    
    team_b_spine.append(team_b_work)?;
    println!("   ✅ Team B: Research result added\n");
    
    // 5. Organization spine references team spines
    println!("🔗 Organization spine references team results...\n");
    
    let team_a_ref = Entry::new(
        org_spine.height,
        Some(org_spine.tip),
        org_did.clone(),
        EntryType::SpineReference {
            referenced_spine: team_a_spine.id,
            referenced_entry: team_a_spine.tip,
            reference_type: "team_submission".to_string(),
        },
    ).with_spine_id(org_spine.id);
    
    org_spine.append(team_a_ref)?;
    println!("   ✅ Referenced Team A's spine");
    println!("      From: {} → To: {}", org_spine.id, team_a_spine.id);
    
    let team_b_ref = Entry::new(
        org_spine.height,
        Some(org_spine.tip),
        org_did.clone(),
        EntryType::SpineReference {
            referenced_spine: team_b_spine.id,
            referenced_entry: team_b_spine.tip,
            reference_type: "team_submission".to_string(),
        },
    ).with_spine_id(org_spine.id);
    
    org_spine.append(team_b_ref)?;
    println!("   ✅ Referenced Team B's spine");
    println!("      From: {} → To: {}\n", org_spine.id, team_b_spine.id);
    
    // 6. Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📊 Recursive Spine Structure:\n");
    
    println!("Root Spine (Organization):");
    println!("   Name: {}", org_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Height: {} entries", org_spine.height);
    println!("   References: 2 child spines\n");
    
    println!("Child Spine A (Team A):");
    println!("   Name: {}", team_a_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Height: {} entries", team_a_spine.height);
    println!("   Owner: {}\n", team_a_did);
    
    println!("Child Spine B (Team B):");
    println!("   Name: {}", team_b_spine.name.as_deref().unwrap_or("Unnamed"));
    println!("   Height: {} entries", team_b_spine.height);
    println!("   Owner: {}\n", team_b_did);
    
    println!("💡 Key Insights:");
    println!("   • Each spine maintains sovereignty (own owner, own history)");
    println!("   • Parent spine can reference children without owning them");
    println!("   • Cross-spine proofs enable verification without copying data");
    println!("   • Enables: organizations, projects, portfolios, research programs");
    println!("   • O(n) composition instead of O(n²) full replication\n");
    
    println!("🌳 Hierarchy:");
    println!("   Organization Spine (root)");
    println!("   ├─ Team A Spine → ML Research");
    println!("   └─ Team B Spine → Systems Research\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Running recursive spines demo...${NC}"
echo ""

cd "${SCRIPT_DIR}/../../../.."
rustc --edition 2024 /tmp/recursive_demo.rs \
  -L target/debug/deps \
  --extern loam_spine_core=target/debug/libloam_spine_core.rlib \
  --extern serde_json=target/debug/deps/libserde_json-*.rlib \
  -o /tmp/recursive_demo 2>&1 || {
    echo "Building dependencies first..."
    cargo build --lib
    rustc --edition 2024 /tmp/recursive_demo.rs \
      -L target/debug/deps \
      --extern loam_spine_core=target/debug/libloam_spine_core.rlib \
      --extern serde_json=target/debug/deps/libserde_json-*.rlib \
      -o /tmp/recursive_demo 2>&1
}

/tmp/recursive_demo
rm /tmp/recursive_demo /tmp/recursive_demo.rs

echo ""
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 What you learned:"
echo "  ✅ SpineReference entry type"
echo "  ✅ Hierarchical spine structures"
echo "  ✅ Cross-spine composition"
echo "  ✅ Sovereign spines with references"
echo "  ✅ O(n) vs O(n²) efficiency"
echo ""
echo "💡 Key concepts:"
echo "   • Each spine maintains sovereignty"
echo "   • References enable composition without duplication"
echo "   • Cross-spine proofs verify without copying"
echo "   • Enables: orgs, projects, portfolios, programs"
echo ""
echo "🎯 Next Phase:"
echo "   See ../02-rpc-service/ for real service integration"
echo ""

