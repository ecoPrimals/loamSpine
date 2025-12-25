# 🦴 Hello LoamSpine - Your First Spine

**Time**: 5 minutes  
**Difficulty**: Beginner  
**Prerequisites**: None

---

## 🎯 What You'll Learn

- How to create a spine with an owner DID
- How to add entries to the spine
- How LoamSpine ensures data integrity
- The append-only nature of spines

---

## 🚀 Quick Start

```bash
./demo.sh
```

---

## 📖 Concepts

### Spine

A **spine** is a sovereign, append-only ledger owned by a single entity (person, organization, or primal). Think of it as:

- A personal blockchain
- A permanent record of events
- A source of truth for provenance

### Owner DID

Every spine has an **owner DID** (Decentralized Identifier):

```
did:example:alice123
```

This identifies who owns and controls the spine.

### Entries

**Entries** are the records stored in a spine:

- Each entry has a type (Text, Metadata, Certificate, etc.)
- Each entry has a payload (the actual data)
- Entries are content-addressed using BLAKE3
- Entries form a Merkle chain for integrity

---

## 🔍 What Happens

1. **Create Spine**: A new spine is created with an owner DID
2. **Add Entries**: Two entries are added (Text and Metadata)
3. **Verify Integrity**: The spine's hash chain is verified
4. **View Metadata**: Spine statistics are displayed

---

## 💡 Key Takeaways

- ✅ Spines are append-only (no edits or deletes)
- ✅ Each spine has a unique owner
- ✅ Entries are cryptographically linked
- ✅ Integrity is automatically verified

---

## 🔗 Next Steps

- **02-entry-types**: Explore all 15+ entry types
- **03-certificate-lifecycle**: Mint and transfer certificates
- **04-proofs**: Generate inclusion and provenance proofs

---

**Ready to explore?** Run `./demo.sh` to get started!

