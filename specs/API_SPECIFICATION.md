# LoamSpine — API Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

LoamSpine exposes two API interfaces:
- **gRPC API** — High-performance, strongly-typed
- **REST API** — HTTP/JSON for compatibility

Both APIs are exposed through Songbird's Universal Port Authority (UPA).

---

## 2. gRPC API

### 2.1 Service Definition

```protobuf
syntax = "proto3";

package loamspine.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

// The LoamSpine service
service LoamSpine {
    // ==================== Spine Management ====================
    
    // Create a new spine
    rpc CreateSpine(CreateSpineRequest) returns (CreateSpineResponse);
    
    // Get spine details
    rpc GetSpine(GetSpineRequest) returns (GetSpineResponse);
    
    // List spines
    rpc ListSpines(ListSpinesRequest) returns (ListSpinesResponse);
    
    // Update spine configuration
    rpc UpdateSpine(UpdateSpineRequest) returns (UpdateSpineResponse);
    
    // Seal spine (make read-only)
    rpc SealSpine(SealSpineRequest) returns (SealSpineResponse);
    
    // Archive spine (move to cold storage)
    rpc ArchiveSpine(ArchiveSpineRequest) returns (ArchiveSpineResponse);
    
    // ==================== Entry Operations ====================
    
    // Append an entry
    rpc AppendEntry(AppendEntryRequest) returns (AppendEntryResponse);
    
    // Get entry by hash
    rpc GetEntry(GetEntryRequest) returns (GetEntryResponse);
    
    // Get entry by index
    rpc GetEntryByIndex(GetEntryByIndexRequest) returns (GetEntryResponse);
    
    // Get entries in range
    rpc GetEntryRange(GetEntryRangeRequest) returns (stream Entry);
    
    // Get spine tip
    rpc GetTip(GetTipRequest) returns (GetTipResponse);
    
    // Query entries
    rpc QueryEntries(QueryEntriesRequest) returns (stream Entry);
    
    // ==================== Certificate Operations ====================
    
    // Mint a new certificate
    rpc MintCertificate(MintCertificateRequest) returns (MintCertificateResponse);
    
    // Get certificate
    rpc GetCertificate(GetCertificateRequest) returns (GetCertificateResponse);
    
    // Transfer certificate
    rpc TransferCertificate(TransferCertificateRequest) returns (TransferCertificateResponse);
    
    // Loan certificate
    rpc LoanCertificate(LoanCertificateRequest) returns (LoanCertificateResponse);
    
    // Return loaned certificate
    rpc ReturnCertificate(ReturnCertificateRequest) returns (ReturnCertificateResponse);
    
    // Get certificate history
    rpc GetCertificateHistory(GetCertificateHistoryRequest) returns (GetCertificateHistoryResponse);
    
    // Verify certificate
    rpc VerifyCertificate(VerifyCertificateRequest) returns (VerifyCertificateResponse);
    
    // ==================== Waypoint Operations ====================
    
    // Anchor slice at waypoint
    rpc AnchorSlice(AnchorSliceRequest) returns (AnchorSliceResponse);
    
    // Record waypoint operation
    rpc RecordWaypointOperation(RecordWaypointOperationRequest) returns (RecordWaypointOperationResponse);
    
    // Depart slice from waypoint
    rpc DepartSlice(DepartSliceRequest) returns (DepartSliceResponse);
    
    // Get active anchors
    rpc GetActiveAnchors(GetActiveAnchorsRequest) returns (GetActiveAnchorsResponse);
    
    // ==================== Proof Operations ====================
    
    // Generate inclusion proof
    rpc GenerateInclusionProof(GenerateInclusionProofRequest) returns (GenerateInclusionProofResponse);
    
    // Verify inclusion proof
    rpc VerifyInclusionProof(VerifyInclusionProofRequest) returns (VerifyInclusionProofResponse);
    
    // Generate certificate provenance proof
    rpc GenerateProvenanceProof(GenerateProvenanceProofRequest) returns (GenerateProvenanceProofResponse);
    
    // ==================== Replication ====================
    
    // Get sync status
    rpc GetSyncStatus(GetSyncStatusRequest) returns (GetSyncStatusResponse);
    
    // Request sync
    rpc RequestSync(RequestSyncRequest) returns (RequestSyncResponse);
    
    // Stream entries (for replication)
    rpc StreamEntries(StreamEntriesRequest) returns (stream Entry);
    
    // ==================== Health ====================
    
    // Health check
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
    
    // Get metrics
    rpc GetMetrics(GetMetricsRequest) returns (GetMetricsResponse);
}
```

