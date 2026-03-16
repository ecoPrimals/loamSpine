// SPDX-License-Identifier: AGPL-3.0-or-later

//! Temporal anchors - what defines order in the linear past.
//!
//! The same DAG can collapse to different linear orderings depending on
//! what matters for the domain.

use serde::{Deserialize, Serialize};

/// Defines what establishes order in the linear timeline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Anchor {
    /// Anchored to blockchain consensus (ETH, BTC, etc.)
    Crypto(CryptoAnchor),

    /// Anchored to atomic time (NIST, GPS, etc.)
    Atomic(AtomicAnchor),

    /// Anchored to causal event order (not clock time!)
    Causal(CausalAnchor),

    /// Anchored to group consensus (social time)
    Consensus(ConsensusAnchor),
}

/// Anchor to blockchain consensus.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CryptoAnchor {
    /// Which chain (ethereum, bitcoin, etc.)
    pub chain: String,

    /// Block height
    pub block_height: u64,

    /// Block hash
    pub block_hash: String,

    /// Transaction hash (optional)
    pub tx_hash: Option<String>,
}

/// Anchor to atomic time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtomicAnchor {
    /// UTC timestamp
    pub timestamp: std::time::SystemTime,

    /// Precision (nanosecond, microsecond, etc.)
    pub precision: TimePrecision,

    /// Source (NIST, GPS, local, etc.)
    pub source: String,
}

/// Time precision level.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TimePrecision {
    /// Nanosecond precision (10^-9 seconds)
    Nanosecond,
    /// Microsecond precision (10^-6 seconds)
    Microsecond,
    /// Millisecond precision (10^-3 seconds)
    Millisecond,
    /// Second precision
    Second,
    /// Minute precision
    Minute,
}

/// Anchor to causal event order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalAnchor {
    /// Sequence number in causal chain
    pub sequence: u64,

    /// Causal parents (events that happened before)
    pub causal_parents: Vec<String>,

    /// Lamport clock (optional)
    pub lamport_clock: Option<u64>,
}

/// Anchor to social consensus.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusAnchor {
    /// Agents who agreed on this ordering
    pub agreed_by: Vec<String>,

    /// When consensus was reached
    pub consensus_timestamp: std::time::SystemTime,

    /// Consensus mechanism used
    pub mechanism: String,
}

/// Type of anchor (for filtering/queries).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnchorType {
    /// Blockchain/cryptocurrency anchor
    Crypto,
    /// Atomic time anchor (physics-based)
    Atomic,
    /// Causal event order anchor
    Causal,
    /// Social consensus anchor
    Consensus,
}

