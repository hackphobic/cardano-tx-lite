//! Transaction inputs, outputs, mint, withdrawals, certs, and the full body.
//!
//! ## Grouping principle
//!
//! Everything a script needs to be executed lives next to the thing it executes against:
//!
//! | Action            | Wrapper          | Carries                                |
//! | ----------------- | ---------------- | -------------------------------------- |
//! | Spending a UTxO   | [`TxInput`]      | `datum`, `redeemer`, `script_ref`      |
//! | Minting / burning | [`MintPolicy`]   | `assets`, `script`, `redeemer`         |
//! | Withdrawing       | [`Withdrawal`]   | `amount`, `script`, `redeemer`         |
//! | Certificates      | [`CertEntry`]    | `cert`, `script`, `redeemer`           |
//!
//! Redeemer **tag** and **index** are implicit from attachment site, so [`Redeemer`] only
//! carries `data` + `exUnits`.

use crate::address::{Address, RewardAddress};
use crate::cert::Cert;
use crate::plutus::{PlutusData, Redeemer};
use crate::primitives::{
    AssetName, DataHash, Hash32, KeyHash, Lovelace, MintQuantity, PolicyId, Slot, TxHash,
};
use crate::script::Script;
use crate::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ─── Inputs ──────────────────────────────────────────────────────────────────────────────

/// A transaction input: a fully-resolved UTxO reference plus the spending context
/// (if it's script-locked).
///
/// Carries `tx_hash`, `index`, **plus** the resolved `address` and `value` of the UTxO
/// being spent — the same shape whisky's `tx_in(tx_hash, index, &[Asset], address)` takes.
/// This lets a tx-builder backend rebuild the tx without a sidecar UTxO-resolution map.
///
/// For a plain key-locked input the optional fields are all `None`. For a script-locked
/// input the builder attaches:
///
/// - `datum` — the datum stored at the UTxO (inline or by hash). Required for spending.
/// - `redeemer` — the redeemer for executing the spending script.
/// - `script_ref` — the validator script. Either the literal script (it will be added to
///    the witness set) or a reference to it via a reference input elsewhere.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInput {
    pub tx_hash: TxHash,
    pub index: u32,
    /// Resolved address of the UTxO being spent.
    pub address: Address,
    /// Resolved value (coin + multi-asset) at the UTxO.
    pub value: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datum: Option<DatumOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeemer: Option<Redeemer>,
    /// Reference script attached to the output (Babbage+).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_ref: Option<Script>,
}

impl TxInput {
    pub fn new(
        tx_hash: impl Into<TxHash>,
        index: u32,
        address: impl Into<Address>,
        value: Value,
    ) -> Self {
        Self {
            tx_hash: tx_hash.into(),
            index,
            address: address.into(),
            value,
            datum: None,
            redeemer: None,
            script_ref: None,
        }
    }
    pub fn with_datum(mut self, d: DatumOption) -> Self { self.datum = Some(d); self }
    pub fn with_inline_datum(mut self, d: PlutusData) -> Self {
        self.datum = Some(DatumOption::Inline { data: d });
        self
    }
    pub fn with_datum_hash(mut self, h: impl Into<DataHash>) -> Self {
        self.datum = Some(DatumOption::Hash { hash: h.into() });
        self
    }
    pub fn with_redeemer(mut self, r: Redeemer) -> Self { self.redeemer = Some(r); self }
    pub fn with_script_ref(mut self, s: Script) -> Self { self.script_ref = Some(s); self }
}

// ─── Outputs ─────────────────────────────────────────────────────────────────────────────

/// Inline datum or datum-hash attached to an output we're creating.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DatumOption {
    Hash { hash: DataHash },
    Inline { data: PlutusData },
}

/// A transaction output (a new UTxO this tx is creating).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxOutput {
    pub address: Address,
    pub value: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datum: Option<DatumOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_ref: Option<Script>,
}

impl TxOutput {
    pub fn new(address: impl Into<Address>, value: Value) -> Self {
        Self { address: address.into(), value, datum: None, script_ref: None }
    }
    pub fn with_inline_datum(mut self, d: PlutusData) -> Self {
        self.datum = Some(DatumOption::Inline { data: d });
        self
    }
    pub fn with_datum_hash(mut self, h: impl Into<DataHash>) -> Self {
        self.datum = Some(DatumOption::Hash { hash: h.into() });
        self
    }
    pub fn with_script_ref(mut self, s: Script) -> Self {
        self.script_ref = Some(s);
        self
    }
}

