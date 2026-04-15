// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP (Model Context Protocol) tool advertisements.
//!
//! Each tool maps to a JSON-RPC method with an `inputSchema` describing
//! the expected `params`. AI agents call `tools/list` to discover what
//! operations are available, then invoke them via `tools/call`.

use std::sync::OnceLock;

static MCP_TOOLS_CACHE: OnceLock<serde_json::Value> = OnceLock::new();

/// Return MCP `tools/list` response payload.
///
/// Uses `OnceLock` to initialize the JSON value once and return a reference thereafter.
#[must_use]
pub fn mcp_tools_list() -> &'static serde_json::Value {
    MCP_TOOLS_CACHE.get_or_init(mcp_tools_list_inner)
}

#[expect(
    clippy::too_many_lines,
    reason = "declarative MCP tool schema definitions"
)]
fn mcp_tools_list_inner() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            mcp_tool("spine_create", "Create a new sovereign spine (append-only ledger)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Human-readable spine name" },
                    "owner": { "type": "string", "description": "DID of the spine owner" }
                },
                "required": ["name", "owner"]
            })),
            mcp_tool("spine_get", "Get a spine by ID", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" }
                },
                "required": ["spine_id"]
            })),
            mcp_tool("spine_seal", "Seal a spine (make permanently read-only)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID to seal" },
                    "sealer": { "type": "string", "description": "DID of the sealer" }
                },
                "required": ["spine_id", "sealer"]
            })),
            mcp_tool("entry_append", "Append an entry to a spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Target spine ID" },
                    "domain": { "type": "string", "description": "Entry domain (e.g. 'commit', 'certificate')" },
                    "payload": { "type": "string", "description": "Entry payload (base64 or JSON string)" }
                },
                "required": ["spine_id", "domain", "payload"]
            })),
            mcp_tool("entry_get", "Get an entry by spine ID and index", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" },
                    "index": { "type": "integer", "description": "Entry index" }
                },
                "required": ["spine_id", "index"]
            })),
            mcp_tool("entry_get_tip", "Get the latest (tip) entry of a spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" }
                },
                "required": ["spine_id"]
            })),
            mcp_tool("certificate_mint", "Mint a new certificate (memory-bound object)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine to mint on" },
                    "owner": { "type": "string", "description": "Owner DID" },
                    "cert_type": { "type": "string", "description": "Certificate type" },
                    "name": { "type": "string", "description": "Certificate name" }
                },
                "required": ["spine_id", "owner", "cert_type"]
            })),
            mcp_tool("certificate_get", "Get certificate by ID", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" }
                },
                "required": ["certificate_id"]
            })),
            mcp_tool("certificate_transfer", "Transfer certificate ownership", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" },
                    "from": { "type": "string", "description": "Current owner DID" },
                    "to": { "type": "string", "description": "New owner DID" }
                },
                "required": ["certificate_id", "from", "to"]
            })),
            mcp_tool("certificate_loan", "Loan a certificate to another identity", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" },
                    "borrower": { "type": "string", "description": "Borrower DID" },
                    "duration_secs": { "type": "integer", "description": "Loan duration in seconds" }
                },
                "required": ["certificate_id", "borrower"]
            })),
            mcp_tool("certificate_return", "Return a loaned certificate", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" },
                    "returner": { "type": "string", "description": "Borrower DID returning the certificate" }
                },
                "required": ["certificate_id", "returner"]
            })),
            mcp_tool("certificate_verify", "Verify a certificate's chain of custody", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID to verify" }
                },
                "required": ["certificate_id"]
            })),
            mcp_tool("certificate_lifecycle", "Get full lifecycle history of a certificate", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" }
                },
                "required": ["certificate_id"]
            })),
            mcp_tool("slice_anchor", "Anchor a slice on a waypoint spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" },
                    "origin_spine_id": { "type": "integer", "description": "Origin spine ID" },
                    "committer": { "type": "string", "description": "Committer DID" }
                },
                "required": ["waypoint_spine_id", "slice_id", "origin_spine_id", "committer"]
            })),
            mcp_tool("slice_checkout", "Checkout a slice from a waypoint spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" },
                    "requester": { "type": "string", "description": "Requester DID" }
                },
                "required": ["waypoint_spine_id", "slice_id", "requester"]
            })),
            mcp_tool("slice_record_operation", "Record an operation on a checked-out slice", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" },
                    "operation": { "type": "string", "description": "Operation payload (JSON)" }
                },
                "required": ["waypoint_spine_id", "slice_id", "operation"]
            })),
            mcp_tool("slice_depart", "Depart (close) a slice and finalize waypoint entry", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" }
                },
                "required": ["waypoint_spine_id", "slice_id"]
            })),
            mcp_tool("proof_generate_inclusion", "Generate an inclusion proof for an entry", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" },
                    "index": { "type": "integer", "description": "Entry index to prove" }
                },
                "required": ["spine_id", "index"]
            })),
            mcp_tool("proof_verify_inclusion", "Verify an inclusion proof", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" },
                    "entry_hash": { "type": "string", "description": "Entry hash (hex)" },
                    "proof": { "type": "object", "description": "Inclusion proof object" }
                },
                "required": ["spine_id", "entry_hash", "proof"]
            })),
            mcp_tool("session_commit", "Commit an ephemeral session to permanent storage", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Target spine ID" },
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "session_hash": { "type": "string", "description": "Session DAG root hash (hex)" },
                    "vertex_count": { "type": "integer", "description": "Number of vertices" },
                    "committer": { "type": "string", "description": "Committer DID" }
                },
                "required": ["spine_id", "session_id", "session_hash", "committer"]
            })),
            mcp_tool("braid_commit", "Commit a semantic attribution braid", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Target spine ID" },
                    "braid_id": { "type": "string", "description": "Braid UUID" },
                    "braid_hash": { "type": "string", "description": "Braid hash (hex)" },
                    "subjects": { "type": "array", "items": { "type": "string" }, "description": "Subject DIDs" },
                    "committer": { "type": "string", "description": "Committer DID" }
                },
                "required": ["spine_id", "braid_id", "braid_hash", "committer"]
            })),
            mcp_tool("anchor_publish", "Record a public chain anchor on a spine (external provenance verification)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "string", "description": "Spine ID (UUID)" },
                    "anchor_target": { "type": "string", "description": "Target system: Bitcoin, Ethereum, FederatedSpine, DataCommons, or Other" },
                    "tx_ref": { "type": "string", "description": "Transaction hash or proof reference on external system" },
                    "block_height": { "type": "integer", "description": "Block height or sequence number (0 if N/A)" },
                    "anchor_timestamp": { "type": "integer", "description": "Anchor confirmation timestamp (epoch ms)" }
                },
                "required": ["spine_id", "anchor_target", "tx_ref", "anchor_timestamp"]
            })),
            mcp_tool("anchor_verify", "Verify a spine's state against a recorded public chain anchor", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "string", "description": "Spine ID (UUID)" },
                    "anchor_entry_hash": { "type": "string", "description": "Specific anchor entry hash (hex); omit for latest" }
                },
                "required": ["spine_id"]
            })),
            mcp_tool("bonding_ledger_store", "Store an ionic bond record in the permanent ledger", &serde_json::json!({
                "type": "object",
                "properties": {
                    "bond_id": { "type": "string", "description": "Unique bond identifier" },
                    "data": { "type": "object", "description": "Bond data to persist (opaque JSON)" }
                },
                "required": ["bond_id", "data"]
            })),
            mcp_tool("bonding_ledger_retrieve", "Retrieve a bond record by ID", &serde_json::json!({
                "type": "object",
                "properties": {
                    "bond_id": { "type": "string", "description": "Bond identifier to look up" }
                },
                "required": ["bond_id"]
            })),
            mcp_tool("bonding_ledger_list", "List all stored bond identifiers", &serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            mcp_tool("health_check", "Check LoamSpine health status", &serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            mcp_tool("capability_list", "List all LoamSpine capabilities and methods", &serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            mcp_tool("identity_get", "Get LoamSpine primal identity (name, version, domain, license)", &serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
        ]
    })
}

