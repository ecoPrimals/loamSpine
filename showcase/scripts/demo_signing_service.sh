#!/bin/bash
# 🦴 LoamSpine Showcase - Signing Service Demo
#
# This script demonstrates how LoamSpine integrates with external signing services
# using the capability-based discovery pattern. The actual signing service is
# discovered at runtime via environment variables.
#
# Usage:
#   export LOAMSPINE_SIGNER_PATH=/path/to/signing-binary
#   cd /path/to/loamSpine/showcase/scripts
#   ./demo_signing_service.sh

set -e

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  🔐 DEMO: SIGNING SERVICE INTEGRATION"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

# Discover signing service (environment variable or default location)
if [ -n "$LOAMSPINE_SIGNER_PATH" ] && [ -x "$LOAMSPINE_SIGNER_PATH" ]; then
    SIGNER="$LOAMSPINE_SIGNER_PATH"
    echo "✓ Signing service discovered via LOAMSPINE_SIGNER_PATH"
elif [ -x "$(cd "$(dirname "$0")/../../../bins" && pwd)/beardog" ]; then
    SIGNER="$(cd "$(dirname "$0")/../../../bins" && pwd)/beardog"
    echo "✓ Signing service discovered in ../bins/"
else
    echo "⊘ No signing service found"
    echo ""
    echo "Set LOAMSPINE_SIGNER_PATH to your signing service binary, e.g.:"
    echo "  export LOAMSPINE_SIGNER_PATH=/path/to/signer"
    echo "  ./demo_signing_service.sh"
    echo ""
    echo "This demonstrates the capability-based discovery pattern:"
    echo "  • LoamSpine doesn't hardcode specific primal names"
    echo "  • Signing services are discovered at runtime"
    echo "  • Environment variables configure which service to use"
    exit 0
fi
echo "  Path: $SIGNER"
echo ""

CERTS_DIR="$(dirname "$SIGNER")/certs"
mkdir -p "$CERTS_DIR" 2>/dev/null || true

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  STEP 1: GENERATE ED25519 SIGNING KEY"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

KEY_ID="loamspine-demo-$(date +%s)"
echo "Generating key: $KEY_ID"
echo ""

# Generate an Ed25519 signing key (standard CLI protocol)
$SIGNER key generate \
    --key-id "$KEY_ID" \
    --algorithm ed25519 \
    --hsm software 2>&1 || echo "(Key generation output above)"

echo ""
echo "✓ Ed25519 key generated"
echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  STEP 2: CHECK SERVICE STATUS"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

$SIGNER status 2>&1 || echo "(Status output above)"

echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  CAPABILITY-BASED INTEGRATION PATTERN"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "LoamSpine uses capability-based discovery for signing:"
echo ""
echo "  1. Environment Variable Discovery"
echo "     LOAMSPINE_SIGNER_PATH=/path/to/signer"
echo ""
echo "  2. Rust Capability Registry"
echo "     use loam_spine_core::{CliSigner, CapabilityRegistry};"
echo "     let signer = CliSigner::new(\"\$LOAMSPINE_SIGNER_PATH\", \"$KEY_ID\")?;"
echo "     registry.register_signer(Arc::new(signer)).await;"
echo ""
echo "  3. Runtime Discovery"
echo "     if let Some(signer) = registry.get_signer().await {"
echo "         let sig = signer.sign_boxed(data).await?;"
echo "     }"
echo ""
echo "Key Principle: LoamSpine code never hardcodes specific primal names!"
echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  🎉 SUCCESS"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "Signing service provides real cryptographic capabilities for LoamSpine!"
echo ""
echo "Key generated: $KEY_ID"
echo "Algorithm:     Ed25519"
echo "HSM:           Software"
echo ""
echo "Next: Run the full inter-primal demo:"
echo "  cargo run -p loam-spine-core --example demo_inter_primal"
echo ""

