// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

// ── Port resolution ──────────────────────────────────────────────────

#[test]
fn jsonrpc_port_from_loamspine_env() {
    assert_eq!(resolve_jsonrpc_port(Some("8888"), None), 8888);
}

#[test]
fn jsonrpc_port_default_when_unset() {
    assert_eq!(resolve_jsonrpc_port(None, None), DEFAULT_JSONRPC_PORT);
}

#[test]
fn jsonrpc_port_invalid_loamspine_falls_back_to_generic() {
    assert_eq!(resolve_jsonrpc_port(Some("invalid"), Some("7777")), 7777);
}

#[test]
fn jsonrpc_port_invalid_both_falls_back_to_default() {
    assert_eq!(
        resolve_jsonrpc_port(Some("not-a-number"), Some("also-invalid")),
        DEFAULT_JSONRPC_PORT,
    );
}

#[test]
fn jsonrpc_port_generic_env_var() {
    assert_eq!(resolve_jsonrpc_port(None, Some("5555")), 5555);
}

#[test]
fn tarpc_port_from_env() {
    assert_eq!(resolve_tarpc_port(Some("9999"), None), 9999);
}

#[test]
fn tarpc_port_default() {
    assert_eq!(resolve_tarpc_port(None, None), DEFAULT_TARPC_PORT);
}

#[test]
fn tarpc_port_invalid_loamspine_falls_back_to_generic() {
    assert_eq!(resolve_tarpc_port(Some("invalid"), Some("8888")), 8888);
}

#[test]
fn tarpc_port_generic_env_var() {
    assert_eq!(resolve_tarpc_port(None, Some("7777")), 7777);
}

#[test]
fn tarpc_port_invalid_both_falls_back_to_default() {
    assert_eq!(
        resolve_tarpc_port(Some("bad"), Some("worse")),
        DEFAULT_TARPC_PORT,
    );
}

// ── OS-assigned ports ────────────────────────────────────────────────

#[test]
fn os_assigned_ports_on_1() {
    assert!(resolve_use_os_assigned_ports(Some("1"), None));
}

#[test]
fn os_assigned_ports_off_0() {
    assert!(!resolve_use_os_assigned_ports(Some("0"), None));
}

#[test]
fn os_assigned_ports_yes() {
    assert!(resolve_use_os_assigned_ports(Some("yes"), None));
}

#[test]
fn os_assigned_ports_true() {
    assert!(resolve_use_os_assigned_ports(Some("true"), None));
}

#[test]
fn os_assigned_ports_loamspine_os_ports() {
    assert!(resolve_use_os_assigned_ports(None, Some("true")));
}

#[test]
fn os_assigned_ports_unset() {
    assert!(!resolve_use_os_assigned_ports(None, None));
}

// ── Actual ports ─────────────────────────────────────────────────────

#[test]
fn actual_ports_with_os_assignment() {
    assert_eq!(resolve_actual_jsonrpc_port(true, 8080), OS_ASSIGNED_PORT);
    assert_eq!(resolve_actual_tarpc_port(true, 9001), OS_ASSIGNED_PORT);
}

#[test]
fn actual_ports_without_os_assignment() {
    assert_eq!(resolve_actual_jsonrpc_port(false, 3333), 3333);
    assert_eq!(resolve_actual_tarpc_port(false, 4444), 4444);
}

// ── Bind address ─────────────────────────────────────────────────────

#[test]
fn bind_address_loamspine_specific() {
    assert_eq!(resolve_bind_address(Some("127.0.0.1"), None), "127.0.0.1");
}

#[test]
fn bind_address_generic() {
    assert_eq!(resolve_bind_address(None, Some("192.0.2.1")), "192.0.2.1");
}

#[test]
fn bind_address_default() {
    assert_eq!(
        resolve_bind_address(None, None),
        crate::constants::BIND_ALL_IPV4
    );
}

// ── Build endpoint ───────────────────────────────────────────────────

#[test]
fn build_endpoint_without_path() {
    assert_eq!(
        build_endpoint("http", "localhost", 8080, None),
        "http://localhost:8080",
    );
}

#[test]
fn build_endpoint_with_path() {
    assert_eq!(
        build_endpoint("http", "localhost", 8080, Some("/api")),
        "http://localhost:8080/api",
    );
}

