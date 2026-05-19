//! Build a sample Conway-era tx body and print it as JSON.
//!
//!     cargo run --example demo

use cardano_tx_lite::*;
use std::collections::BTreeMap;

fn main() {
    let policy = PolicyId::from("ab".repeat(28));

    // A native-token output carrying an inline datum.
    let token_out = TxOutput::new(
        "addr1q9example",
        Value::ada(2_000_000)
            .with_asset(policy.clone(), AssetName::from("4d79546f6b656e"), 100),
    )
    .with_inline_datum(PlutusData::constr(
        0,
        vec![
            PlutusData::bytes("deadbeef"),
            PlutusData::int(42_i64),
            PlutusData::list(vec![PlutusData::int(1_i64), PlutusData::int(2_i64)]),
        ],
    ))
    .with_script_ref(Script::plutus(PlutusLanguage::V3, "59012a..."));

    // Mint 50 of one asset, burn 5 of another. Both belong to the same policy, so they
    // share a single `MintPolicy` entry with its own script + redeemer.
    let mut mint: Mint = BTreeMap::new();
    mint.insert(
        policy.clone(),
        MintPolicy::new()
            .mint(AssetName::from("4d79546f6b656e"), 50)
            .mint(AssetName::from("4275726e4d65"), -5)
            .with_script(Script::plutus(PlutusLanguage::V3, "590200..."))
            .with_redeemer(Redeemer::new(
                PlutusData::unit(),
                ExUnits { mem: 200_000, steps: 80_000_000 },
            )),
    );

    // Withdraw rewards from a key-controlled stake address (no script).
    let mut withdrawals: Withdrawals = BTreeMap::new();
    withdrawals.insert(
        RewardAddress::from("stake1uxexample"),
        Withdrawal::new(Lovelace(7_500_000)),
    );

    // A Conway-only combined cert.
    let certs = vec![CertEntry::new(Cert::StakeVoteRegDeleg {
        credential: Credential::key("aa".repeat(28)),
        pool: PoolId::from("bb".repeat(28)),
        drep: DRep::AlwaysAbstain,
        deposit: Lovelace(2_000_000),
    })];

    let body = TxBody {
        inputs: vec![
            TxInput::new("11".repeat(32), 0, "addr1q9pubkey", Value::ada(8_000_000)),
            TxInput::new("22".repeat(32), 3, "addr1q9pubkey", Value::ada(3_000_000)),
        ],
        reference_inputs: vec![
            TxInput::new("33".repeat(32), 0, "addr1qrefscript", Value::ada(50_000_000)),
        ],
        outputs: vec![token_out],
        fee: Lovelace(187_493),
        ttl: Some(120_000_000),
        certs,
        withdrawals,
        mint,
        required_signers: vec![KeyHash::from("cc".repeat(28))],
        network_id: Some(1),
        ..TxBody::new()
    };

    println!("{}", serde_json::to_string_pretty(&body).unwrap());
}