### 2.2 Message Definitions

```protobuf
// ==================== Common Types ====================

message SpineId {
    string uuid = 1;
}

message EntryHash {
    bytes hash = 1;  // 32-byte Blake3
}

message CertificateId {
    string uuid = 1;
}

message SliceId {
    string uuid = 1;
}

message Did {
    string value = 1;  // did:key:z6Mk...
}

message Signature {
    bytes value = 1;
}

message PayloadRef {
    bytes hash = 1;
    uint64 size = 2;
    optional string mime_type = 3;
}

// ==================== Spine Messages ====================

message CreateSpineRequest {
    SpineType spine_type = 1;
    optional string name = 2;
    SpineConfig config = 3;
    Did owner = 4;
}

message CreateSpineResponse {
    Spine spine = 1;
    Entry genesis_entry = 2;
}

message SpineType {
    oneof type {
        PersonalSpine personal = 1;
        ProfessionalSpine professional = 2;
        CommunitySpine community = 3;
        WaypointSpine waypoint = 4;
        PublicSpine public = 5;
        CustomSpine custom = 6;
    }
}

message PersonalSpine {}
message ProfessionalSpine {}
message CommunitySpine { string community_id = 1; }
message WaypointSpine { optional uint32 max_anchor_depth = 1; }
message PublicSpine {}
message CustomSpine { string type_name = 1; }

message SpineConfig {
    ReplicationPolicy replication = 1;
    AccessPolicy read_access = 2;
    AccessPolicy write_access = 3;
    AccessPolicy admin_access = 4;
    optional uint64 auto_rollup_threshold = 5;
    optional WaypointConfig waypoint = 6;
}

message ReplicationPolicy {
    oneof policy {
        bool none = 1;
        PeersReplication peers = 2;
        FederationReplication federation = 3;
        bool full = 4;
    }
}

message PeersReplication {
    repeated string peers = 1;
    uint32 min_copies = 2;
}

message FederationReplication {
    uint32 min_copies = 1;
    bool prefer_geographic_distribution = 2;
}

message AccessPolicy {
    oneof policy {
        bool owner = 1;
        AllowList allow_list = 2;
        DenyList deny_list = 3;
        bool public = 4;
        string capability = 5;
    }
}

message AllowList { repeated Did dids = 1; }
message DenyList { repeated Did dids = 1; }

message WaypointConfig {
    bool accept_anchors = 1;
    optional uint32 max_anchored_slices = 2;
    optional uint32 max_anchor_depth = 3;
    repeated SpineId allowed_origins = 4;
    PropagationPolicy propagation_policy = 5;
    bool auto_return_expired = 6;
    google.protobuf.Duration expiry_grace_period = 7;
}

enum PropagationPolicy {
    PROPAGATION_NEVER = 0;
    PROPAGATION_SUMMARY_ONLY = 1;
    PROPAGATION_SELECTIVE = 2;
    PROPAGATION_FULL = 3;
}

message Spine {
    SpineId id = 1;
    optional string name = 2;
    Did owner = 3;
    SpineType spine_type = 4;
    SpineConfig config = 5;
    EntryHash genesis = 6;
    EntryHash tip = 7;
    uint64 height = 8;
    uint64 created_at = 9;
    uint64 updated_at = 10;
    SpineState state = 11;
}

message SpineState {
    oneof state {
        bool active = 1;
        FrozenState frozen = 2;
        SealedState sealed = 3;
        ArchivedState archived = 4;
    }
}

message FrozenState { string reason = 1; optional uint64 until = 2; }
message SealedState { uint64 sealed_at = 1; EntryHash final_entry = 2; }
message ArchivedState { uint64 archived_at = 1; string archive_location = 2; }

// ==================== Entry Messages ====================

message AppendEntryRequest {
    SpineId spine_id = 1;
    EntryType entry_type = 2;
    optional bytes payload = 3;
    map<string, string> metadata = 4;
    repeated Attestation attestations = 5;
}

message AppendEntryResponse {
    EntryHash entry_hash = 1;
    uint64 index = 2;
    Entry entry = 3;
}

message Entry {
    EntryHash hash = 1;
    uint64 index = 2;
    optional EntryHash previous = 3;
    uint64 timestamp = 4;
    Did committer = 5;
    EntryType entry_type = 6;
    optional PayloadRef payload = 7;
    map<string, string> metadata = 8;
    Signature signature = 9;
    repeated Attestation attestations = 10;
}

message EntryType {
    oneof type {
        GenesisEntry genesis = 1;
        MetadataUpdateEntry metadata_update = 2;
        SpineSealedEntry spine_sealed = 3;
        SessionCommitEntry session_commit = 4;
        DataAnchorEntry data_anchor = 5;
        CertificateMintEntry certificate_mint = 6;
        CertificateTransferEntry certificate_transfer = 7;
        CertificateLoanEntry certificate_loan = 8;
        CertificateReturnEntry certificate_return = 9;
        SliceCheckoutEntry slice_checkout = 10;
        SliceAnchorEntry slice_anchor = 11;
        SliceOperationEntry slice_operation = 12;
        SliceDepartureEntry slice_departure = 13;
        SliceReturnEntry slice_return = 14;
        AttestationEntry attestation = 15;
        RevocationEntry revocation = 16;
        CustomEntry custom = 17;
    }
}

message GenesisEntry {
    SpineId spine_id = 1;
    Did owner = 2;
    SpineConfig config = 3;
}

message SessionCommitEntry {
    string session_id = 1;
    string session_type = 2;
    bytes merkle_root = 3;
    bytes summary = 4;  // Serialized DehydrationSummary
}

message CertificateMintEntry {
    CertificateId cert_id = 1;
    CertificateType cert_type = 2;
    Did initial_owner = 3;
    CertificateMetadata metadata = 4;
}

message CertificateTransferEntry {
    CertificateId cert_id = 1;
    Did from = 2;
    Did to = 3;
    optional TransferConditions conditions = 4;
}

message CertificateLoanEntry {
    CertificateId cert_id = 1;
    Did lender = 2;
    Did borrower = 3;
    LoanTerms terms = 4;
}

message CertificateReturnEntry {
    CertificateId cert_id = 1;
    EntryHash loan_entry = 2;
    optional UsageSummary usage_summary = 3;
}

message SliceAnchorEntry {
    SliceId slice_id = 1;
    SpineId origin_spine = 2;
    EntryHash origin_entry = 3;
    SliceTerms terms = 4;
}

message SliceOperationEntry {
    SliceId slice_id = 1;
    SliceOperationType operation = 2;
    optional PayloadRef payload = 3;
}

message SliceDepartureEntry {
    SliceId slice_id = 1;
    DepartureReason reason = 2;
    WaypointSummary summary = 3;
}

message Attestation {
    Did attester = 1;
    bytes data_hash = 2;
    Signature signature = 3;
    uint64 timestamp = 4;
}

// ==================== Certificate Messages ====================

message MintCertificateRequest {
    SpineId spine_id = 1;
    CertificateType cert_type = 2;
    Did initial_owner = 3;
    CertificateMetadata metadata = 4;
}

message MintCertificateResponse {
    Certificate certificate = 1;
    EntryHash mint_entry = 2;
}

message Certificate {
    CertificateId id = 1;
    CertificateType cert_type = 2;
    uint32 version = 3;
    Did owner = 4;
    optional Did holder = 5;
    MintInfo mint_info = 6;
    CertificateLocation current_location = 7;
    CertificateState state = 8;
    uint64 transfer_count = 9;
    optional LoanInfo active_loan = 10;
    CertificateMetadata metadata = 11;
    uint64 created_at = 12;
    uint64 updated_at = 13;
}

message CertificateType {
    oneof type {
        DigitalGameCert digital_game = 1;
        GameItemCert game_item = 2;
        DigitalCollectibleCert digital_collectible = 3;
        SoftwareLicenseCert software_license = 4;
        AcademicCredentialCert academic_credential = 5;
        ProfessionalLicenseCert professional_license = 6;
        CustomCert custom = 99;
    }
}

message DigitalGameCert {
    string platform = 1;
    string game_id = 2;
    optional string edition = 3;
}

message GameItemCert {
    string game_id = 1;
    string item_type = 2;
    string item_id = 3;
    map<string, string> attributes = 4;
}

message DigitalCollectibleCert {
    string collection_id = 1;
    optional uint64 item_number = 2;
    optional uint64 total_supply = 3;
    optional string rarity = 4;
}

message SoftwareLicenseCert {
    string software_id = 1;
    string license_type = 2;
    optional uint32 seats = 3;
}

message AcademicCredentialCert {
    string institution = 1;
    string credential_type = 2;
    string field_of_study = 3;
    uint64 date_awarded = 4;
}

message ProfessionalLicenseCert {
    string issuing_authority = 1;
    string license_type = 2;
    string license_number = 3;
    string jurisdiction = 4;
}

message CustomCert {
    string type_uri = 1;
    uint32 schema_version = 2;
}

message CertificateMetadata {
    optional string name = 1;
    optional string description = 2;
    optional PayloadRef image = 3;
    optional string external_url = 4;
    repeated MetadataAttribute attributes = 5;
}

message MetadataAttribute {
    string trait_type = 1;
    string value = 2;
    optional string display_type = 3;
}

message MintInfo {
    Did minter = 1;
    SpineId spine = 2;
    EntryHash entry = 3;
    uint64 timestamp = 4;
}

message CertificateLocation {
    SpineId spine = 1;
    EntryHash entry = 2;
    uint64 index = 3;
}

message CertificateState {
    oneof state {
        bool active = 1;
        LoanedState loaned = 2;
        PendingTransferState pending_transfer = 3;
        RevokedState revoked = 4;
        ExpiredState expired = 5;
    }
}

message LoanedState { EntryHash loan_entry = 1; }
message PendingTransferState { EntryHash transfer_entry = 1; Did to = 2; }
message RevokedState { EntryHash revoke_entry = 1; string reason = 2; }
message ExpiredState { uint64 expired_at = 1; }

message LoanInfo {
    EntryHash loan_entry = 1;
    Did borrower = 2;
    LoanTerms terms = 3;
    uint64 started_at = 4;
    optional uint64 expires_at = 5;
    optional SpineId waypoint = 6;
}

message LoanTerms {
    optional google.protobuf.Duration duration = 1;
    optional google.protobuf.Duration grace_period = 2;
    bool auto_return = 3;
    repeated string allowed_operations = 4;
    repeated string forbidden_operations = 5;
    bool allow_subloan = 6;
    optional uint32 max_subloan_depth = 7;
    PropagationPolicy propagation_policy = 8;
}

message TransferConditions {
    optional PaymentCondition payment = 1;
    repeated Did required_attestations = 2;
    optional uint64 time_lock = 3;
    optional uint64 expiry = 4;
}

message PaymentCondition {
    string currency = 1;
    string amount = 2;
    Did recipient = 3;
}

message TransferCertificateRequest {
    CertificateId cert_id = 1;
    Did to = 2;
    optional TransferConditions conditions = 3;
}

message TransferCertificateResponse {
    Certificate certificate = 1;
    EntryHash transfer_entry = 2;
}

message LoanCertificateRequest {
    CertificateId cert_id = 1;
    Did borrower = 2;
    LoanTerms terms = 3;
}

message LoanCertificateResponse {
    Certificate certificate = 1;
    EntryHash loan_entry = 2;
}

message ReturnCertificateRequest {
    CertificateId cert_id = 1;
    optional UsageSummary usage_summary = 2;
}

message ReturnCertificateResponse {
    Certificate certificate = 1;
    EntryHash return_entry = 2;
}

message UsageSummary {
    uint64 duration_nanos = 1;
    uint64 operation_count = 2;
    repeated string operation_types = 3;
    optional WaypointSummary waypoint_summary = 4;
}

message WaypointSummary {
    SliceId slice_id = 1;
    uint64 duration_nanos = 2;
    uint64 operation_count = 3;
    repeated string operation_types = 4;
    bytes operations_hash = 5;
    bool was_relent = 6;
    uint32 max_relend_depth = 7;
}

// ==================== Waypoint Messages ====================

message AnchorSliceRequest {
    SliceId slice_id = 1;
    SpineId origin_spine = 2;
    EntryHash origin_entry = 3;
    Did owner = 4;
    uint32 current_depth = 5;
    SliceTerms terms = 6;
    SpineId waypoint_spine = 7;
}

message SliceTerms {
    optional google.protobuf.Duration duration = 1;
    optional google.protobuf.Duration grace_period = 2;
    repeated string allowed_operations = 3;
    repeated string forbidden_operations = 4;
    bool allow_relend = 5;
    optional uint32 max_relend_depth = 6;
    PropagationPolicy propagation = 7;
}

message AnchorSliceResponse {
    ActiveAnchor anchor = 1;
    EntryHash anchor_entry = 2;
}

message ActiveAnchor {
    SliceId slice_id = 1;
    EntryHash anchor_entry = 2;
    SpineId origin_spine = 3;
    EntryHash origin_entry = 4;
    Did owner = 5;
    uint32 depth = 6;
    SliceTerms terms = 7;
    uint64 anchored_at = 8;
    optional uint64 expires_at = 9;
    AnchorState state = 10;
}

enum AnchorState {
    ANCHOR_ACTIVE = 0;
    ANCHOR_EXPIRED = 1;
    ANCHOR_DEPARTING = 2;
    ANCHOR_DEPARTED = 3;
}

message SliceOperationType {
    oneof type {
        UseOperation use = 1;
        ViewOperation view = 2;
        GameOperation game = 3;
        DocumentOperation document = 4;
        CustomOperation custom = 99;
    }
}

message UseOperation { string action = 1; optional uint64 duration_nanos = 2; }
message ViewOperation { optional string viewport = 1; }
message GameOperation { string operation = 1; map<string, string> details = 2; }
message DocumentOperation { string operation = 1; optional string format = 2; }
message CustomOperation { string name = 1; map<string, string> metadata = 2; }

message DepartSliceRequest {
    SliceId slice_id = 1;
    DepartureReason reason = 2;
}

message DepartureReason {
    oneof reason {
        bool expired = 1;
        bool manual_return = 2;
        bool owner_recall = 3;
        RelendReason relend = 4;
        SessionResolutionReason session_resolution = 5;
        string administrative = 6;
    }
}

message RelendReason { SpineId target_waypoint = 1; }
message SessionResolutionReason { string session_id = 1; string outcome = 2; }

message DepartSliceResponse {
    EntryHash departure_entry = 1;
    WaypointSummary summary = 2;
}

// ==================== Proof Messages ====================

message GenerateInclusionProofRequest {
    SpineId spine_id = 1;
    EntryHash entry_hash = 2;
}

message GenerateInclusionProofResponse {
    InclusionProof proof = 1;
}

message InclusionProof {
    Entry entry = 1;
    EntryHash entry_hash = 2;
    repeated EntryHash path = 3;
    EntryHash tip = 4;
    SpineId spine_id = 5;
    uint64 timestamp = 6;
    optional Signature owner_attestation = 7;
}

message VerifyInclusionProofRequest {
    InclusionProof proof = 1;
}

message VerifyInclusionProofResponse {
    bool valid = 1;
}

message GenerateProvenanceProofRequest {
    CertificateId cert_id = 1;
}

message GenerateProvenanceProofResponse {
    ProvenanceProof proof = 1;
}

message ProvenanceProof {
    CertificateId cert_id = 1;
    Did current_owner = 2;
    InclusionProof mint_proof = 3;
    repeated InclusionProof transfer_proofs = 4;
    InclusionProof current_proof = 5;
    HistorySummary history_summary = 6;
    uint64 timestamp = 7;
}

message HistorySummary {
    uint64 transfer_count = 1;
    uint64 loan_count = 2;
    uint64 age_nanos = 3;
}

// ==================== Health Messages ====================

message HealthCheckRequest {}

message HealthCheckResponse {
    string status = 1;
    repeated ComponentHealth components = 2;
    map<string, string> metrics = 3;
}

message ComponentHealth {
    string name = 1;
    string status = 2;
    optional string message = 3;
}
```

