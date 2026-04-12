// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn error_display() {
    let spine_id = SpineId::nil();
    let err = LoamSpineError::SpineNotFound(spine_id);
    assert!(err.to_string().contains("spine not found"));
}

#[test]
fn chain_validation_error() {
    let err = LoamSpineError::ChainValidation {
        index: 42,
        reason: "hash mismatch".into(),
    };
    assert!(err.to_string().contains("42"));
    assert!(err.to_string().contains("hash mismatch"));
}

#[test]
fn capability_provider_error() {
    let err = LoamSpineError::capability_provider("signing", "HSM unavailable");
    assert!(err.to_string().contains("capability provider error"));
    assert!(err.to_string().contains("signing"));
    assert!(err.to_string().contains("HSM unavailable"));
}

#[test]
fn ipc_error_display_and_helper() {
    let err = LoamSpineError::ipc(IpcErrorPhase::Connect, "socket not found");
    assert!(err.to_string().contains("ipc error (connect)"));
    assert!(err.to_string().contains("socket not found"));
}

#[test]
fn ipc_phase_display() {
    assert_eq!(IpcErrorPhase::Connect.to_string(), "connect");
    assert_eq!(IpcErrorPhase::Write.to_string(), "write");
    assert_eq!(IpcErrorPhase::Read.to_string(), "read");
    assert_eq!(IpcErrorPhase::InvalidJson.to_string(), "invalid_json");
    assert_eq!(IpcErrorPhase::HttpStatus(404).to_string(), "http_404");
    assert_eq!(IpcErrorPhase::NoResult.to_string(), "no_result");
    assert_eq!(
        IpcErrorPhase::JsonRpcError(-32601).to_string(),
        "jsonrpc_-32601"
    );
    assert_eq!(IpcErrorPhase::Serialization.to_string(), "serialization");
}

#[test]
fn ipc_phase_backward_compat_alias() {
    let phase: IpcPhase = IpcErrorPhase::Connect;
    assert_eq!(phase.to_string(), "connect");
}

#[test]
fn is_recoverable_ipc_phases() {
    assert!(LoamSpineError::ipc(IpcErrorPhase::Connect, "timeout").is_recoverable());
    assert!(LoamSpineError::ipc(IpcErrorPhase::Write, "broken pipe").is_recoverable());
    assert!(LoamSpineError::ipc(IpcErrorPhase::Read, "eof").is_recoverable());
    assert!(LoamSpineError::ipc(IpcErrorPhase::HttpStatus(503), "unavail").is_recoverable());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::HttpStatus(404), "not found").is_recoverable());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::NoResult, "missing").is_recoverable());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::JsonRpcError(-32601), "method").is_recoverable());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::InvalidJson, "parse").is_recoverable());
}

#[test]
fn is_recoverable_other_variants() {
    assert!(LoamSpineError::Network("timeout".into()).is_recoverable());
    assert!(LoamSpineError::CapabilityUnavailable("signer".into()).is_recoverable());
    assert!(!LoamSpineError::Storage("corrupt".into()).is_recoverable());
    assert!(!LoamSpineError::Config("bad".into()).is_recoverable());
}

#[test]
fn is_method_not_found() {
    assert!(
        LoamSpineError::ipc(IpcErrorPhase::JsonRpcError(-32601), "not found").is_method_not_found()
    );
    assert!(
        !LoamSpineError::ipc(IpcErrorPhase::JsonRpcError(-32600), "other").is_method_not_found()
    );
    assert!(!LoamSpineError::ipc(IpcErrorPhase::Connect, "timeout").is_method_not_found());
    assert!(!LoamSpineError::Network("err".into()).is_method_not_found());
}

#[test]
fn dispatch_outcome_ok() {
    let outcome: DispatchOutcome<i32> = DispatchOutcome::Ok(42);
    assert!(outcome.is_ok());
    assert!(!outcome.is_application_error());
    assert_eq!(outcome.into_result().unwrap(), 42);
}

#[test]
fn dispatch_outcome_application_error() {
    let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
        code: -32601,
        message: "method not found".into(),
    };
    assert!(!outcome.is_ok());
    assert!(outcome.is_application_error());
    let err = outcome.into_result().unwrap_err();
    assert!(err.is_method_not_found());
}

#[test]
fn dispatch_outcome_protocol_error() {
    let outcome: DispatchOutcome<i32> =
        DispatchOutcome::ProtocolError(LoamSpineError::ipc(IpcErrorPhase::Connect, "refused"));
    assert!(!outcome.is_ok());
    assert!(!outcome.is_application_error());
    let err = outcome.into_result().unwrap_err();
    assert!(err.is_recoverable());
}

#[test]
fn extract_rpc_error_present() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "method not found" },
        "id": 1
    });
    let (code, msg) = extract_rpc_error(&response).unwrap();
    assert_eq!(code, -32601);
    assert_eq!(msg, "method not found");
}

#[test]
fn extract_rpc_error_absent() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": 42,
        "id": 1
    });
    assert!(extract_rpc_error(&response).is_none());
}

#[test]
fn extract_rpc_error_missing_fields() {
    let response = serde_json::json!({
        "error": {}
    });
    let (code, msg) = extract_rpc_error(&response).unwrap();
    assert_eq!(code, -1);
    assert_eq!(msg, "Unknown error");
}

