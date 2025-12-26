# 📚 LoamSpine Documentation Index

**Version**: 0.7.0-dev  
**Last Updated**: December 25, 2025  
**Status**: Production Ready

---

## 🚀 GETTING STARTED

### For New Users
1. **[START_HERE.md](START_HERE.md)** — Your first stop! Quick start guide
2. **[README.md](README.md)** — Project overview, features, and architecture
3. **[CONTRIBUTING.md](CONTRIBUTING.md)** — How to contribute to LoamSpine

### Quick Navigation
- 🎯 Current status → [STATUS.md](STATUS.md)
- 🗺️ Future plans → [WHATS_NEXT.md](WHATS_NEXT.md) & [ROADMAP_V0.8.0.md](ROADMAP_V0.8.0.md)
- 📊 Quality metrics → [AUDIT_SUMMARY.md](AUDIT_SUMMARY.md)
- 🔗 Integration status → [INTEGRATION_GAPS.md](INTEGRATION_GAPS.md)

---

## 📊 CURRENT STATUS

### Project Health (v0.7.0-dev)
- **[STATUS.md](STATUS.md)** — Current project status and metrics
- **[AUDIT_SUMMARY.md](AUDIT_SUMMARY.md)** — Latest quality audit (A grade, 95/100)
- **[INTEGRATION_GAPS.md](INTEGRATION_GAPS.md)** — Integration gap tracking (all resolved)

### Key Metrics
- ✅ **372 tests** passing (90.39% coverage)
- ✅ **Zero unsafe code**
- ✅ **Zero clippy warnings**
- ✅ **Infant discovery** complete
- ✅ **30% hardcoding reduction**

---

## 🗺️ PLANNING & ROADMAP

### Current & Future
- **[ROADMAP_V0.8.0.md](ROADMAP_V0.8.0.md)** — v0.8.0 implementation plan (DNS SRV + mDNS)
- **[WHATS_NEXT.md](WHATS_NEXT.md)** — High-level future direction

### Technical Planning
All planning documents are in `docs/planning/`:
- **[EXECUTIVE_SUMMARY.md](docs/planning/EXECUTIVE_SUMMARY.md)** — Project executive summary
- **[KNOWN_ISSUES.md](docs/planning/KNOWN_ISSUES.md)** — Known issues and limitations
- **[REFACTORING_RECOMMENDATIONS.md](docs/planning/REFACTORING_RECOMMENDATIONS.md)** — Future refactoring ideas
- **[SHOWCASE_EVOLUTION_PLAN.md](docs/planning/SHOWCASE_EVOLUTION_PLAN.md)** — Showcase development plan
- **[SHOWCASE_STATUS.md](docs/planning/SHOWCASE_STATUS.md)** — Current showcase status
- **[ZERO_COPY_MIGRATION_PLAN.md](docs/planning/ZERO_COPY_MIGRATION_PLAN.md)** — Zero-copy RPC migration plan

---

## 📖 SPECIFICATIONS

### Core Specifications (`specs/`)
Complete technical specifications for LoamSpine:

1. **[00_SPECIFICATIONS_INDEX.md](specs/00_SPECIFICATIONS_INDEX.md)** — Specs navigation
2. **[LOAMSPINE_SPECIFICATION.md](specs/LOAMSPINE_SPECIFICATION.md)** — Core specification
3. **[ARCHITECTURE.md](specs/ARCHITECTURE.md)** — System architecture
4. **[DATA_MODEL.md](specs/DATA_MODEL.md)** — Data structures and models
5. **[API_SPECIFICATION.md](specs/API_SPECIFICATION.md)** — RPC API specification
6. **[PURE_RUST_RPC.md](specs/PURE_RUST_RPC.md)** — Pure Rust RPC philosophy
7. **[WAYPOINT_SEMANTICS.md](specs/WAYPOINT_SEMANTICS.md)** — Waypoint system
8. **[CERTIFICATE_LAYER.md](specs/CERTIFICATE_LAYER.md)** — Certificate management
9. **[SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md)** — Service lifecycle protocol
10. **[INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)** — Inter-primal integration
11. **[STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)** — Storage backend abstraction

---

## 🎯 SHOWCASES & EXAMPLES

### Showcase Directory (`showcase/`)
Practical examples and demonstrations:

#### Level 0: Local Primal Operations
**[showcase/01-local-primal/](showcase/01-local-primal/README.md)**
- Basic operations without dependencies
- Entry types, certificates, proofs
- Storage backends, backup/restore
- 7 complete demos

