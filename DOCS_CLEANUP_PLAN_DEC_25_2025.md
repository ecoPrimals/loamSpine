# 🧹 Root Documentation Cleanup Plan — December 25, 2025

**Current State**: 35 markdown files at root  
**Target**: ~10 essential files + archived historical docs  
**Status**: In Progress

---

## 📊 CATEGORIZATION

### ✅ KEEP (Essential Documentation - 10 files)
These are the core documents users need:

1. **README.md** — Project overview and quick start
2. **START_HERE.md** — New user onboarding
3. **STATUS.md** — Current project status
4. **WHATS_NEXT.md** — Roadmap and future plans
5. **CHANGELOG.md** — Version history
6. **CONTRIBUTING.md** — Contribution guidelines
7. **INTEGRATION_GAPS.md** — Current integration status
8. **AUDIT_SUMMARY.md** — Latest audit results
9. **ROADMAP_V0.8.0.md** — v0.8.0 planning (NEW)
10. **DOCS_INDEX.md** — Master documentation index

### 📦 ARCHIVE (Historical Session Reports - 16 files)
Move to `docs/archive/dec-25-2025-infant-discovery/`:

1. AUDIT_ACTION_ITEMS_DEC_25_2025.md
2. AUDIT_EXECUTIVE_SUMMARY_DEC_25_2025.md
3. AUDIT_SUMMARY_QUICK_REFERENCE.md
4. CLIPPY_FIXES_DEEP_SOLUTIONS.md
5. COMPLETE_AUDIT_AND_HARDCODING_FINAL.md
6. COMPLETE_SUCCESS_DEC_25_2025.md
7. COMPREHENSIVE_AUDIT_DEC_25_2025.md
8. DOCS_CLEANUP_DEC_25_2025.md
9. FINAL_AUDIT_WITH_HARDCODING_DEC_25_2025.md
10. FINAL_SESSION_REPORT_DEC_25_2025.md
11. HARDCODING_CLEANUP_PROGRESS_DEC_25_2025.md
12. HARDCODING_ELIMINATION_PLAN.md
13. HARDCODING_SESSION_COMPLETE_DEC_25_2025.md
14. IMPLEMENTATION_PROGRESS_DEC_25_2025.md
15. PHASE_1_2_COMPLETE_DEC_25_2025.md
16. SESSION_COMPLETE_DEC_25_2025.md
17. SESSION_FINAL_DEC_25_2025.md
18. SESSION_PROGRESS_DEC_25_2025.md

### 📝 MOVE TO DOCS/ (Technical Planning - 5 files)
Move to `docs/planning/`:

1. EXECUTIVE_SUMMARY.md
2. KNOWN_ISSUES.md
3. REFACTORING_RECOMMENDATIONS.md
4. SHOWCASE_EVOLUTION_PLAN.md
5. SHOWCASE_STATUS.md
6. ZERO_COPY_MIGRATION_PLAN.md