#[test]
fn is_timeout_likely_phases() {
    assert!(LoamSpineError::ipc(IpcErrorPhase::Connect, "timeout").is_timeout_likely());
    assert!(LoamSpineError::ipc(IpcErrorPhase::Read, "timeout").is_timeout_likely());
    assert!(LoamSpineError::ipc(IpcErrorPhase::Write, "timeout").is_timeout_likely());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::InvalidJson, "parse").is_timeout_likely());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::JsonRpcError(-32601), "m").is_timeout_likely());
    assert!(!LoamSpineError::Network("err".into()).is_timeout_likely());
}

#[test]
fn is_application_error_phases() {
    assert!(
        LoamSpineError::ipc(IpcErrorPhase::JsonRpcError(-32601), "not found")
            .is_application_error()
    );
    assert!(
        LoamSpineError::ipc(IpcErrorPhase::JsonRpcError(-32000), "app err").is_application_error()
    );
    assert!(!LoamSpineError::ipc(IpcErrorPhase::Connect, "refused").is_application_error());
    assert!(!LoamSpineError::ipc(IpcErrorPhase::InvalidJson, "parse").is_application_error());
    assert!(!LoamSpineError::Network("err".into()).is_application_error());
}

#[test]
fn extract_rpc_result_success() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "spine_id": "abc-123" },
        "id": 1
    });
    let result = extract_rpc_result(&response).unwrap();
    assert_eq!(result["spine_id"], "abc-123");
}

#[test]
fn extract_rpc_result_error_response() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "method not found" },
        "id": 1
    });
    let err = extract_rpc_result(&response).unwrap_err();
    assert!(err.is_method_not_found());
}

#[test]
fn extract_rpc_result_missing_result() {
    let response = serde_json::json!({ "jsonrpc": "2.0", "id": 1 });
    let err = extract_rpc_result(&response).unwrap_err();
    assert!(matches!(
        err,
        LoamSpineError::Ipc {
            phase: IpcErrorPhase::NoResult,
            ..
        }
    ));
}

#[test]
fn extract_rpc_result_typed_success() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": 42,
        "id": 1
    });
    let val: i32 = extract_rpc_result_typed(&response).unwrap();
    assert_eq!(val, 42);
}

#[test]
fn extract_rpc_result_typed_wrong_type() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": "not a number",
        "id": 1
    });
    let err = extract_rpc_result_typed::<i32>(&response).unwrap_err();
    assert!(matches!(
        err,
        LoamSpineError::Ipc {
            phase: IpcErrorPhase::InvalidJson,
            ..
        }
    ));
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "proptest assertions use unwrap_err for error-path validation"
)]
#[expect(
    clippy::redundant_clone,
    reason = "proptest macro takes ownership; clone needed for subsequent assertions"
)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_ipc_phase() -> impl Strategy<Value = IpcErrorPhase> {
        prop_oneof![
            Just(IpcErrorPhase::Connect),
            Just(IpcErrorPhase::Write),
            Just(IpcErrorPhase::Read),
            Just(IpcErrorPhase::InvalidJson),
            (0u16..=999u16).prop_map(IpcErrorPhase::HttpStatus),
            Just(IpcErrorPhase::NoResult),
            any::<i64>().prop_map(IpcErrorPhase::JsonRpcError),
            Just(IpcErrorPhase::Serialization),
        ]
    }

    proptest! {
        #[test]
        fn ipc_phase_display_never_panics(phase in arb_ipc_phase()) {
            let s = phase.to_string();
            prop_assert!(!s.is_empty());
        }

        #[test]
        fn ipc_error_helpers_consistent(phase in arb_ipc_phase(), msg in ".*") {
            let err = LoamSpineError::ipc(phase.clone(), msg);
            if err.is_method_not_found() {
                prop_assert!(err.is_application_error());
            }
            if err.is_timeout_likely() {
                prop_assert!(err.is_recoverable());
            }
        }

        #[test]
        fn extract_rpc_error_never_panics(json_str in "\\PC{0,200}") {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                let _ = extract_rpc_error(&val);
            }
        }

        #[test]
        fn dispatch_outcome_into_result_consistent(code in any::<i64>(), msg in ".*") {
            let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
                code,
                message: msg,
            };
            let err = outcome.into_result().unwrap_err();
            prop_assert!(err.is_application_error());
        }
    }
}

mod storage_ext_tests {
    use super::*;
    use crate::error::StorageResultExt;

    #[test]
    fn storage_err_converts_ok() {
        let r: Result<i32, String> = Ok(42);
        assert_eq!(r.storage_err().unwrap(), 42);
    }

    #[test]
    fn storage_err_converts_error() {
        let r: Result<i32, String> = Err("disk full".into());
        let err = r.storage_err().unwrap_err();
        assert!(matches!(err, LoamSpineError::Storage(msg) if msg == "disk full"));
    }

    #[test]
    fn storage_ctx_adds_context() {
        let r: Result<(), &str> = Err("permission denied");
        let err = r.storage_ctx("write entry").unwrap_err();
        assert!(
            matches!(err, LoamSpineError::Storage(msg) if msg == "write entry: permission denied")
        );
    }

    #[test]
    fn storage_ctx_ok_passes_through() {
        let r: Result<&str, String> = Ok("data");
        assert_eq!(r.storage_ctx("ctx").unwrap(), "data");
    }
}
