// SPDX-License-Identifier: AGPL-3.0-only

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use loam_spine_core::entry::{Entry, EntryType, SpineConfig};
use loam_spine_core::types::{Did, SpineId, Timestamp};

/// Arbitrary input for entry creation fuzzing
#[derive(Debug, Arbitrary)]
struct EntryInput {
    owner_suffix: String,
    name: Option<String>,
    index: u64,
}

fuzz_target!(|input: EntryInput| {
    // Create a DID from arbitrary input
    let owner = Did::new(&format!("did:key:z6Mk{}", &input.owner_suffix));
    let spine_id = SpineId::now_v7();

    // Try creating a genesis entry
    let entry = Entry::genesis(owner.clone(), spine_id, SpineConfig::default());

    // Verify hash computation doesn't panic
    let hash = entry.compute_hash();
    assert_eq!(hash.len(), 32);

    // Verify serialization roundtrip
    if let Ok(serialized) = serde_json::to_string(&entry) {
        if let Ok(deserialized) = serde_json::from_str::<Entry>(&serialized) {
            // Hashes should match after roundtrip
            assert_eq!(entry.compute_hash(), deserialized.compute_hash());
        }
    }
});

