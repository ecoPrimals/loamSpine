#!/bin/bash
set -e

# Demo: Temporal Moments - Universal Time Tracking
# Demonstrates LoamSpine's unique temporal primitives (NEW in v0.7.0)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_NAME="08-temporal-moments"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🦴 LoamSpine Demo: Temporal Moments${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "⏰ NEW in v0.7.0: Universal Time Tracking"
echo ""
echo "Temporal moments allow you to track time across ANY domain:"
echo "  • Code commits (version control)"
echo "  • Art creation (creative works)"
echo "  • Life events (personal milestones)"
echo "  • Scientific experiments (research)"
echo "  • Business milestones (organizational)"
echo ""
echo "This demo shows how LoamSpine can be the universal time layer."
echo ""

# Run the example
echo -e "${YELLOW}Running temporal moments example...${NC}"
echo ""

cd "${SCRIPT_DIR}/../../../.."
cargo run --example temporal_moments 2>&1

echo ""
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 What you learned:"
echo "  ✅ Moment structure (id, timestamp, agent, context)"
echo "  ✅ Multiple moment types (code, art, life, experiment)"
echo "  ✅ Anchor types (atomic, crypto, causal, consensus)"
echo "  ✅ EntryType::TemporalMoment integration"
echo "  ✅ Universal time tracking primitives"
echo ""
echo "💡 Key insight:"
echo "   Time is the primitive, not version control."
echo "   LoamSpine provides universal temporal tracking for ANY domain."
echo ""
echo "🎯 Next:"
echo "   See 09-waypoint-anchoring/ for spine composition patterns"
echo ""

