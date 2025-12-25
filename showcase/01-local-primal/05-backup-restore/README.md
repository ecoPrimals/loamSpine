# 💾 Backup & Restore - Data Portability

**Time**: 10 minutes  
**Difficulty**: Beginner  
**Prerequisites**: 01-hello-loamspine

---

## 🎯 What You'll Learn

- Export spines to binary format
- Export spines to JSON format
- Import and verify backups
- Backup verification workflow

---

## 📖 Backup Formats

### 1. Binary Export
Compact, efficient format for production backups.

### 2. JSON Export
Human-readable format for inspection and auditing.

---

## 🔍 Workflow

```
Create Spine
    ↓
Add Entries
    ↓
Export Backup (Binary or JSON)
    ↓
Store Safely
    ↓
Import Backup
    ↓
Verify Integrity
```

---

## 💡 Key Features

- ✅ Full spine backup (entries + certificates)
- ✅ Integrity verification
- ✅ Compression support
- ✅ Versioned format

---

**Status**: ⏳ Rust example needed  
**Related**: `crates/loam-spine-core/src/backup.rs`

