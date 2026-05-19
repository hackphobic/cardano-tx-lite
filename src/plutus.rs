//! `PlutusData` — JSON shape matches cardano-serialization-lib's `PlutusDatumSchema::DetailedSchema`.
//!
//! This is exactly what `whisky::WData::JSON(s).to_cbor()` parses (it calls
//! `csl::PlutusData::from_json(s, DetailedSchema)` under the hood).
//!
//! ## Serialized forms
//!
//! ```text
//! Constr   →  { "constructor": <u64>, "fields": [ <PlutusData>... ] }
//! Map      →  { "map": [ { "k": <PlutusData>, "v": <PlutusData> }, ... ] }
//! List     →  { "list": [ <PlutusData>... ] }
//! Int      →  { "int": <number-or-decimal-string> }
//! Bytes    →  { "bytes": "<hex>" }
//! ```

use serde::{Deserialize, Serialize};

/// A Plutus integer. Serializes as a plain JSON number when it fits in `i64`,
/// otherwise as a decimal string. CSL's parser accepts both.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlutusInt {
    /// Small enough for an `i64` — emitted as a JSON number.
    Number(i64),
    /// Decimal-string fallback for values outside `i64`.
    String(String),
}

impl From<i64> for PlutusInt {
    fn from(v: i64) -> Self { Self::Number(v) }
}
impl From<i32> for PlutusInt {
    fn from(v: i32) -> Self { Self::Number(v as i64) }
}
impl From<u32> for PlutusInt {
    fn from(v: u32) -> Self { Self::Number(v as i64) }
}
impl From<String> for PlutusInt {
    fn from(v: String) -> Self { Self::String(v) }
}
impl From<&str> for PlutusInt {
    fn from(v: &str) -> Self { Self::String(v.to_string()) }
}

/// Plutus data — the on-chain datum / redeemer language.
///
/// Variants are `#[serde(untagged)]`; the discriminant is whichever required field is present
/// (`constructor`/`fields`, `map`, `list`, `int`, or `bytes`). This mirrors CSL DetailedSchema
/// exactly.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlutusData {
    /// Algebraic data: a tagged constructor with positional fields.
    Constr {
        constructor: u64,
        fields: Vec<PlutusData>,
    },
    /// Key–value map (order-preserving in CBOR; emitted as an array of `{k,v}` objects).
    Map {
        map: Vec<PlutusDataMapEntry>,
    },
    /// Ordered list.
    List {
        list: Vec<PlutusData>,
    },
    /// Big integer.
    Int {
        int: PlutusInt,
    },
    /// Byte string (hex-encoded).
    Bytes {
        /// Hex-encoded bytes (no `0x` prefix).
        bytes: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlutusDataMapEntry {
    pub k: PlutusData,
    pub v: PlutusData,
}

impl PlutusData {
    pub fn constr(idx: u64, fields: Vec<PlutusData>) -> Self {
        Self::Constr { constructor: idx, fields }
    }

    /// Unit `()` — `Constr 0 []`.
    pub fn unit() -> Self {
        Self::Constr { constructor: 0, fields: vec![] }
    }

    pub fn int(n: impl Into<PlutusInt>) -> Self {
        Self::Int { int: n.into() }
    }

    pub fn bytes(hex: impl Into<String>) -> Self {
        Self::Bytes { bytes: hex.into() }
    }

    pub fn list(items: Vec<PlutusData>) -> Self {
        Self::List { list: items }
    }

    pub fn map(entries: impl IntoIterator<Item = (PlutusData, PlutusData)>) -> Self {
        Self::Map {
            map: entries.into_iter().map(|(k, v)| PlutusDataMapEntry { k, v }).collect(),
        }
    }
}

/// A redeemer: `PlutusData` argument plus an execution-units budget.
///
/// The redeemer **tag** (spend / mint / cert / reward / vote / propose) and **index** are
/// implicit from where the redeemer is attached (on a `TxInput`, `MintPolicy`, `CertEntry`,
/// `Withdrawal`, …) and therefore not stored on the redeemer itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Redeemer {
    pub data: PlutusData,
    pub ex_units: ExUnits,
}

impl Redeemer {
    pub fn new(data: PlutusData, ex_units: ExUnits) -> Self {
        Self { data, ex_units }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExUnits {
    pub mem: u64,
    pub steps: u64,
}
