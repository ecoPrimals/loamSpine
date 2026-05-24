// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

// ── Socket path resolution (pure inner functions) ────────────────────────

#[test]
fn resolve_socket_path_returns_valid_path() {
    let path = resolve_socket_path_with(None, None, None);
    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("loamspine"));
}

#[test]
fn socket_path_respects_override() {
    let path = resolve_socket_path_with(Some("/custom/loamspine.sock"), None, None);
    assert_eq!(path.to_string_lossy(), "/custom/loamspine.sock");
}

#[test]
fn resolve_socket_path_uses_xdg_runtime_dir() {
    let path = resolve_socket_path_with(None, None, Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine.sock"
    );
}

#[test]
fn resolve_socket_path_uses_xdg_runtime_dir_with_family() {
    let path = resolve_socket_path_with(None, Some("myfamily"), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine-myfamily.sock"
    );
}

#[test]
fn resolve_socket_path_fallback_when_xdg_unset() {
    let path = resolve_socket_path_with(None, None, None);
    assert!(
        path.to_string_lossy().ends_with("biomeos/loamspine.sock"),
        "got: {}",
        path.display()
    );
}

#[test]
fn resolve_socket_path_with_custom_family_id() {
    let path = resolve_socket_path_with(None, Some("custom-family"), None);
    assert!(
        path.to_string_lossy()
            .ends_with("biomeos/loamspine-custom-family.sock"),
        "got: {}",
        path.display()
    );
}

#[test]
fn resolve_socket_path_override_wins_over_xdg_and_family() {
    let path = resolve_socket_path_with(
        Some("/override/path.sock"),
        Some("ignored"),
        Some("/run/user/1000"),
    );
    assert_eq!(path.to_string_lossy(), "/override/path.sock");
}

#[test]
fn resolve_socket_path_empty_family_id_treated_as_unset() {
    let path = resolve_socket_path_with(None, Some(""), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine.sock",
        "empty BIOMEOS_FAMILY_ID should be treated as unset"
    );
}

#[test]
fn resolve_socket_path_default_family_id_treated_as_unset() {
    let path = resolve_socket_path_with(None, Some("default"), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine.sock",
        "BIOMEOS_FAMILY_ID=default should produce domain-only socket"
    );
}

// ── Domain socket naming ────────────────────────────────────────────

#[test]
fn domain_socket_name_without_family() {
    assert_eq!(domain_socket_name(None), "loamspine.sock");
    assert_eq!(domain_socket_name(Some("")), "loamspine.sock");
    assert_eq!(domain_socket_name(Some("default")), "loamspine.sock");
}

#[test]
fn domain_socket_name_with_family() {
    assert_eq!(domain_socket_name(Some("prod")), "loamspine-prod.sock");
}

#[test]
fn legacy_socket_name_without_family() {
    assert_eq!(legacy_socket_name(None), "permanence.sock");
}

#[test]
fn legacy_socket_name_with_family() {
    assert_eq!(legacy_socket_name(Some("prod")), "permanence-prod.sock");
}

#[test]
fn legacy_symlink_path_matches_parent() {
    let primary = std::path::Path::new("/run/user/1000/biomeos/loamspine.sock");
    let legacy = resolve_legacy_symlink_path(primary, None);
    assert_eq!(
        legacy.to_string_lossy(),
        "/run/user/1000/biomeos/permanence.sock"
    );
}

// ── Capability-domain socket naming ─────────────────────────────────

#[test]
fn capability_domain_socket_name_without_family() {
    assert_eq!(capability_domain_socket_name(None), "ledger.sock");
    assert_eq!(capability_domain_socket_name(Some("")), "ledger.sock");
    assert_eq!(
        capability_domain_socket_name(Some("default")),
        "ledger.sock"
    );
}

#[test]
fn capability_domain_socket_name_with_family() {
    assert_eq!(
        capability_domain_socket_name(Some("prod")),
        "ledger-prod.sock"
    );
}

#[test]
fn capability_symlink_path_matches_parent() {
    let primary = std::path::Path::new("/run/user/1000/biomeos/loamspine.sock");
    let cap = resolve_capability_symlink_path(primary, None);
    assert_eq!(cap.to_string_lossy(), "/run/user/1000/biomeos/ledger.sock");
}

// ── Security config validation ──────────────────────────────────────

#[test]
fn validate_security_config_ok_no_family() {
    assert!(validate_security_config(None, None).is_ok());
    assert!(validate_security_config(None, Some("1")).is_ok());
}

#[test]
fn validate_security_config_ok_family_no_insecure() {
    assert!(validate_security_config(Some("prod"), None).is_ok());
    assert!(validate_security_config(Some("prod"), Some("0")).is_ok());
}

#[test]
fn validate_security_config_ok_default_family_insecure() {
    assert!(validate_security_config(Some("default"), Some("1")).is_ok());
    assert!(validate_security_config(Some(""), Some("1")).is_ok());
}

#[test]
fn validate_security_config_rejects_family_plus_insecure() {
    let err = validate_security_config(Some("prod"), Some("1"));
    assert!(err.is_err(), "family + insecure must be rejected");
    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("BIOMEOS_INSECURE"));
}

// ── NeuralAPI socket resolution (pure inner function) ────────────────────

#[test]
fn resolve_neural_api_socket_with_explicit() {
    let path = resolve_neural_api_socket_with(Some("/custom/neural.sock"), None, None);
    assert!(path.is_some());
    assert_eq!(path.unwrap().to_string_lossy(), "/custom/neural.sock");
}

#[test]
fn resolve_neural_api_socket_with_xdg_runtime_dir() {
    let path = resolve_neural_api_socket_with(None, Some("/run/user/1000"), None);
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().to_string_lossy(),
        "/run/user/1000/biomeos/neural-api-default.sock"
    );
}

#[test]
fn resolve_neural_api_socket_without_env_returns_none() {
    let path = resolve_neural_api_socket_with(None, None, None);
    assert!(path.is_none());
}

#[test]
fn resolve_neural_api_socket_with_family_id() {
    let path = resolve_neural_api_socket_with(None, Some("/run/user/42"), Some("my-family"));
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().to_string_lossy(),
        "/run/user/42/biomeos/neural-api-my-family.sock"
    );
}

// ── validate_security_config_from_env ────────────────────────────────

#[test]
fn validate_security_config_from_env_does_not_panic() {
    let result = super::validate_security_config_from_env();
    let _ = result;
}