---

## 3. REST API

### 3.1 OpenAPI Specification (Summary)

```yaml
openapi: 3.0.3
info:
  title: LoamSpine API
  version: 0.2.0

servers:
  - url: /api/v1/loamspine

paths:
  # Spines
  /spines:
    post:
      summary: Create spine
    get:
      summary: List spines
  
  /spines/{spine_id}:
    get:
      summary: Get spine
    patch:
      summary: Update spine
  
  /spines/{spine_id}/seal:
    post:
      summary: Seal spine
  
  /spines/{spine_id}/entries:
    post:
      summary: Append entry
    get:
      summary: Query entries
  
  /spines/{spine_id}/entries/{entry_hash}:
    get:
      summary: Get entry by hash
  
  /spines/{spine_id}/entries/index/{index}:
    get:
      summary: Get entry by index
  
  /spines/{spine_id}/tip:
    get:
      summary: Get tip entry
  
  # Certificates
  /certificates:
    post:
      summary: Mint certificate
  
  /certificates/{cert_id}:
    get:
      summary: Get certificate
  
  /certificates/{cert_id}/transfer:
    post:
      summary: Transfer certificate
  
  /certificates/{cert_id}/loan:
    post:
      summary: Loan certificate
  
  /certificates/{cert_id}/return:
    post:
      summary: Return certificate
  
  /certificates/{cert_id}/history:
    get:
      summary: Get certificate history
  
  /certificates/{cert_id}/verify:
    get:
      summary: Verify certificate
  
  /certificates/{cert_id}/provenance:
    get:
      summary: Get provenance proof
  
  # Waypoints
  /waypoints/{spine_id}/anchors:
    post:
      summary: Anchor slice
    get:
      summary: List anchors
  
  /waypoints/{spine_id}/anchors/{slice_id}:
    get:
      summary: Get anchor
  
  /waypoints/{spine_id}/anchors/{slice_id}/operations:
    post:
      summary: Record operation
  
  /waypoints/{spine_id}/anchors/{slice_id}/depart:
    post:
      summary: Depart slice
  
  # Proofs
  /proofs/inclusion:
    post:
      summary: Generate inclusion proof
  
  /proofs/inclusion/verify:
    post:
      summary: Verify inclusion proof
  
  # Health
  /health:
    get:
      summary: Health check
  
  /metrics:
    get:
      summary: Prometheus metrics
```

---

## 4. Authentication

All API calls require BearDog authentication:

```
Authorization: Bearer <beardog-token>
X-BearDog-DID: did:key:z6Mk...
```

---

## 5. Rate Limiting

| Endpoint Category | Rate Limit |
|-------------------|------------|
| Spine management | 100 req/min |
| Entry operations | 1,000 req/min |
| Certificate operations | 500 req/min |
| Waypoint operations | 500 req/min |
| Proof generation | 100 req/min |

---

## 6. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) — Primal integrations

---

*LoamSpine: The permanent record that gives memory its meaning.*

