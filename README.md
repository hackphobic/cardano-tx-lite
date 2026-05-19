# cardano-tx-lite

Simplified Cardano (Conway-era) transaction types, designed for web-API JSON I/O.

Types only — no CBOR, no signing, no balancing. The shape on the wire is what you'd
hand off between a frontend and a tx-builder backend (e.g. one wrapping
`cardano-serialization-lib` or `whisky`).

## Conventions

| Concern                | Representation                                         |
| ---------------------- | ------------------------------------------------------ |
| Addresses              | bech32 strings (`addr1…`, `stake1…`)                   |
| Hashes, key hashes     | hex strings (no `0x`)                                  |
| Asset names            | hex strings                                            |
| Lovelace / token qty   | JSON **string** (avoids `2^53` JS precision loss)      |
| Mint quantities        | JSON **string**, can be negative                       |
| Field naming           | `camelCase`                                            |
| Enum discriminant      | `"type"` field (or `"kind"` for `Script`)              |
| `PlutusData`           | CSL `DetailedSchema` — see below                       |

## Grouping principle

Everything a script needs to execute lives next to the action it authorizes — not in a
separate witness-set sidecar that has to be cross-referenced by index. Inputs also carry
their resolved address+value, matching whisky's `tx_in` shape, so backends don't need a
sidecar UTxO map.

| Action            | Wrapper       | Inline fields                                       |
| ----------------- | ------------- | --------------------------------------------------- |
| Spending a UTxO   | `TxInput`     | `address`, `value`, `datum`, `redeemer`, `scriptRef` |
| Minting / burning | `MintPolicy`  | `assets`, `script`, `redeemer`                      |
| Withdrawing       | `Withdrawal`  | `amount`, `script`, `redeemer`                      |
| Certificates      | `CertEntry`   | `cert`, `script`, `redeemer`                        |

Redeemer **tag** and **index** are implicit from attachment site, so the `Redeemer`
struct only carries `data` + `exUnits`.

## PlutusData → whisky / CSL

`PlutusData` is laid out so that `serde_json::to_string(&pd)` produces exactly the JSON
that `csl::PlutusData::from_json(s, DetailedSchema)` accepts. Pipe it straight into
whisky:

```rust
use cardano_tx_lite::PlutusData;
use whisky::builder::data::WData;

let datum = PlutusData::constr(0, vec![
    PlutusData::bytes("deadbeef"),
    PlutusData::int(42_i64),
]);

let json = serde_json::to_string(&datum).unwrap();
let cbor_hex = WData::JSON(json).to_cbor().unwrap();   // ✓ parses
```

## Module map

| Module       | Highlights                                                                       |
| ------------ | -------------------------------------------------------------------------------- |
| `primitives` | hex newtypes (`TxHash`, `ScriptHash`, `PolicyId`, …), `Lovelace`, big-int helpers |
| `address`    | `Address`, `RewardAddress`, `Credential`                                          |
| `value`      | `Value` (`coin` + multi-asset bundle)                                             |
| `plutus`     | `PlutusData`, `PlutusInt`, `Redeemer`, `ExUnits`                                  |
| `script`     | `NativeScript`, `PlutusScript`, `Script`                                          |
| `cert`       | Conway certs: stake, pool, DRep, committee, combined delegations                  |
| `tx`         | `TxInput`, `TxOutput`, `MintPolicy`, `Withdrawal`, `CertEntry`, `TxBody`          |

## Quick example

```rust
use cardano_tx_lite::*;

// A script-locked input carries the full resolved UTxO info + spending context.
let spending_input = TxInput::new(
    "ee".repeat(32),
    1,
    "addr1qscriptlocked",
    Value::ada(5_000_000),
)
.with_inline_datum(PlutusData::int(99_i64))
.with_redeemer(Redeemer::new(
    PlutusData::unit(),
    ExUnits { mem: 1_400_000, steps: 500_000_000 },
))
.with_script_ref(Script::plutus(PlutusLanguage::V3, "5901aa..."));

let body = TxBody {
    inputs: vec![spending_input],
    outputs: vec![TxOutput::new("addr1q…", Value::ada(1_500_000))],
    fee: Lovelace(180_000),
    ..TxBody::new()
};

println!("{}", serde_json::to_string_pretty(&body).unwrap());
```

Each `TxInput` carries its resolved `address` and `value`, matching the shape
whisky's `tx_in(tx_hash, index, &[Asset], address)` takes. A tx-builder backend
can rebuild the tx straight from the body — no sidecar UTxO-resolution map needed.

See `examples/full_tx.rs` for every field populated.

## Tests

```
cargo test
```

20 JSON shape / roundtrip tests, including PlutusData DetailedSchema compatibility,
input-with-attached-context, script-controlled withdrawals, script-witnessed certs,
and mint with grouped script + redeemer.

## Cert coverage (Conway)

Pre-Conway: `stakeRegistration`, `stakeDeregistration`, `stakeDelegation`,
`poolRegistration`, `poolRetirement`.

Conway additions:
`reg`, `unreg`, `voteDeleg`, `stakeVoteDeleg`, `stakeRegDeleg`, `voteRegDeleg`,
`stakeVoteRegDeleg`, `authCommitteeHot`, `resignCommitteeCold`,
`regDRep`, `unregDRep`, `updateDRep`.

Out of scope for "simplified": voting procedures, proposal procedures, treasury &
donation fields, full witness set, auxiliary-data body.
