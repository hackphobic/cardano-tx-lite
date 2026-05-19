//! Bech32-encoded address types and on-chain credentials.

use crate::primitives::{KeyHash, ScriptHash};
use serde::{Deserialize, Serialize};

/// A Cardano payment address in bech32 form (`addr1…` mainnet, `addr_test1…` testnet).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Address(pub String);

impl Address {
    #[inline]
    pub fn new(b32: impl Into<String>) -> Self { Self(b32.into()) }
    #[inline]
    pub fn as_str(&self) -> &str { &self.0 }
}
impl From<String> for Address {
    fn from(s: String) -> Self { Self(s) }
}
impl From<&str> for Address {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A bech32 stake/reward address (`stake1…` / `stake_test1…`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RewardAddress(pub String);

impl RewardAddress {
    #[inline]
    pub fn new(b32: impl Into<String>) -> Self { Self(b32.into()) }
    #[inline]
    pub fn as_str(&self) -> &str { &self.0 }
}
impl From<String> for RewardAddress {
    fn from(s: String) -> Self { Self(s) }
}
impl From<&str> for RewardAddress {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}

/// On-chain credential — a key hash or a script hash. Used by certs, withdrawals, DReps, etc.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Credential {
    /// Authorized by signature of the corresponding key.
    KeyHash {
        #[serde(rename = "keyHash")]
        key_hash: KeyHash,
    },
    /// Authorized by execution of the corresponding script.
    ScriptHash {
        #[serde(rename = "scriptHash")]
        script_hash: ScriptHash,
    },
}

impl Credential {
    pub fn key(h: impl Into<KeyHash>) -> Self {
        Self::KeyHash { key_hash: h.into() }
    }
    pub fn script(h: impl Into<ScriptHash>) -> Self {
        Self::ScriptHash { script_hash: h.into() }
    }
}