#### Level 1: RPC API
**[showcase/02-rpc-api/](showcase/02-rpc-api/README.md)**
- tarpc and JSON-RPC basics
- Health monitoring
- Concurrent operations
- Error handling
- 5 complete demos

#### Level 2: Service Discovery
**[showcase/03-songbird-discovery/](showcase/03-songbird-discovery/README.md)**
- Songbird integration
- Capability discovery
- Auto-advertisement
- Heartbeat monitoring
- 4 complete demos

#### Level 3: Inter-Primal Integration
**[showcase/04-inter-primal/](showcase/04-inter-primal/README.md)**
- Session and braid commits
- Signing and storage capabilities
- Full ecosystem integration
- 5 complete demos

#### Showcase Documentation
- **[00_SHOWCASE_INDEX.md](showcase/00_SHOWCASE_INDEX.md)** — Complete showcase navigation
- **[SHOWCASE_PRINCIPLES.md](showcase/SHOWCASE_PRINCIPLES.md)** — Testing philosophy
- **[README.md](showcase/README.md)** — Showcase overview

---

## 📜 HISTORY & ARCHIVES

### Version History
- **[CHANGELOG.md](CHANGELOG.md)** — Complete version history
- **[RELEASE_NOTES_v0.6.0.md](RELEASE_NOTES_v0.6.0.md)** — v0.6.0 release notes

### Historical Archives (`docs/archive/`)
Complete session reports and evolution history:

#### December 24, 2025: Foundation Evolution
**[docs/archive/dec-24-2025-evolution/](docs/archive/)**
- Initial comprehensive audit
- Code quality improvements
- Showcase development
- Service lifecycle implementation

#### December 25, 2025: Infant Discovery
**[docs/archive/dec-25-2025-infant-discovery/](docs/archive/dec-25-2025-infant-discovery/)**
- Hardcoding elimination (18 detailed reports)
- Infant discovery implementation
- Capability-based architecture evolution
- Zero-knowledge startup achievement

Key archived documents:
- **PHASE_1_2_COMPLETE_DEC_25_2025.md** — Full completion report
- **HARDCODING_ELIMINATION_PLAN.md** — Hardcoding cleanup strategy
- **COMPREHENSIVE_AUDIT_DEC_25_2025.md** — Complete audit report
- **SESSION_COMPLETE_DEC_25_2025.md** — Session summary

---

## 🔍 FINDING INFORMATION

### By Topic

#### Architecture & Design
- Core architecture → [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)
- Data model → [specs/DATA_MODEL.md](specs/DATA_MODEL.md)
- Pure Rust RPC → [specs/PURE_RUST_RPC.md](specs/PURE_RUST_RPC.md)
- Service lifecycle → [specs/SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md)