fn mcp_tool(name: &str, description: &str, input_schema: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

/// Handle an MCP `tools/call` by mapping the tool name to a JSON-RPC method.
///
/// Returns `(method, params)` suitable for dispatching through the JSON-RPC
/// handler. Returns `None` if the tool name is unrecognized.
#[must_use]
pub fn mcp_tool_to_rpc(
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<(&'static str, serde_json::Value)> {
    let method = match tool_name {
        "spine_create" => "spine.create",
        "spine_get" => "spine.get",
        "spine_seal" => "spine.seal",
        "entry_append" => "entry.append",
        "entry_get" => "entry.get",
        "entry_get_tip" => "entry.get_tip",
        "certificate_mint" => "certificate.mint",
        "certificate_get" => "certificate.get",
        "certificate_transfer" => "certificate.transfer",
        "certificate_loan" => "certificate.loan",
        "certificate_return" => "certificate.return",
        "certificate_verify" => "certificate.verify",
        "certificate_lifecycle" => "certificate.lifecycle",
        "slice_anchor" => "slice.anchor",
        "slice_checkout" => "slice.checkout",
        "slice_record_operation" => "slice.record_operation",
        "slice_depart" => "slice.depart",
        "proof_generate_inclusion" => "proof.generate_inclusion",
        "proof_verify_inclusion" => "proof.verify_inclusion",
        "session_commit" => "session.commit",
        "braid_commit" => "braid.commit",
        "anchor_publish" => "anchor.publish",
        "anchor_verify" => "anchor.verify",
        "bonding_ledger_store" => "bonding.ledger.store",
        "bonding_ledger_retrieve" => "bonding.ledger.retrieve",
        "bonding_ledger_list" => "bonding.ledger.list",
        "health_check" => "health.check",
        "capability_list" => "capabilities.list",
        "identity_get" => "identity.get",
        _ => return None,
    };
    Some((method, arguments))
}
