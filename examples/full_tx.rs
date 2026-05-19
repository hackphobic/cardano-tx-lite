//! A worked example that fills *every* field of [`TxBody`] with realistic-looking data,
//! and demonstrates how datum/redeemer/script context attaches to each action.
//!
//! Run with:
//!
//!     cargo run --example full_tx
//!
//! Hashes are placeholder byte-repeats (`aa…aa`) so it's easy to see where each piece
//! ends up in the JSON output. None of this would validate on-chain — the point is to
//! show the *shape* a backend tx-builder would accept over a web API.

use cardano_tx_lite::*;
use std::collections::BTreeMap;

// ── Placeholder-bytes helpers ────────────────────────────────────────────────────────────
fn h28(b: &str) -> String { b.repeat(28) }   // 28-byte hash → 56 hex chars
fn h32(b: &str) -> String { b.repeat(32) }   // 32-byte hash → 64 hex chars

fn main() {
    // ─────────────────────────────────────────────────────────────────────────────
    // 1. Inputs.
    //    - Two plain key-locked inputs (no spending context needed).
    //    - One script-locked input that carries everything needed to spend it:
    //      the locked datum (inline), the redeemer, and the validator script.
    // ─────────────────────────────────────────────────────────────────────────────
    let key_input_a = TxInput::new(h32("11"), 0, "addr1q9alicepubkey", Value::ada(8_000_000));
    let key_input_b = TxInput::new(h32("22"), 3, "addr1q9alicepubkey", Value::ada(3_000_000));

    let script_input = TxInput::new(
        h32("ee"),
        1,
        "addr1qscriptlocked",
        Value::ada(5_000_000)
            .with_asset(PolicyId::from(h28("aa")), AssetName::from("4d79546f6b656e"), 50),
    )
    .with_inline_datum(PlutusData::constr(
        0,
        vec![
            PlutusData::bytes(h28("a1")),                  // owner pubkey hash
            PlutusData::int(1_704_067_200_i64),            // deadline (slot)
        ],
    ))
    .with_redeemer(Redeemer::new(
        PlutusData::constr(1, vec![PlutusData::int(7_i64)]),   // e.g. Redeem { amount = 7 }
        ExUnits { mem: 1_400_000, steps: 500_000_000 },
    ))
    .with_script_ref(Script::plutus(PlutusLanguage::V3, "5901aa..."));

    let inputs = vec![key_input_a, key_input_b, script_input];

    // Reference inputs are read-only — they don't carry spending context, just a pointer
    // plus their resolved address+value (so the builder knows what's stored there).
    let reference_inputs = vec![
        // A UTxO whose attached reference script we want to use instead of inlining it.
        TxInput::new(h32("33"), 0, "addr1qrefscriptholder", Value::ada(50_000_000)),
        // An oracle UTxO whose inline datum we read but don't spend.
        TxInput::new(h32("44"), 1, "addr1qoracle", Value::ada(2_000_000)),
    ];

    // ─────────────────────────────────────────────────────────────────────────────
    // 2. Collateral. Required because we're spending a Plutus-locked input.
    // ─────────────────────────────────────────────────────────────────────────────
    let collateral_inputs = vec![
        TxInput::new(h32("55"), 0, "addr1qcollateralowner", Value::ada(5_000_000)),
    ];
    let collateral_return = Some(
        TxOutput::new("addr1qcollateralreturn", Value::ada(4_500_000)),
    );
    let total_collateral = Some(Lovelace(5_000_000));

    // ─────────────────────────────────────────────────────────────────────────────
    // 3. Outputs — one of each interesting shape.
    // ─────────────────────────────────────────────────────────────────────────────
    let policy_a = PolicyId::from(h28("aa"));
    let policy_b = PolicyId::from(h28("bb"));
    let asset_token = AssetName::from("4d79546f6b656e");   // "MyToken"
    let asset_burn  = AssetName::from("4275726e4d65");     // "BurnMe"

    // (a) Pure-ADA change output.
    let out_change = TxOutput::new("addr1qchange", Value::ada(8_750_000));

    // (b) Multi-asset output to a script address, carrying a *datum hash* (legacy style).
    let out_to_script = TxOutput::new(
        "addr1qscriptaddr",
        Value::ada(2_000_000).with_asset(policy_a.clone(), asset_token.clone(), 100),
    )
    .with_datum_hash(DataHash::from(h32("dd")));

    // (c) Output with an *inline datum* — a nested PlutusData value exercising every variant.
    let out_inline = TxOutput::new("addr1qinline", Value::ada(3_500_000))
        .with_inline_datum(PlutusData::constr(
            0,
            vec![
                PlutusData::bytes(h28("ef")),
                PlutusData::int(42_i64),
                PlutusData::list(vec![
                    PlutusData::int(1_i64),
                    PlutusData::int(2_i64),
                    PlutusData::int("170141183460469231731687303715884105728"), // > i64
                ]),
                PlutusData::map(vec![
                    (PlutusData::bytes("01"), PlutusData::int(100_i64)),
                    (PlutusData::bytes("02"), PlutusData::unit()),
                ]),
            ],
        ));

    // (d) Output carrying an attached Plutus V3 reference script.
    let out_with_script_ref = TxOutput::new("addr1qrefscript", Value::ada(50_000_000))
        .with_script_ref(Script::plutus(PlutusLanguage::V3, "59012aabbccddeeff00112"));

    // (e) Output carrying a native (2-of-3 multisig) reference script.
    let out_with_native_ref = TxOutput::new("addr1qnativescript", Value::ada(1_400_000))
        .with_script_ref(Script::native(NativeScript::AtLeast {
            required: 2,
            scripts: vec![
                NativeScript::Sig { key_hash: KeyHash::from(h28("a1")) },
                NativeScript::Sig { key_hash: KeyHash::from(h28("a2")) },
                NativeScript::Sig { key_hash: KeyHash::from(h28("a3")) },
            ],
        }));

    let outputs = vec![
        out_change,
        out_to_script,
        out_inline,
        out_with_script_ref,
        out_with_native_ref,
    ];

    // ─────────────────────────────────────────────────────────────────────────────
    // 4. Certificates — each `CertEntry` bundles a cert with its optional script
    //    + redeemer (only needed when the cert's credential is a script hash).
    // ─────────────────────────────────────────────────────────────────────────────
    let pool_params = PoolParams {
        id: PoolId::from(h28("70")),
        vrf_key_hash: VrfKeyHash::from(h32("71")),
        pledge: Lovelace(500_000_000_000),
        cost: Lovelace(340_000_000),
        margin: UnitInterval { numerator: 3, denominator: 100 },
        reward_account: RewardAddress::from("stake1upooloperator"),
        owners: vec![KeyHash::from(h28("72")), KeyHash::from(h28("73"))],
        relays: vec![
            Relay::SingleHostAddr {
                port: Some(3001),
                ipv4: Some("198.51.100.10".into()),
                ipv6: None,
            },
            Relay::SingleHostName {
                port: Some(3001),
                dns_name: "relay1.example.com".into(),
            },
        ],
        metadata: Some(PoolMetadata {
            url: "https://example.com/pool.json".into(),
            hash: Hash32::from(h32("7a")),
        }),
    };

    let certs = vec![
        // Pool registration — pool operator signs, no Plutus needed.
        CertEntry::new(Cert::PoolRegistration { pool_params }),

        // Register + delegate a *key*-controlled stake credential — no script.
        CertEntry::new(Cert::StakeRegDeleg {
            credential: Credential::key(h28("c0")),
            pool: PoolId::from(h28("70")),
            deposit: Lovelace(2_000_000),
        }),

        // Register + vote-delegate a *script*-controlled stake credential —
        // the cert carries the witnessing script + a redeemer.
        CertEntry::new(Cert::VoteRegDeleg {
            credential: Credential::script(h28("c1")),
            drep: DRep::KeyHash { hash: DRepKeyHash::from(h28("d0")) },
            deposit: Lovelace(2_000_000),
        })
        .with_script(Script::plutus(PlutusLanguage::V3, "59010f..."))
        .with_redeemer(Redeemer::new(
            PlutusData::unit(),
            ExUnits { mem: 80_000, steps: 30_000_000 },
        )),

        // Register a DRep with a metadata anchor.
        CertEntry::new(Cert::RegDRep {
            credential: Credential::key(h28("d1")),
            deposit: Lovelace(500_000_000),
            anchor: Some(Anchor {
                url: "https://example.com/drep.json".into(),
                data_hash: Hash32::from(h32("d2")),
            }),
        }),

        // Authorize a hot key for an existing cold committee credential.
        CertEntry::new(Cert::AuthCommitteeHot {
            cold_credential: Credential::key(h28("cc")),
            hot_credential:  Credential::key(h28("cd")),
        }),

        // Retire a pool at the start of epoch 500.
        CertEntry::new(Cert::PoolRetirement {
            pool: PoolId::from(h28("7f")),
            epoch: 500,
        }),
    ];

    // ─────────────────────────────────────────────────────────────────────────────
    // 5. Withdrawals — one plain key-controlled, one script-controlled.
    // ─────────────────────────────────────────────────────────────────────────────
    let mut withdrawals: Withdrawals = BTreeMap::new();
    withdrawals.insert(
        RewardAddress::from("stake1ualice"),
        Withdrawal::new(Lovelace(7_500_000)),
    );
    withdrawals.insert(
        RewardAddress::from("stake1uscriptcontrolled"),
        Withdrawal::new(Lovelace(125_000))
            .with_script(Script::plutus(PlutusLanguage::V3, "5900ff..."))
            .with_redeemer(Redeemer::new(
                PlutusData::unit(),
                ExUnits { mem: 50_000, steps: 20_000_000 },
            )),
    );

    // ─────────────────────────────────────────────────────────────────────────────
    // 6. Mint — each policy carries its own script + redeemer alongside its assets.
    //    Policy A is Plutus (mint + burn). Policy B is a native script (single NFT).
    // ─────────────────────────────────────────────────────────────────────────────
    let mut mint: Mint = BTreeMap::new();
    mint.insert(
        policy_a.clone(),
        MintPolicy::new()
            .mint(asset_token.clone(), 100)
            .mint(asset_burn.clone(),  -5)
            .with_script(Script::plutus(PlutusLanguage::V3, "590111..."))
            .with_redeemer(Redeemer::new(
                PlutusData::constr(0, vec![]),   // e.g. MintAction
                ExUnits { mem: 300_000, steps: 120_000_000 },
            )),
    );
    mint.insert(
        policy_b.clone(),
        MintPolicy::new()
            .mint(AssetName::from("4e4654"), 1)   // single NFT, "NFT"
            .with_script(Script::native(NativeScript::Sig {
                key_hash: KeyHash::from(h28("b1")),
            })),
    );

    // ─────────────────────────────────────────────────────────────────────────────
    // 7. Required signers.
    // ─────────────────────────────────────────────────────────────────────────────
    let required_signers = vec![
        KeyHash::from(h28("5a")),
        KeyHash::from(h28("5b")),
    ];

    // ─────────────────────────────────────────────────────────────────────────────
    // 8. Assemble.
    // ─────────────────────────────────────────────────────────────────────────────
    let body = TxBody {
        inputs,
        reference_inputs,
        collateral_inputs,
        outputs,
        collateral_return,
        total_collateral,
        fee: Lovelace(187_493),
        ttl: Some(120_000_000),
        validity_start: Some(119_990_000),
        certs,
        withdrawals,
        mint,
        required_signers,
        network_id: Some(1),
        auxiliary_data_hash: Some(Hash32::from(h32("a0"))),
        script_data_hash:    Some(Hash32::from(h32("5d"))),
    };

    println!("{}", serde_json::to_string_pretty(&body).unwrap());
}