### 📚 KEEP IN SUBDIRECTORIES (Already organized - 4 files)
1. RELEASE_NOTES_v0.6.0.md → Keep at root (or move to `releases/`)
2. specs/*.md → Already organized
3. showcase/*.md → Already organized
4. docs/archive/*.md → Already organized

---

## 🎯 CLEANUP ACTIONS

### Phase 1: Create Archive Directory ✅
```bash
mkdir -p docs/archive/dec-25-2025-infant-discovery
```

### Phase 2: Archive Session Reports
Move all December 25 session/audit files to archive:
```bash
mv AUDIT_ACTION_ITEMS_DEC_25_2025.md docs/archive/dec-25-2025-infant-discovery/
mv AUDIT_EXECUTIVE_SUMMARY_DEC_25_2025.md docs/archive/dec-25-2025-infant-discovery/
# ... (all 18 files)
```

### Phase 3: Organize Planning Documents
```bash
mkdir -p docs/planning
mv EXECUTIVE_SUMMARY.md docs/planning/
mv KNOWN_ISSUES.md docs/planning/
mv REFACTORING_RECOMMENDATIONS.md docs/planning/
mv SHOWCASE_EVOLUTION_PLAN.md docs/planning/
mv SHOWCASE_STATUS.md docs/planning/
mv ZERO_COPY_MIGRATION_PLAN.md docs/planning/
```

### Phase 4: Update Master Index (DOCS_INDEX.md)
Create comprehensive navigation for all documentation.

### Phase 5: Update Core Documents
- **STATUS.md** — Update with v0.7.0-dev completion
- **WHATS_NEXT.md** — Point to ROADMAP_V0.8.0.md
- **README.md** — Ensure links are current
- **START_HERE.md** — Update quick start guide

---

## 📈 BEFORE & AFTER

### Before
```
Root: 35 .md files
├── 10 essential
├── 18 session reports (cluttered)
├── 6 planning docs (mixed in)
└── 1 release note
```

### After
```
Root: 10-11 .md files (clean!)
├── 10 essential docs
└── 1 release note (optional)

docs/
├── archive/
│   ├── dec-24-2025-evolution/ (existing)
│   └── dec-25-2025-infant-discovery/ (NEW - 18 files)
├── planning/ (NEW - 6 files)
└── ... (other docs)
```

**Reduction**: 35 → 10-11 files at root (71% cleanup!)

---

## ✅ UPDATED DOCUMENTS PLAN

### STATUS.md Updates
```markdown
**Version**: 0.7.0-dev
**Status**: Production Ready
**Last Updated**: December 25, 2025

## Recent Achievements
- ✅ Infant Discovery Complete
- ✅ 30% Hardcoding Reduction
- ✅ 372 Tests Passing (90.39% coverage)

## Current Focus
- Planning v0.8.0 (DNS SRV + mDNS)

## Next Milestone
- v0.8.0: Complete Discovery Stack (2-3 weeks)
```

### WHATS_NEXT.md Updates
```markdown
# What's Next for LoamSpine

## Immediate (v0.8.0 - Next 2-3 weeks)
See detailed roadmap: **ROADMAP_V0.8.0.md**

1. DNS SRV Discovery Implementation
2. mDNS Discovery Implementation
3. Full Discovery Chain Integration

## Short Term (v0.9.0 - 1-2 months)
- Enhanced capability registry
- Production metrics
- Performance optimization

## Long Term (v1.0.0+)
- Network federation
- Zero-copy RPC migration
- Advanced observability
```

### DOCS_INDEX.md Updates
```markdown
# 📚 LoamSpine Documentation Index

## 🚀 Getting Started
- **START_HERE.md** — Begin here!
- **README.md** — Project overview
- **CONTRIBUTING.md** — How to contribute

## 📊 Current Status
- **STATUS.md** — Project status
- **AUDIT_SUMMARY.md** — Latest quality metrics
- **INTEGRATION_GAPS.md** — Integration status

## 🗺️ Planning & Roadmap
- **ROADMAP_V0.8.0.md** — Next version plan
- **WHATS_NEXT.md** — Future direction
- **docs/planning/** — Technical planning docs

## 📖 Specifications
- **specs/** — Complete technical specifications
  - LOAMSPINE_SPECIFICATION.md
  - ARCHITECTURE.md
  - API_SPECIFICATION.md
  - ... (8 more)

## 📜 History & Archives
- **CHANGELOG.md** — Version history
- **docs/archive/** — Historical reports
  - dec-24-2025-evolution/
  - dec-25-2025-infant-discovery/

## 🎯 Showcases
- **showcase/** — Examples and demos
  - 01-local-primal/
  - 02-rpc-api/
  - 03-songbird-discovery/
  - 04-inter-primal/
```

---

## 🎯 SUCCESS CRITERIA

- [ ] Root directory has ≤11 markdown files
- [ ] All session reports archived
- [ ] Planning docs organized in docs/planning/
- [ ] DOCS_INDEX.md comprehensive and up-to-date
- [ ] STATUS.md reflects v0.7.0-dev completion
- [ ] WHATS_NEXT.md points to v0.8.0 roadmap
- [ ] All links verified and working
- [ ] README.md updated with latest info

---

**Execution Time**: ~30 minutes  
**Impact**: 71% reduction in root clutter  
**Benefit**: Much cleaner, more navigable documentation structure