// ─── Mint ────────────────────────────────────────────────────────────────────────────────

/// Per-policy mint entry: the assets being minted/burned under this policy, plus the policy
/// script and (if Plutus) its redeemer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintPolicy {
    /// Asset name → signed quantity. Positive = mint, negative = burn.
    pub assets: BTreeMap<AssetName, MintQuantity>,
    /// The minting policy script. Optional in the wire type (a server might fill it in from
    /// a separate registry), but required for the tx to be valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<Script>,
    /// Redeemer for executing a Plutus minting policy. `None` for native-script policies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeemer: Option<Redeemer>,
}

impl MintPolicy {
    pub fn new() -> Self { Self::default() }

    pub fn mint(mut self, asset: impl Into<AssetName>, qty: i128) -> Self {
        self.assets.insert(asset.into(), MintQuantity(qty));
        self
    }
    pub fn with_script(mut self, s: Script) -> Self { self.script = Some(s); self }
    pub fn with_redeemer(mut self, r: Redeemer) -> Self { self.redeemer = Some(r); self }
}

/// Full mint field: `policyId → MintPolicy`.
pub type Mint = BTreeMap<PolicyId, MintPolicy>;

// ─── Withdrawals ─────────────────────────────────────────────────────────────────────────

/// A reward withdrawal: the amount being claimed plus (for script-controlled stake creds)
/// the script that must execute and its redeemer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    pub amount: Lovelace,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<Script>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeemer: Option<Redeemer>,
}

impl Withdrawal {
    pub fn new(amount: Lovelace) -> Self {
        Self { amount, script: None, redeemer: None }
    }
    pub fn with_script(mut self, s: Script) -> Self { self.script = Some(s); self }
    pub fn with_redeemer(mut self, r: Redeemer) -> Self { self.redeemer = Some(r); self }
}

/// Full withdrawals field: `rewardAddress → Withdrawal`.
pub type Withdrawals = BTreeMap<RewardAddress, Withdrawal>;

// ─── Cert entries ────────────────────────────────────────────────────────────────────────

/// A certificate plus the script + redeemer needed to authorize it, if the credential it
/// touches is script-controlled.
///
/// Certs that operate on a `Credential::ScriptHash { … }` need a witnessing script and
/// (when that script is Plutus) a redeemer. Key-credential certs leave both `None`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertEntry {
    pub cert: Cert,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<Script>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeemer: Option<Redeemer>,
}

impl CertEntry {
    pub fn new(cert: Cert) -> Self {
        Self { cert, script: None, redeemer: None }
    }
    pub fn with_script(mut self, s: Script) -> Self { self.script = Some(s); self }
    pub fn with_redeemer(mut self, r: Redeemer) -> Self { self.redeemer = Some(r); self }
}

impl From<Cert> for CertEntry {
    fn from(cert: Cert) -> Self { Self::new(cert) }
}

// ─── Body ────────────────────────────────────────────────────────────────────────────────

/// A simplified Conway-era transaction body.
///
/// All script-execution context (datum/redeemer/script) is grouped with the action it
/// authorizes (input/mint/withdrawal/cert), not held in a separate witness sidecar.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxBody {
    pub inputs: Vec<TxInput>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_inputs: Vec<TxInput>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collateral_inputs: Vec<TxInput>,

    pub outputs: Vec<TxOutput>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collateral_return: Option<TxOutput>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_collateral: Option<Lovelace>,

    pub fee: Lovelace,

    /// Invalid-hereafter slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl: Option<Slot>,

    /// Invalid-before slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validity_start: Option<Slot>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub certs: Vec<CertEntry>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub withdrawals: Withdrawals,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mint: Mint,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_signers: Vec<KeyHash>,

    /// Network id discriminator (0 = testnet, 1 = mainnet). Optional in the body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u8>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary_data_hash: Option<Hash32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_data_hash: Option<Hash32>,
}

impl TxBody {
    pub fn new() -> Self { Self::default() }
}
