//! Benchmarks for core `LoamSpine` operations.
//!
//! Run with: `cargo bench`

// Benchmarks allow patterns that would be problematic in production
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use loam_spine_core::{
    service::LoamSpineService,
    traits::{CommitAcceptor, DehydrationSummary},
    types::{Did, SessionId},
};
use tokio::runtime::Runtime;

fn bench_spine_creation(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let service = LoamSpineService::new();

    c.bench_function("create_spine", |b| {
        b.iter(|| {
            let owner = Did::new(format!("did:key:z6Mk{}", uuid::Uuid::now_v7()));
            rt.block_on(async {
                black_box(
                    service
                        .ensure_spine(owner, Some("Bench".into()))
                        .await
                        .expect("Failed to create spine"),
                )
            })
        });
    });
}

fn bench_session_commit(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkBenchOwner");

    let spine_id = rt.block_on(async {
        service
            .ensure_spine(owner.clone(), Some("Bench Spine".into()))
            .await
            .expect("Failed to create spine")
    });

    c.bench_function("commit_session", |b| {
        b.iter(|| {
            let summary = DehydrationSummary::new(SessionId::now_v7(), "bench", [0u8; 32])
                .with_vertex_count(100);

            rt.block_on(async {
                black_box(
                    service
                        .commit_session(spine_id, owner.clone(), summary)
                        .await
                        .expect("Failed to commit"),
                )
            })
        });
    });
}

fn bench_session_commit_throughput(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkBenchOwner2");

    let spine_id = rt.block_on(async {
        service
            .ensure_spine(owner.clone(), Some("Throughput Bench".into()))
            .await
            .expect("Failed to create spine")
    });

    let mut group = c.benchmark_group("commit_throughput");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_commit", |b| {
        b.iter(|| {
            let summary = DehydrationSummary::new(SessionId::now_v7(), "bench", [0u8; 32]);

            rt.block_on(async {
                black_box(
                    service
                        .commit_session(spine_id, owner.clone(), summary)
                        .await
                        .expect("Failed to commit"),
                )
            })
        });
    });

    group.finish();
}

fn bench_entry_hashing(c: &mut Criterion) {
    use loam_spine_core::types::hash_bytes;

    let data_small = vec![0u8; 64];
    let data_medium = vec![0u8; 1024];
    let data_large = vec![0u8; 65536];

    let mut group = c.benchmark_group("hash_bytes");

    group.bench_function("64B", |b| {
        b.iter(|| black_box(hash_bytes(&data_small)));
    });

    group.bench_function("1KB", |b| {
        b.iter(|| black_box(hash_bytes(&data_medium)));
    });

    group.bench_function("64KB", |b| {
        b.iter(|| black_box(hash_bytes(&data_large)));
    });

    group.finish();
}

fn bench_certificate_mint(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkBenchCertOwner");

    let spine_id = rt.block_on(async {
        service
            .ensure_spine(owner.clone(), Some("Cert Bench".into()))
            .await
            .expect("Failed to create spine")
    });

    c.bench_function("mint_certificate", |b| {
        b.iter(|| {
            let cert_type = loam_spine_core::CertificateType::DigitalCollectible {
                collection_id: "bench-collection".to_string(),
                item_number: Some(1),
                total_supply: None,
                rarity: None,
            };
            rt.block_on(async {
                black_box(
                    service
                        .mint_certificate(spine_id, cert_type, owner.clone(), None)
                        .await
                        .expect("Failed to mint"),
                )
            })
        });
    });
}

criterion_group!(
    benches,
    bench_spine_creation,
    bench_session_commit,
    bench_session_commit_throughput,
    bench_entry_hashing,
    bench_certificate_mint,
);
criterion_main!(benches);
