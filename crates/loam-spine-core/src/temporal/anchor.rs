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
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
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
    pub fn anchor_type(&self) -> AnchorType {
        match self {
            Anchor::Crypto(_) => AnchorType::Crypto,
            Anchor::Atomic(_) => AnchorType::Atomic,
            Anchor::Causal(_) => AnchorType::Causal,
            Anchor::Consensus(_) => AnchorType::Consensus,
        }
    }
}

