//! Multi-asset value (ADA + native tokens).

use crate::primitives::{AssetName, Lovelace, PolicyId, Quantity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A multi-asset value: lovelace plus optional native-token bundle.
///
/// JSON shape:
/// ```json
/// {
///   "coin": "2000000",
///   "assets": {
///     "<policyId hex>": { "<assetName hex>": "1" }
///   }
/// }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    /// Lovelace amount.
    pub coin: Lovelace,
    /// Native tokens, keyed by policy id then asset name. Omitted in JSON when empty.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assets: BTreeMap<PolicyId, BTreeMap<AssetName, Quantity>>,
}

impl Value {
    /// Pure-ADA value.
    pub fn ada(lovelace: u64) -> Self {
        Self { coin: Lovelace(lovelace), assets: BTreeMap::new() }
    }

    /// Add a single native-token entry. Existing entries are replaced.
    pub fn with_asset(mut self, policy: PolicyId, asset: AssetName, qty: u64) -> Self {
        self.assets.entry(policy).or_default().insert(asset, Quantity(qty));
        self
    }

    /// `true` if no native tokens are present.
    pub fn is_pure_ada(&self) -> bool {
        self.assets.is_empty()
    }
}