#### Implementation
- API specification → [specs/API_SPECIFICATION.md](specs/API_SPECIFICATION.md)
- Integration patterns → [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)
- Storage backends → [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- Examples & demos → [showcase/](showcase/)

#### Quality & Status
- Current status → [STATUS.md](STATUS.md)
- Audit results → [AUDIT_SUMMARY.md](AUDIT_SUMMARY.md)
- Integration gaps → [INTEGRATION_GAPS.md](INTEGRATION_GAPS.md)
- Known issues → [docs/planning/KNOWN_ISSUES.md](docs/planning/KNOWN_ISSUES.md)

#### Future & Planning
- Next version → [ROADMAP_V0.8.0.md](ROADMAP_V0.8.0.md)
- Long-term plans → [WHATS_NEXT.md](WHATS_NEXT.md)
- Refactoring ideas → [docs/planning/REFACTORING_RECOMMENDATIONS.md](docs/planning/REFACTORING_RECOMMENDATIONS.md)
- Zero-copy migration → [docs/planning/ZERO_COPY_MIGRATION_PLAN.md](docs/planning/ZERO_COPY_MIGRATION_PLAN.md)

---

## 📂 DIRECTORY STRUCTURE

```
loamSpine/
├── README.md                  # Project overview
├── START_HERE.md              # Quick start
├── STATUS.md                  # Current status
├── WHATS_NEXT.md              # Future plans
├── ROADMAP_V0.8.0.md          # v0.8.0 roadmap
├── AUDIT_SUMMARY.md           # Quality metrics
├── INTEGRATION_GAPS.md        # Integration status
├── CHANGELOG.md               # Version history
├── CONTRIBUTING.md            # Contribution guide
├── DOCS_INDEX.md              # This file!
│
├── specs/                     # Technical specifications
│   ├── 00_SPECIFICATIONS_INDEX.md
│   ├── LOAMSPINE_SPECIFICATION.md
│   ├── ARCHITECTURE.md
│   └── ... (8 more)
│
├── showcase/                  # Examples & demos
│   ├── 00_SHOWCASE_INDEX.md
│   ├── 01-local-primal/      # Level 0 demos
│   ├── 02-rpc-api/           # Level 1 demos
│   ├── 03-songbird-discovery/ # Level 2 demos
│   └── 04-inter-primal/      # Level 3 demos
│
├── docs/
│   ├── planning/             # Technical planning docs
│   │   ├── EXECUTIVE_SUMMARY.md
│   │   ├── KNOWN_ISSUES.md
│   │   └── ... (4 more)
│   │
│   └── archive/              # Historical reports
│       ├── dec-24-2025-evolution/
│       └── dec-25-2025-infant-discovery/
│
├── crates/                   # Source code
│   ├── loam-spine-core/
│   └── loam-spine-api/
│
└── ... (other files)
```

---

## 🎓 LEARNING PATHS

### Path 1: New Developer
1. [START_HERE.md](START_HERE.md)
2. [README.md](README.md)
3. [showcase/01-local-primal/](showcase/01-local-primal/)
4. [specs/ARCHITECTURE.md](specs/ARCHITECTURE.md)
5. [CONTRIBUTING.md](CONTRIBUTING.md)

### Path 2: Integration Developer
1. [specs/INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md)
2. [specs/API_SPECIFICATION.md](specs/API_SPECIFICATION.md)
3. [showcase/03-songbird-discovery/](showcase/03-songbird-discovery/)
4. [showcase/04-inter-primal/](showcase/04-inter-primal/)
5. [INTEGRATION_GAPS.md](INTEGRATION_GAPS.md)

### Path 3: Project Manager / Stakeholder
1. [STATUS.md](STATUS.md)
2. [AUDIT_SUMMARY.md](AUDIT_SUMMARY.md)
3. [docs/planning/EXECUTIVE_SUMMARY.md](docs/planning/EXECUTIVE_SUMMARY.md)
4. [ROADMAP_V0.8.0.md](ROADMAP_V0.8.0.md)
5. [WHATS_NEXT.md](WHATS_NEXT.md)

### Path 4: Quality Assurance
1. [AUDIT_SUMMARY.md](AUDIT_SUMMARY.md)
2. [INTEGRATION_GAPS.md](INTEGRATION_GAPS.md)
3. [docs/planning/KNOWN_ISSUES.md](docs/planning/KNOWN_ISSUES.md)
4. [showcase/](showcase/)
5. [specs/](specs/)

---

## 🔗 EXTERNAL LINKS

### Related Projects
- **Phase 1 Primals**: `../../phase1/`
  - BearDog (signing service)
  - Songbird (discovery/orchestration)
  - NestGate (data gateway)
  - Squirrel (AI service)
  - ToadStool (compute/encryption)

### Dependencies
- Rust: https://www.rust-lang.org/
- tarpc: https://github.com/google/tarpc
- tokio: https://tokio.rs/

---

## 📝 DOCUMENT CONVENTIONS

### Naming
- `UPPERCASE.md` — Root-level important documents
- `CamelCase.md` — Subdirectory documents
- `kebab-case/` — Directory names
- `DEC_25_2025` suffix — Dated documents (archived)

### Status Indicators
- ✅ Complete/Working
- 🟡 In Progress
- ❌ Not Started/Broken
- 📋 Planned
- 🔵 Future Enhancement

---

## 🆘 NEED HELP?

1. **Quick Start**: [START_HERE.md](START_HERE.md)
2. **Current Status**: [STATUS.md](STATUS.md)
3. **Known Issues**: [docs/planning/KNOWN_ISSUES.md](docs/planning/KNOWN_ISSUES.md)
4. **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
5. **Examples**: [showcase/](showcase/)

---

**Last Updated**: December 25, 2025  
**Maintained By**: LoamSpine Development Team  
**Version**: 0.7.0-dev (Infant Discovery Complete)

🦴 **LoamSpine: Permanent ledger for ecoPrimals ecosystem**
