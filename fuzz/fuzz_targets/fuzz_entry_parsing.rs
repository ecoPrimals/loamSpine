// SPDX-License-Identifier: AGPL-3.0-only

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use loam_spine_core::entry::{Entry, SpineConfig};
use loam_spine_core::types::{Did, SpineId};

#[derive(Debug, Arbitrary)]
struct EntryInput {
    owner_suffix: String,
    name: Option<String>,
    index: u64,
}

fuzz_target!(|input: EntryInput| {
    let owner = Did::new(&format!("did:key:z6Mk{}", &input.owner_suffix));
    let spine_id = SpineId::now_v7();

    let entry = Entry::genesis(owner.clone(), spine_id, SpineConfig::default());

    if let Ok(hash) = entry.compute_hash() {
        assert_eq!(hash.len(), 32);
    }

    if let Ok(serialized) = serde_json::to_string(&entry) {
        if let Ok(deserialized) = serde_json::from_str::<Entry>(&serialized) {
            if let (Ok(h1), Ok(h2)) = (entry.compute_hash(), deserialized.compute_hash()) {
                assert_eq!(h1, h2);
            }
        }
    }
});