// ── Env var name builders ────────────────────────────────────────────

#[test]
fn socket_env_var_formatting() {
    assert_eq!(socket_env_var("rhizoCrypt"), "RHIZOCRYPT_SOCKET");
    assert_eq!(socket_env_var("sweetGrass"), "SWEETGRASS_SOCKET");
    assert_eq!(socket_env_var("loamSpine"), "LOAMSPINE_SOCKET");
    assert_eq!(socket_env_var("bear-dog"), "BEAR_DOG_SOCKET");
}

#[test]
fn address_env_var_formatting() {
    assert_eq!(address_env_var("rhizoCrypt"), "RHIZOCRYPT_ADDRESS");
    assert_eq!(address_env_var("songbird"), "SONGBIRD_ADDRESS");
    assert_eq!(address_env_var("loamSpine"), "LOAMSPINE_ADDRESS");
}

// ── Socket base dir ──────────────────────────────────────────────────

#[test]
fn socket_base_dir_with_xdg() {
    let base = resolve_socket_base_dir_with(Some("/run/user/1000"));
    assert_eq!(base, std::path::PathBuf::from("/run/user/1000/biomeos"));
}

#[test]
fn socket_base_dir_fallback() {
    let base = resolve_socket_base_dir_with(None);
    assert!(
        base.to_string_lossy().ends_with("biomeos"),
        "got: {}",
        base.display(),
    );
}

// ── Primal socket resolution ─────────────────────────────────────────

#[test]
fn primal_socket_path() {
    let base = resolve_socket_base_dir_with(None);
    let path = resolve_primal_socket_from(&base, "loamspine", "default");
    assert!(
        path.to_string_lossy()
            .ends_with("biomeos/loamspine-default.sock"),
        "got: {}",
        path.display(),
    );
}

#[test]
fn primal_tarpc_socket_path() {
    let base = resolve_socket_base_dir_with(None);
    let path = resolve_primal_tarpc_socket_from(&base, "loamspine", "default");
    assert!(
        path.to_string_lossy()
            .ends_with("biomeos/loamspine-default.tarpc.sock"),
        "got: {}",
        path.display(),
    );
}

#[test]
fn primal_socket_with_xdg() {
    let base = resolve_socket_base_dir_with(Some("/run/user/1000"));
    let path = resolve_primal_socket_from(&base, "rhizocrypt", "myfamily");
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/rhizocrypt-myfamily.sock",
    );
}

#[test]
fn primal_socket_with_env_override() {
    let path = resolve_primal_socket_with(Some("/tmp/override.sock"), "testprimal", "dev");
    assert_eq!(path, std::path::PathBuf::from("/tmp/override.sock"));
}

#[test]
fn primal_socket_with_env_fallback() {
    let path = resolve_primal_socket_with(None, "testprimal", "dev");
    assert!(path.to_string_lossy().contains("testprimal-dev.sock"));
}

// ── Protocol negotiation ─────────────────────────────────────────────

#[test]
fn negotiate_protocol_prefers_tarpc_when_available() {
    let tmp = tempfile::tempdir().unwrap();
    let biomeos_dir = tmp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();
    std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();
    std::fs::write(biomeos_dir.join("testprimal-dev.tarpc.sock"), "").unwrap();

    let (protocol, path) = negotiate_protocol_from(&biomeos_dir, "testprimal", "dev");
    assert_eq!(protocol, IpcProtocol::Tarpc);
    assert!(path.to_string_lossy().contains("tarpc.sock"));
}

#[test]
fn negotiate_protocol_falls_back_to_jsonrpc() {
    let tmp = tempfile::tempdir().unwrap();
    let biomeos_dir = tmp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();
    std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();

    let (protocol, path) = negotiate_protocol_from(&biomeos_dir, "testprimal", "dev");
    assert_eq!(protocol, IpcProtocol::JsonRpc);
    assert!(!path.to_string_lossy().contains("tarpc"));
}

#[test]
fn ipc_protocol_equality() {
    assert_eq!(IpcProtocol::JsonRpc, IpcProtocol::JsonRpc);
    assert_eq!(IpcProtocol::Tarpc, IpcProtocol::Tarpc);
    assert_ne!(IpcProtocol::JsonRpc, IpcProtocol::Tarpc);
}
