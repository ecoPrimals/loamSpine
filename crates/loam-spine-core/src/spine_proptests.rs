// SPDX-License-Identifier: AGPL-3.0-or-later

#[expect(clippy::unwrap_used, reason = "proptests use unwrap for assertions")]
mod inner {
    use super::super::*;
    use crate::entry::SpineConfig;
    use proptest::prelude::*;

    fn arb_owner() -> impl Strategy<Value = Did> {
        "[a-z]{3,8}".prop_map(|s| Did::new(format!("did:key:z6Mk{s}")))
    }

    proptest! {
        #[test]
        fn spine_genesis_height_is_one(owner in arb_owner()) {
            let spine = Spine::new(owner, None, SpineConfig::default()).unwrap();
            prop_assert_eq!(spine.height, 1, "new spine has genesis entry");
            prop_assert_eq!(spine.state, SpineState::Active);
        }

        #[test]
        fn spine_append_increments_height(owner in arb_owner(), n in 1u64..20) {
            let mut spine = Spine::new(owner.clone(), None, SpineConfig::default()).unwrap();
            for i in 0..n {
                let entry = crate::entry::Entry::new(
                    i + 1,
                    Some(spine.tip),
                    owner.clone(),
                    crate::entry::EntryType::DataAnchor {
                        data_hash: crate::types::ContentHash::default(),
                        mime_type: None,
                        size: i,
                    },
                ).with_spine_id(spine.id);
                spine.append(entry).unwrap();
            }
            prop_assert_eq!(spine.height, n + 1);
        }

        #[test]
        fn sealed_spine_rejects_entries(owner in arb_owner()) {
            let mut spine = Spine::new(owner.clone(), None, SpineConfig::default()).unwrap();
            spine.seal(Some("test".to_string())).unwrap();
            prop_assert!(spine.is_sealed());
            let entry = crate::entry::Entry::new(
                1, Some(spine.tip), owner,
                crate::entry::EntryType::DataAnchor {
                    data_hash: crate::types::ContentHash::default(),
                    mime_type: None, size: 0,
                },
            ).with_spine_id(spine.id);
            let result = spine.append(entry);
            prop_assert!(result.is_err(), "sealed spine must reject appends");
        }

        #[test]
        fn spine_name_preserved(owner in arb_owner(), name in "[a-z]{1,20}") {
            let spine = Spine::new(owner, Some(name.clone()), SpineConfig::default()).unwrap();
            prop_assert_eq!(spine.name.as_deref(), Some(name.as_str()));
        }
    }
}
