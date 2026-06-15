//! Sweep Fate decisions over multiple scenarios / data types.
//! Usage: cargo run --release --bin fate_decision_sweep > fate_decisions.csv

use yaksha_cipher::evolve::{
    fate_decide_for, DataTypeHint, FateDecision, Scenario,
};

fn print_decision_csv(idx: usize, dec: &FateDecision) {
    println!(
        "{idx},{scenario:?},{dtype:?},{q:?},{qs:.2},{mode_before:?},{pat_before:?},{b_before},{mode_after:?},{pat_after:?},{b_after}",
        idx = idx,
        scenario = dec.scenario,
        dtype = dec.data_type,
        q = dec.quality,
        qs = dec.quality_score,
        mode_before = dec.control_before_rules.mode,
        pat_before = dec.control_before_rules.cascade_pattern,
        b_before = dec.control_before_rules.breaths,
        mode_after = dec.control_after_rules.mode,
        pat_after = dec.control_after_rules.cascade_pattern,
        b_after = dec.control_after_rules.breaths,
    );
}

fn main() {
    let key = b"fate_decision_sweep_master_key";
    let scenarios = [
        Scenario::Storage,
        Scenario::Channel,
        Scenario::HighRandomness,
        Scenario::Experimental,
    ];
    let dtypes = [
        DataTypeHint::Text,
        DataTypeHint::Binary,
        DataTypeHint::LowEntropy,
        DataTypeHint::HighEntropy,
    ];

    // CSV header
    println!("idx,scenario,data_type,quality,quality_score,mode_before,pattern_before,breaths_before,mode_after,pattern_after,breaths_after");

    let mut idx = 0usize;
    for (si, scenario) in scenarios.iter().enumerate() {
        for (di, dtype) in dtypes.iter().enumerate() {
            for n in 0..8u8 {
                let nonce = [si as u8, di as u8, n, 0, 0, 0, 0, 0];
                let plain: Vec<u8> = match dtype {
                    DataTypeHint::LowEntropy => vec![0u8; 64],
                    DataTypeHint::Text => b"Example text payload for Fate decision sweep."
                        .to_vec(),
                    DataTypeHint::Binary => (0..64u8).collect(),
                    DataTypeHint::HighEntropy => {
                        // simple LCG-based pseudo-random bytes
                        let mut lcg: u64 =
                            0x5DEECE66Du64 ^ (n as u64).wrapping_mul(0x9E3779B97F4A7C15);
                        (0..64)
                            .map(|_| {
                                lcg = lcg
                                    .wrapping_mul(0x5DEECE66Du64)
                                    .wrapping_add(0xB);
                                (lcg >> 16) as u8
                            })
                            .collect()
                    }
                };
                let dec = fate_decide_for(&plain, key, &nonce, *scenario, *dtype);
                print_decision_csv(idx, &dec);
                idx += 1;
            }
        }
    }
}

