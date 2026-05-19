//! Conway-era certificates: stake registration/delegation, pool ops, DRep ops, committee ops.

use crate::address::{Credential, RewardAddress};
use crate::primitives::{
    DRepKeyHash, Epoch, Hash32, KeyHash, Lovelace, PoolId, ScriptHash, VrfKeyHash,
};
use serde::{Deserialize, Serialize};

/// A "delegate to a DRep" target.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DRep {
    KeyHash { hash: DRepKeyHash },
    ScriptHash { hash: ScriptHash },
    /// Abstain from every vote.
    AlwaysAbstain,
    /// Always vote "no confidence".
    AlwaysNoConfidence,
}

/// A URL + content-hash anchor (off-chain metadata pointer).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    pub url: String,
    pub data_hash: Hash32,
}

/// Stake-pool relay.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Relay {
    /// IPv4 and/or IPv6 host.
    #[serde(rename_all = "camelCase")]
    SingleHostAddr {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        port: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ipv4: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ipv6: Option<String>,
    },
    /// DNS A/AAAA name (single host).
    #[serde(rename_all = "camelCase")]
    SingleHostName {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        port: Option<u16>,
        dns_name: String,
    },
    /// DNS SRV name (multi-host).
    #[serde(rename_all = "camelCase")]
    MultiHostName {
        dns_name: String,
    },
}

/// Off-chain pool metadata pointer.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMetadata {
    pub url: String,
    pub hash: Hash32,
}

/// A rational number in `[0, 1]`. Used for pool margin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitInterval {
    pub numerator: u64,
    pub denominator: u64,
}

/// Stake-pool parameters (used by `Cert::PoolRegistration`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolParams {
    pub id: PoolId,
    pub vrf_key_hash: VrfKeyHash,
    pub pledge: Lovelace,
    pub cost: Lovelace,
    pub margin: UnitInterval,
    pub reward_account: RewardAddress,
    pub owners: Vec<KeyHash>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relays: Vec<Relay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PoolMetadata>,
}

/// A Conway-era certificate.
///
/// Pre-Conway certs (`stakeRegistration`, `stakeDeregistration`, `stakeDelegation`,
/// `poolRegistration`, `poolRetirement`) are still valid. Conway adds explicit-deposit
/// stake registration, vote delegation, combined certs, DRep ops, and committee ops.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Cert {
    // ── Legacy (still accepted in Conway) ─────────────────────────────────────
    /// Implicit-deposit stake registration.
    StakeRegistration { credential: Credential },
    /// Implicit-refund stake deregistration.
    StakeDeregistration { credential: Credential },
    /// Delegate the stake credential to a pool.
    StakeDelegation { credential: Credential, pool: PoolId },

    // ── Pool ──────────────────────────────────────────────────────────────────
    PoolRegistration { pool_params: PoolParams },
    PoolRetirement { pool: PoolId, epoch: Epoch },

    // ── Conway: explicit-deposit stake reg/unreg ──────────────────────────────
    /// Explicit-deposit stake registration (Conway).
    #[serde(rename_all = "camelCase")]
    Reg { credential: Credential, deposit: Lovelace },
    /// Explicit-refund stake deregistration (Conway).
    #[serde(rename_all = "camelCase")]
    Unreg { credential: Credential, deposit: Lovelace },

    // ── Conway: delegation variants ───────────────────────────────────────────
    /// Delegate vote-only to a DRep.
    #[serde(rename_all = "camelCase")]
    VoteDeleg { credential: Credential, drep: DRep },
    /// Delegate stake to a pool AND vote to a DRep.
    #[serde(rename_all = "camelCase")]
    StakeVoteDeleg { credential: Credential, pool: PoolId, drep: DRep },
    /// Register stake + delegate to a pool in one cert.
    #[serde(rename_all = "camelCase")]
    StakeRegDeleg { credential: Credential, pool: PoolId, deposit: Lovelace },
    /// Register stake + delegate vote to a DRep in one cert.
    #[serde(rename_all = "camelCase")]
    VoteRegDeleg { credential: Credential, drep: DRep, deposit: Lovelace },
    /// Register stake + delegate to pool + delegate vote to DRep, all-in-one.
    #[serde(rename_all = "camelCase")]
    StakeVoteRegDeleg {
        credential: Credential,
        pool: PoolId,
        drep: DRep,
        deposit: Lovelace,
    },

    // ── Conway: constitutional committee ──────────────────────────────────────
    /// Authorize a hot-key credential for an existing cold committee member.
    #[serde(rename_all = "camelCase")]
    AuthCommitteeHot {
        cold_credential: Credential,
        hot_credential: Credential,
    },
    /// Resign a cold committee credential.
    #[serde(rename_all = "camelCase")]
    ResignCommitteeCold {
        cold_credential: Credential,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
    },

    // ── Conway: DRep ops ──────────────────────────────────────────────────────
    /// Register a DRep.
    #[serde(rename_all = "camelCase")]
    RegDRep {
        credential: Credential,
        deposit: Lovelace,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
    },
    /// Unregister a DRep (refunds the deposit).
    #[serde(rename_all = "camelCase")]
    UnregDRep { credential: Credential, deposit: Lovelace },
    /// Update a DRep's metadata anchor.
    #[serde(rename_all = "camelCase")]
    UpdateDRep {
        credential: Credential,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
    },
}
