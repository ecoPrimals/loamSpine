// SPDX-License-Identifier: AGPL-3.0-only

//! Benchmarks for storage operations.
//!
//! Run with: `cargo bench --bench storage_ops`

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::unit_arg)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(missing_docs)]

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use loam_spine_core::{
    entry::{Entry, EntryType, SpineConfig},
    spine::Spine,
    storage::{EntryStorage, RedbStorage, SpineStorage},
    types::{Did, SessionId},
};
use tokio::runtime::Runtime;

fn bench_redb_spine_save(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    c.bench_function("redb_spine_save", |b| {
        b.iter(|| {
            let storage = RedbStorage::temporary().expect("Failed to create storage");
            let owner = Did::new(format!("did:key:z6Mk{}", uuid::Uuid::now_v7()));
            let spine = Spine::new(owner, Some("Bench".into()), SpineConfig::default())
                .expect("Failed to create spine");

            rt.block_on(async {
                black_box(
                    storage
                        .spines
                        .save_spine(&spine)
                        .await
                        .expect("Failed to save spine"),
                )
            })
        });
    });
}

fn bench_redb_spine_load(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let storage = RedbStorage::temporary().expect("Failed to create storage");
    let owner = Did::new("did:key:z6MkBenchOwner");
    let spine = Spine::new(owner, Some("Bench".into()), SpineConfig::default())
        .expect("Failed to create spine");
    let spine_id = spine.id;

    rt.block_on(async {
        storage
            .spines
            .save_spine(&spine)
            .await
            .expect("Failed to save spine");
    });

    c.bench_function("redb_spine_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    storage
                        .spines
                        .get_spine(spine_id)
                        .await
                        .expect("Failed to load spine"),
                )
            })
        });
    });
}

fn bench_redb_entry_save(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    c.bench_function("redb_entry_save", |b| {
        b.iter(|| {
            let storage = RedbStorage::temporary().expect("Failed to create storage");
            let owner = Did::new("did:key:z6MkBenchOwner");
            let spine = Spine::new(owner.clone(), Some("Bench".into()), SpineConfig::default())
                .expect("Failed to create spine");

            let mut entry = Entry::new(
                0,
                Some(spine.genesis),
                owner.clone(),
                EntryType::SessionCommit {
                    session_id: SessionId::now_v7(),
                    merkle_root: [0u8; 32],
                    vertex_count: 100,
                    committer: owner,
                },
            )
            .with_spine_id(spine.id);
            let _ = entry.hash();

            rt.block_on(async {
                black_box(
                    storage
                        .entries
                        .save_entry(&entry)
                        .await
                        .expect("Failed to save entry"),
                )
            })
        });
    });
}

fn bench_redb_entry_load(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let storage = RedbStorage::temporary().expect("Failed to create storage");
    let owner = Did::new("did:key:z6MkBenchOwner");
    let spine = Spine::new(owner.clone(), Some("Bench".into()), SpineConfig::default())
        .expect("Failed to create spine");

    let mut entry = Entry::new(
        0,
        Some(spine.genesis),
        owner.clone(),
        EntryType::SessionCommit {
            session_id: SessionId::now_v7(),
            merkle_root: [0u8; 32],
            vertex_count: 100,
            committer: owner,
        },
    )
    .with_spine_id(spine.id);
    let entry_hash = entry.hash().expect("compute hash");

    rt.block_on(async {
        storage
            .entries
            .save_entry(&entry)
            .await
            .expect("Failed to save entry");
    });

    c.bench_function("redb_entry_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    storage
                        .entries
                        .get_entry(entry_hash)
                        .await
                        .expect("Failed to load entry"),
                )
            })
        });
    });
}

fn bench_redb_bulk_entry_save(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    let mut group = c.benchmark_group("redb_bulk_entry_save");
    group.throughput(Throughput::Elements(100));

    group.bench_function("100_entries", |b| {
        b.iter(|| {
            let storage = RedbStorage::temporary().expect("Failed to create storage");
            let owner = Did::new("did:key:z6MkBenchOwner");
            let spine = Spine::new(owner.clone(), Some("Bench".into()), SpineConfig::default())
                .expect("Failed to create spine");

            rt.block_on(async {
                for i in 0..100 {
                    let mut entry = Entry::new(
                        i,
                        Some(spine.genesis),
                        owner.clone(),
                        EntryType::SessionCommit {
                            session_id: SessionId::now_v7(),
                            merkle_root: [i as u8; 32],
                            vertex_count: 100,
                            committer: owner.clone(),
                        },
                    )
                    .with_spine_id(spine.id);
                    let _ = entry.hash();
                    storage
                        .entries
                        .save_entry(&entry)
                        .await
                        .expect("Failed to save entry");
                }
                black_box(())
            })
        });
    });

    group.finish();
}

fn bench_redb_flush(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let storage = RedbStorage::temporary().expect("Failed to create storage");

    // Add some data first
    let owner = Did::new("did:key:z6MkBenchOwner");
    let spine = Spine::new(owner.clone(), Some("Bench".into()), SpineConfig::default())
        .expect("Failed to create spine");

    rt.block_on(async {
        storage
            .spines
            .save_spine(&spine)
            .await
            .expect("Failed to save spine");
    });

    c.bench_function("redb_flush", |b| {
        b.iter(|| black_box(storage.flush().expect("Failed to flush")));
    });
}

criterion_group!(
    benches,
    bench_redb_spine_save,
    bench_redb_spine_load,
    bench_redb_entry_save,
    bench_redb_entry_load,
    bench_redb_bulk_entry_save,
    bench_redb_flush,
);
criterion_main!(benches);
