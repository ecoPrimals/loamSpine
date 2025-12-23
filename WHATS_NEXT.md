# 🦴 LoamSpine — What's Next

**Last Updated**: December 22, 2025

---

## 🎯 Implementation Roadmap

### Phase 1: Entry Structure (Weeks 3-4)

**Goal**: Implement the fundamental ledger types.

> **Note**: LoamSpine development starts after RhizoCrypt core (Weeks 1-2).

#### Week 3: Entry Types
- [ ] Implement `EntryHash` type (Blake3)
- [ ] Implement `SpineId` type
- [ ] Implement core `LoamEntry` structure:
  ```rust
  pub struct LoamEntry {
      pub index: u64,
      pub previous: Option<EntryHash>,
      pub timestamp: Timestamp,
      pub entry_type: EntryType,
      pub committer: Did,
      pub signature: Signature,
      pub hash: EntryHash,
  }
  ```
- [ ] Implement `EntryType` enum:
  - `SessionCommit` — RhizoCrypt dehydration
  - `CertificateMint` — New certificate
  - `CertificateTransfer` — Ownership change
  - `CertificateLoan` — Temporary lending
  - `CertificateReturn` — Loan completion
  - `SpineAnchor` — Reference to another spine
- [ ] Add entry serialization
- [ ] Add entry hash computation
- [ ] Unit tests for entry creation

#### Week 4: Spine Structure
- [ ] Implement `Spine` structure:
  ```rust
  pub struct Spine {
      pub id: SpineId,
      pub owner: Did,
      pub entries: Vec<LoamEntry>,
      pub head: Option<EntryHash>,
      pub config: SpineConfig,
  }
  ```
- [ ] Implement spine creation
- [ ] Implement entry append (with chain validation)
- [ ] Add BearDog signing integration
- [ ] Unit tests for spine operations

---

### Phase 2: Certificate Model (Weeks 5-6)

**Goal**: Implement the ownership and lending system.

#### Week 5: Certificate Types
- [ ] Implement `Certificate` structure:
  ```rust
  pub struct Certificate {
      pub id: CertificateId,
      pub cert_type: CertificateType,
      pub mint_entry: EntryHash,
      pub current_entry: EntryHash,
      pub owner: Did,
      pub state: CertificateState,
  }
  ```
- [ ] Implement `CertificateType` enum:
  - `DigitalGameKey` — Game ownership
  - `DataProvenance` — Data origin proof
  - `Credential` — Identity credential
  - `Collectible` — NFT-like items
- [ ] Implement mint operation
- [ ] Unit tests for certificate creation

#### Week 6: Transfer & Lending
- [ ] Implement transfer operation
- [ ] Implement loan terms structure:
  ```rust
  pub struct LoanTerms {
      pub duration: Option<Duration>,
      pub auto_return: bool,
      pub conditions: Vec<LoanCondition>,
  }
  ```
- [ ] Implement loan operation
- [ ] Implement return operation
- [ ] Add loan expiration handling
- [ ] Unit tests for ownership changes

---

### Phase 3: RhizoCrypt Integration (Weeks 7-8)

**Goal**: Accept commits from RhizoCrypt.

#### Week 7: Commit Acceptance
- [ ] Implement `CommitReceiver` trait
- [ ] Implement `DehydrationSummary` validation
- [ ] Implement Merkle root storage
- [ ] Create commit entries in spine
- [ ] Add signature verification

#### Week 8: Query Interface
- [ ] Implement commit lookup by session ID
- [ ] Implement certificate lookup by ID
- [ ] Implement ownership history query
- [ ] Implement verification of Merkle proofs
- [ ] Integration tests with RhizoCrypt

---

### Phase 4: Storage Backend (Weeks 9-10)

**Goal**: Persistent spine storage.

#### Week 9: Spine Storage
- [ ] Implement `SpineStore` trait:
  ```rust
  pub trait SpineStore: Send + Sync {
      async fn create_spine(&self, owner: Did, config: SpineConfig) -> Result<SpineId>;
      async fn get_spine(&self, id: SpineId) -> Result<Option<Spine>>;
      async fn append_entry(&self, spine: SpineId, entry: LoamEntry) -> Result<()>;
      async fn get_entries(&self, spine: SpineId, range: Range) -> Result<Vec<LoamEntry>>;
  }
  ```
- [ ] Implement file-based storage
- [ ] Add indexing for efficient lookups
- [ ] Unit tests for persistence

#### Week 10: Federation Preparation
- [ ] Implement spine export (for replication)
- [ ] Implement spine import (from peers)
- [ ] Add entry validation for imports
- [ ] Add conflict detection

---

### Phase 5: SweetGrass Integration (Weeks 11-12)

**Goal**: Trigger Braid creation.

#### Week 11: Event Broadcasting
- [ ] Implement commit event emission
- [ ] Implement certificate event emission
- [ ] Add event subscription interface
- [ ] SweetGrass listener integration

#### Week 12: Integration & Testing
- [ ] End-to-end tests: RhizoCrypt → LoamSpine → SweetGrass
- [ ] Performance benchmarking
- [ ] Chaos testing
- [ ] Documentation completion
- [ ] Showcase demos

---

## 📊 Success Metrics

| Metric | Target |
|--------|--------|
| Entry append latency | < 10ms |
| Spine query latency | < 5ms |
| Certificate transfer | < 50ms |
| Storage efficiency | < 1KB per entry |
| Test coverage | > 80% |

---

## 🔗 Dependencies

### External
- `blake3` — Content addressing
- `serde` / `serde_json` — Serialization
- `tokio` — Async runtime

### Gen 1 Primals
- **BearDog** — Entry signing (Week 4)
- **Songbird** — Service discovery (Week 11)

### Phase 2 Siblings
- **RhizoCrypt** — Commit source (Week 7)
- **SweetGrass** — Braid consumer (Week 11)

---

## 📚 Reference Documents

- [specs/LOAMSPINE_SPECIFICATION.md](./specs/LOAMSPINE_SPECIFICATION.md) — Full specification
- [../ARCHITECTURE.md](../ARCHITECTURE.md) — Unified architecture
- [../INTEGRATION_OVERVIEW.md](../INTEGRATION_OVERVIEW.md) — Data flows

---

*LoamSpine: Where memories become permanent.*

