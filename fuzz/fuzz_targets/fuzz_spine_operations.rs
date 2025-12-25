#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use loam_spine_core::entry::SpineConfig;
use loam_spine_core::spine::{Spine, SpineBuilder, SpineState};
use loam_spine_core::types::Did;

/// Arbitrary input for spine operations fuzzing
#[derive(Debug, Arbitrary)]
struct SpineInput {
    owner_suffix: String,
    name: Option<String>,
    max_entries: Option<u32>,
    operations: Vec<SpineOperation>,
}

#[derive(Debug, Arbitrary)]
enum SpineOperation {
    GetHeight,
    GetTip,
    VerifyChain,
    IsSealed,
    IsActive,
    Seal,
}

fuzz_target!(|input: SpineInput| {
    // Create owner DID
    let owner = Did::new(&format!("did:key:z6Mk{}", &input.owner_suffix));

    // Build spine
    let mut builder = SpineBuilder::new(owner);
    if let Some(ref name) = input.name {
        builder = builder.name(name);
    }

    // Try to build - should always succeed with valid owner
    let mut spine = match builder.build() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Execute operations
    for op in input.operations.iter().take(100) {
        // Limit operations
        match op {
            SpineOperation::GetHeight => {
                let _ = spine.height;
            }
            SpineOperation::GetTip => {
                let _ = spine.tip;
            }
            SpineOperation::VerifyChain => {
                let _ = spine.verify();
            }
            SpineOperation::IsSealed => {
                let _ = matches!(spine.state, SpineState::Sealed { .. });
            }
            SpineOperation::IsActive => {
                let _ = matches!(spine.state, SpineState::Active);
            }
            SpineOperation::Seal => {
                let _ = spine.seal();
            }
        }
    }

    // Verify serialization roundtrip
    if let Ok(serialized) = serde_json::to_string(&spine) {
        let _ = serde_json::from_str::<Spine>(&serialized);
    }
});