impl Anchor {
    /// Get the type of this anchor.
    #[must_use]
    pub const fn anchor_type(&self) -> AnchorType {
        match self {
            Self::Crypto(_) => AnchorType::Crypto,
            Self::Atomic(_) => AnchorType::Atomic,
            Self::Causal(_) => AnchorType::Causal,
            Self::Consensus(_) => AnchorType::Consensus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn crypto_anchor_creation() {
        let anchor = CryptoAnchor {
            chain: "ethereum".to_string(),
            block_height: 18_000_000,
            block_hash: "0xabc123".to_string(),
            tx_hash: Some("0xdef456".to_string()),
        };

        assert_eq!(anchor.chain, "ethereum");
        assert_eq!(anchor.block_height, 18_000_000);
        assert!(anchor.tx_hash.is_some());
    }

    #[test]
    fn atomic_anchor_creation() {
        let anchor = AtomicAnchor {
            timestamp: SystemTime::now(),
            precision: TimePrecision::Nanosecond,
            source: "NIST".to_string(),
        };

        assert_eq!(anchor.source, "NIST");
        assert!(matches!(anchor.precision, TimePrecision::Nanosecond));
    }

    #[test]
    fn causal_anchor_creation() {
        let anchor = CausalAnchor {
            sequence: 42,
            causal_parents: vec!["event1".to_string(), "event2".to_string()],
            lamport_clock: Some(100),
        };

        assert_eq!(anchor.sequence, 42);
        assert_eq!(anchor.causal_parents.len(), 2);
        assert_eq!(anchor.lamport_clock, Some(100));
    }

    #[test]
    fn consensus_anchor_creation() {
        let anchor = ConsensusAnchor {
            agreed_by: vec!["alice".to_string(), "bob".to_string()],
            consensus_timestamp: SystemTime::now(),
            mechanism: "raft".to_string(),
        };

        assert_eq!(anchor.agreed_by.len(), 2);
        assert_eq!(anchor.mechanism, "raft");
    }

    #[test]
    fn anchor_type_detection() {
        let crypto = Anchor::Crypto(CryptoAnchor {
            chain: "bitcoin".to_string(),
            block_height: 800_000,
            block_hash: "0x123".to_string(),
            tx_hash: None,
        });
        assert_eq!(crypto.anchor_type(), AnchorType::Crypto);

        let atomic = Anchor::Atomic(AtomicAnchor {
            timestamp: SystemTime::now(),
            precision: TimePrecision::Microsecond,
            source: "GPS".to_string(),
        });
        assert_eq!(atomic.anchor_type(), AnchorType::Atomic);

        let causal = Anchor::Causal(CausalAnchor {
            sequence: 1,
            causal_parents: vec![],
            lamport_clock: None,
        });
        assert_eq!(causal.anchor_type(), AnchorType::Causal);

        let consensus = Anchor::Consensus(ConsensusAnchor {
            agreed_by: vec!["node1".to_string()],
            consensus_timestamp: SystemTime::now(),
            mechanism: "paxos".to_string(),
        });
        assert_eq!(consensus.anchor_type(), AnchorType::Consensus);
    }

    #[test]
    fn anchor_type_equality() {
        assert_eq!(AnchorType::Crypto, AnchorType::Crypto);
        assert_eq!(AnchorType::Atomic, AnchorType::Atomic);
        assert_ne!(AnchorType::Crypto, AnchorType::Atomic);
    }

    #[test]
    fn time_precision_variants() {
        let precisions = [
            TimePrecision::Nanosecond,
            TimePrecision::Microsecond,
            TimePrecision::Millisecond,
            TimePrecision::Second,
            TimePrecision::Minute,
        ];

        for precision in &precisions {
            let debug_str = format!("{precision:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn anchor_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let anchor = Anchor::Atomic(AtomicAnchor {
            timestamp: SystemTime::UNIX_EPOCH,
            precision: TimePrecision::Second,
            source: "test".to_string(),
        });

        let json = serde_json::to_string(&anchor)?;
        let deserialized: Anchor = serde_json::from_str(&json)?;

        assert_eq!(anchor.anchor_type(), deserialized.anchor_type());
        Ok(())
    }

    #[test]
    fn crypto_anchor_clone() {
        let anchor = CryptoAnchor {
            chain: "ethereum".to_string(),
            block_height: 1_000_000,
            block_hash: "0xabc".to_string(),
            tx_hash: None,
        };

        let cloned = anchor.clone();
        assert_eq!(anchor.chain, cloned.chain);
        assert_eq!(anchor.block_height, cloned.block_height);
    }

    #[test]
    fn causal_anchor_empty_parents() {
        let anchor = CausalAnchor {
            sequence: 0,
            causal_parents: vec![],
            lamport_clock: None,
        };

        assert!(anchor.causal_parents.is_empty());
        assert!(anchor.lamport_clock.is_none());
    }

    #[test]
    fn crypto_anchor_without_tx() {
        let anchor = CryptoAnchor {
            chain: "bitcoin".to_string(),
            block_height: 500_000,
            block_hash: "0x000".to_string(),
            tx_hash: None,
        };

        assert!(anchor.tx_hash.is_none());
    }

    #[test]
    fn anchor_debug_impl() {
        let anchor = Anchor::Atomic(AtomicAnchor {
            timestamp: SystemTime::UNIX_EPOCH,
            precision: TimePrecision::Millisecond,
            source: "local".to_string(),
        });

        let debug_str = format!("{anchor:?}");
        assert!(debug_str.contains("Atomic"));
    }
}
