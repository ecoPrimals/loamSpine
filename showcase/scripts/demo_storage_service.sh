#!/bin/bash
# 🦴 LoamSpine Showcase - Storage Service Demo
#
# This script demonstrates how LoamSpine integrates with external storage services
# using the capability-based discovery pattern. The actual storage service is
# discovered at runtime via environment variables.
#
# Usage:
#   export LOAMSPINE_STORAGE_PATH=/path/to/storage-binary
#   cd /path/to/loamSpine/showcase/scripts
#   ./demo_storage_service.sh

set -e

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  🏛️ DEMO: STORAGE SERVICE INTEGRATION"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

# Discover storage service (environment variable or default location)
if [ -n "$LOAMSPINE_STORAGE_PATH" ] && [ -x "$LOAMSPINE_STORAGE_PATH" ]; then
    STORAGE="$LOAMSPINE_STORAGE_PATH"
    echo "✓ Storage service discovered via LOAMSPINE_STORAGE_PATH"
elif [ -x "$(cd "$(dirname "$0")/../../../bins" && pwd)/nestgate" ]; then
    STORAGE="$(cd "$(dirname "$0")/../../../bins" && pwd)/nestgate"
    echo "✓ Storage service discovered in ../bins/"
else
    echo "⊘ No storage service found"
    echo ""
    echo "Set LOAMSPINE_STORAGE_PATH to your storage service binary, e.g.:"
    echo "  export LOAMSPINE_STORAGE_PATH=/path/to/storage"
    echo "  ./demo_storage_service.sh"
    echo ""
    echo "This demonstrates the capability-based discovery pattern:"
    echo "  • LoamSpine doesn't hardcode specific primal names"
    echo "  • Storage services are discovered at runtime"
    echo "  • Environment variables configure which service to use"
    exit 0
fi
echo "  Path: $STORAGE"
echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  STORAGE SERVICE CAPABILITIES"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

$STORAGE --help 2>&1 | head -25

echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  INTEGRATION PATTERN: LOAMSPINE → STORAGE SERVICE"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "LoamSpine uses capability-based discovery for storage:"
echo ""
echo "  1. DataAnchor Blob Storage"
echo "     ┌─────────────────────────────────────────────────────────────────────┐"
echo "     │ LoamSpine Entry:                    Storage Service:                │"
echo "     │   EntryType::DataAnchor {      →   POST /api/v1/objects/{hash}     │"
echo "     │     data_hash: [0x12...],          Content: <binary blob>           │"
echo "     │     size: 1024,                                                     │"
echo "     │   }                                                                 │"
echo "     └─────────────────────────────────────────────────────────────────────┘"
echo ""
echo "  2. Spine Backup Storage"
echo "     SpineBackup.export() → Storage service /api/v1/backups/{spine_id}"
echo ""
echo "  3. Content-Addressable Storage"
echo "     Hash → storage key, Blob → storage value"
echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  CAPABILITY-BASED RUST CODE"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "// Discover storage service at runtime (no hardcoded names)"
echo "let storage_url = std::env::var(\"LOAMSPINE_STORAGE_URL\")"
echo "    .unwrap_or_else(|_| \"http://localhost:8092\".to_string());"
echo ""
echo "// Store data anchor blob"
echo "async fn store_blob(hash: ContentHash, data: &[u8]) -> Result<()> {"
echo "    let client = reqwest::Client::new();"
echo "    client.put(&format!(\"{}/api/v1/objects/{}\", storage_url, hex::encode(hash)))"
echo "        .body(data.to_vec())"
echo "        .send().await?;"
echo "    Ok(())"
echo "}"
echo ""
echo "Key Principle: LoamSpine code never hardcodes specific primal names!"
echo ""

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "  🎉 PATTERN COMPLETE"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "Storage service provides sovereign storage for LoamSpine data anchors!"
echo ""
echo "Integration flow:"
echo "  1. LoamSpine creates DataAnchor entry (hash reference)"
echo "  2. Blob is stored in storage service (via capability)"
echo "  3. Hash in spine, data in storage service"
echo "  4. Content-addressable, verifiable, sovereign"
echo ""
echo "Environment variables for configuration:"
echo "  LOAMSPINE_STORAGE_PATH  - Path to storage binary"
echo "  LOAMSPINE_STORAGE_URL   - URL of running storage service"
echo ""

