//! Simplified Cardano (Conway-era) transaction types, optimized for JSON I/O over web APIs.
//!
//! ## Design
//! - **Addresses** are bech32 strings (`addr1…`, `stake1…`). No on-chain bytes parsing here.
//! - **Hashes / key-hashes / asset names** are hex-encoded strings (no `0x` prefix).
//! - **Amounts** (lovelace, native-token quantities, mint quantities) are serialized as JSON
//!   strings to avoid 2^53 precision loss in JS clients.
//! - **PlutusData** serializes to CSL's *DetailedSchema*, so the JSON can be fed directly to
//!   `cardano-serialization-lib`'s `PlutusData::from_json(_, DetailedSchema)` (or whisky's
//!   `WData::JSON(...).to_cbor()`).
//! - **Field naming** is `camelCase` throughout.
//!
//! ## Scope
//! Types only — no CBOR, no signing, no balancing. Designed to be the on-the-wire shape
//! that a frontend / API exchanges with a builder backend.

pub mod address;
pub mod cert;
pub mod plutus;
pub mod primitives;
pub mod script;
pub mod tx;
pub mod value;

pub use address::*;
pub use cert::*;
pub use plutus::*;
pub use primitives::*;
pub use script::*;
pub use tx::*;
pub use value::*;
