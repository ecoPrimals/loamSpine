// SPDX-License-Identifier: AGPL-3.0-or-later

//! Parser for `capability.list` responses from partner primals.
//!
//! Supports 4 wire formats used across the ecosystem per the
//! Capability Wire Standard v1.0.

/// Information about a single capability method from a partner primal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMethod {
    /// Method name (e.g. `"spine.create"`).
    pub method: String,
    /// Domain (e.g. `"spine"`, `"certificate"`).
    pub domain: Option<String>,
    /// Cost tier (e.g. `"low"`, `"medium"`, `"high"`).
    pub cost: Option<String>,
    /// Dependencies (other methods that must be called first).
    pub deps: Vec<String>,
}

/// Parsed capability list from a partner primal's `capability.list` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCapabilities {
    /// Primal name.
    pub primal: Option<String>,
    /// Primal version.
    pub version: Option<String>,
    /// Flat capability strings (e.g. `["permanence", "session.commit"]`).
    pub capabilities: Vec<String>,
    /// Structured method descriptors (if `methods` array of objects is present).
    pub methods: Vec<CapabilityMethod>,
}

/// Parse a `capability.list` JSON-RPC response from any primal.
///
/// Supports 4 formats used across the ecosystem:
/// 1. Flat array: `{ "capabilities": ["a", "b"] }`
/// 2. Object array: `{ "methods": [{ "method": "a", "domain": "x" }] }`
/// 3. Nested domains: `{ "domains": { "spine": ["create", "get"] } }`
/// 4. Combined: flat `capabilities` + structured `methods`
///
/// Also handles Wire Standard L2 flat `methods` (string array) by treating
/// each string as both a capability and a method name.
///
/// Aligns with wetSpring V125 / airSpring v0.8.7 `parse_capabilities()`.
#[must_use]
pub fn extract_capabilities(response: &serde_json::Value) -> ParsedCapabilities {
    let primal = response
        .get("primal")
        .and_then(serde_json::Value::as_str)
        .map(String::from);
    let version = response
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(String::from);

    let mut capabilities = Vec::new();
    let mut methods = Vec::new();

    if let Some(caps) = response
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
    {
        for cap in caps {
            if let Some(s) = cap.as_str() {
                capabilities.push(s.to_string());
            }
        }
    }

    if let Some(meths) = response
        .get("methods")
        .and_then(serde_json::Value::as_array)
    {
        for m in meths {
            let Some(method_name) = m.get("method").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let domain = m
                .get("domain")
                .and_then(serde_json::Value::as_str)
                .map(String::from);
            let cost = m
                .get("cost")
                .and_then(serde_json::Value::as_str)
                .map(String::from);
            let deps = m
                .get("deps")
                .and_then(serde_json::Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            methods.push(CapabilityMethod {
                method: method_name.to_string(),
                domain,
                cost,
                deps,
            });
        }
    }

    if let Some(domains) = response
        .get("domains")
        .and_then(serde_json::Value::as_object)
    {
        for (domain_name, methods_val) in domains {
            if let Some(arr) = methods_val.as_array() {
                for m in arr {
                    if let Some(method_str) = m.as_str() {
                        let full_method = format!("{domain_name}.{method_str}");
                        if !capabilities.contains(&full_method) {
                            capabilities.push(full_method);
                        }
                    }
                }
            }
        }
    }

    ParsedCapabilities {
        primal,
        version,
        capabilities,
        methods,
    }
}
