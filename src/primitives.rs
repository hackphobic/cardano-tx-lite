//! Primitive aliases, hex-string newtypes, and serde helpers for big-number stringification.

use serde::{Deserialize, Serialize};

/// Serialize a `u64` as a JSON string (and back).
///
/// Cardano lovelace amounts can exceed `2^53`, which is the safe-integer limit in JavaScript
/// JSON parsers. Stringify them.
pub mod stringified_u64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Serialize an `i128` as a JSON string (and back). Used for mint quantities (burns are negative).
pub mod stringified_i128 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &i128, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<i128, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Generate a `#[serde(transparent)]` newtype wrapping a hex `String`.
macro_rules! hex_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            #[inline]
            pub fn new(hex: impl Into<String>) -> Self { Self(hex.into()) }
            #[inline]
            pub fn as_str(&self) -> &str { &self.0 }
        }

        impl From<String> for $name {
            #[inline]
            fn from(s: String) -> Self { Self(s) }
        }
        impl From<&str> for $name {
            #[inline]
            fn from(s: &str) -> Self { Self(s.to_string()) }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

hex_newtype!(
    /// 32-byte transaction hash, hex-encoded.
    TxHash
);
hex_newtype!(
    /// 28-byte script hash, hex-encoded.
    ScriptHash
);
hex_newtype!(
    /// 28-byte minting policy id (equivalent to a script hash).
    PolicyId
);
hex_newtype!(
    /// Asset name, hex-encoded (0–32 bytes).
    AssetName
);
hex_newtype!(
    /// 32-byte Plutus datum hash, hex-encoded.
    DataHash
);
hex_newtype!(
    /// 28-byte key hash (verification-key hash), hex-encoded.
    KeyHash
);
hex_newtype!(
    /// 28-byte stake-pool key hash, hex-encoded.
    PoolId
);
hex_newtype!(
    /// 28-byte DRep key hash, hex-encoded.
    DRepKeyHash
);
hex_newtype!(
    /// 32-byte VRF key hash, hex-encoded.
    VrfKeyHash
);
hex_newtype!(
    /// Generic 28-byte hash, hex-encoded.
    Hash28
);
hex_newtype!(
    /// Generic 32-byte hash, hex-encoded.
    Hash32
);

/// Lovelace amount (1 ADA = 1_000_000 lovelace). Serializes as a JSON string.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Lovelace(#[serde(with = "stringified_u64")] pub u64);

impl From<u64> for Lovelace {
    #[inline]
    fn from(v: u64) -> Self { Self(v) }
}
impl Lovelace {
    #[inline]
    pub fn get(self) -> u64 { self.0 }
}

/// Native-token quantity (always non-negative in tx outputs / withdrawals).
/// Serializes as a JSON string.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quantity(#[serde(with = "stringified_u64")] pub u64);

impl From<u64> for Quantity {
    #[inline]
    fn from(v: u64) -> Self { Self(v) }
}

/// Mint-field quantity. Can be negative (burns). Serializes as a JSON string.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MintQuantity(#[serde(with = "stringified_i128")] pub i128);

impl From<i128> for MintQuantity {
    #[inline]
    fn from(v: i128) -> Self { Self(v) }
}
impl From<i64> for MintQuantity {
    #[inline]
    fn from(v: i64) -> Self { Self(v as i128) }
}

/// Absolute slot number.
pub type Slot = u64;

/// Epoch number.
pub type Epoch = u64;
