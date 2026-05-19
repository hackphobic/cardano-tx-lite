//! Native scripts (multisig/timelock) and Plutus scripts.

use crate::primitives::{KeyHash, Slot};
use serde::{Deserialize, Serialize};

/// A native script — recursive multisig/timelock tree. JSON uses cardano-cli/CSL keys:
/// `sig`, `all`, `any`, `atLeast`, `after`, `before`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NativeScript {
    /// Requires a signature by the holder of `keyHash`.
    Sig {
        #[serde(rename = "keyHash")]
        key_hash: KeyHash,
    },
    /// All sub-scripts must succeed.
    All { scripts: Vec<NativeScript> },
    /// At least one sub-script must succeed.
    Any { scripts: Vec<NativeScript> },
    /// At least `required` of the sub-scripts must succeed.
    #[serde(rename_all = "camelCase")]
    AtLeast {
        required: u32,
        scripts: Vec<NativeScript>,
    },
    /// Valid only at or after `slot` (invalid-before).
    After { slot: Slot },
    /// Valid only before `slot` (invalid-hereafter).
    Before { slot: Slot },
}

/// Plutus script language version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlutusLanguage {
    #[serde(rename = "plutusV1")]
    V1,
    #[serde(rename = "plutusV2")]
    V2,
    #[serde(rename = "plutusV3")]
    V3,
}

/// A Plutus script: language tag plus hex-encoded CBOR script bytes.
///
/// Per Cardano convention `cborHex` is the *single*-CBOR-wrapped form
/// (i.e. a CBOR bytestring whose contents are the raw Plutus script).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlutusScript {
    pub language: PlutusLanguage,
    pub cbor_hex: String,
}

/// Either a native or a Plutus script. Tagged by `kind`.
///
/// JSON:
/// ```json
/// { "kind": "native",  "script": { "type": "sig", "keyHash": "..." } }
/// { "kind": "plutus",  "script": { "language": "plutusV3", "cborHex": "..." } }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Script {
    Native { script: NativeScript },
    Plutus { script: PlutusScript },
}

impl Script {
    pub fn native(s: NativeScript) -> Self { Self::Native { script: s } }
    pub fn plutus(language: PlutusLanguage, cbor_hex: impl Into<String>) -> Self {
        Self::Plutus { script: PlutusScript { language, cbor_hex: cbor_hex.into() } }
    }
}
