//! Python `transformer_grinder/grind.py` hash backend — byte-compatible ports for fair benchmark.

use crate::dl_front::{BodyFrontPorts, DlFrontConfig, BODY_PORT_SEED_HUB, BODY_PORT_SEED_STEM, BODY_PORT_SEED_TILE, DL_EMBED_DIM};

pub const HASH_SEED_TEXT: u64 = 0x5445_5854_5F46_4E54;
pub const NUM_QUADS: usize = 4;
const LCG_MUL: u64 = 6364136223846793005;

fn mask64(h: u64) -> u64 {
    h & 0xFFFF_FFFF_FFFF_FFFF
}

fn normalize(v: &mut [f64]) {
    let n: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
    for x in v.iter_mut() {
        *x /= n;
    }
}

fn pad_dim(v: &[f64], dim: usize) -> Vec<f64> {
    if v.len() >= dim {
        return v[..dim].to_vec();
    }
    let mut out = v.to_vec();
    out.resize(dim, 0.0);
    out
}

/// Match Python `hash_embed`.
pub fn hash_embed(text: &str, dim: usize, seed: u64) -> Vec<f64> {
    let mut h = mask64(seed);
    for b in text.bytes() {
        h = mask64(h.wrapping_mul(LCG_MUL).wrapping_add(b as u64));
    }
    let mut out = vec![0.0f64; dim];
    for i in 0..dim {
        h = mask64(h.wrapping_mul(LCG_MUL).wrapping_add(i as u64));
        out[i] = ((h >> 33) as f64 / (u32::MAX as f64)) * 2.0 - 1.0;
    }
    normalize(&mut out);
    out
}

fn project_port(embedding: &[f64], body_seed: u64, cap: usize) -> Vec<f64> {
    if embedding.is_empty() || cap == 0 {
        return Vec::new();
    }
    let mut out = vec![0.0f64; cap];
    let mut h = mask64(body_seed);
    for (i, slot) in out.iter_mut().enumerate() {
        h = mask64(h.wrapping_mul(LCG_MUL).wrapping_add(i as u64));
        let j = (h as usize) % embedding.len();
        let w = ((h >> 17) as f64 / (u32::MAX as f64)) * 2.0 - 1.0;
        *slot = embedding[j] * w;
    }
    normalize(&mut out);
    out
}

fn mix_vectors(a: &[f64], b: &[f64], dim: usize) -> Vec<f64> {
    let n = a.len().max(b.len());
    let mut mixed = vec![0.0f64; n];
    for i in 0..n {
        mixed[i] = a.get(i).copied().unwrap_or(0.0) + b.get(i).copied().unwrap_or(0.0);
    }
    let scaled: Vec<f64> = mixed.iter().map(|x| x / 2.0).collect();
    normalize(pad_dim(&scaled, dim).as_mut_slice());
    pad_dim(&scaled, dim)
}

fn grind_hash_quads(text: &str) -> (Vec<Vec<f64>>, &'static str) {
    let mut quads = Vec::with_capacity(NUM_QUADS);
    for i in 0..NUM_QUADS {
        let seed = mask64(HASH_SEED_TEXT.wrapping_add(i as u64 * 0x9E37_79B9_7F4A_7C15));
        quads.push(hash_embed(text, DL_EMBED_DIM, seed));
    }
    (quads, "grinder/hash_quad")
}

fn ports_from_quads(quads: &[Vec<f64>], encoder: &str, modality_label: &str, port_cap: usize) -> BodyFrontPorts {
    let mut qs = quads.to_vec();
    while qs.len() < NUM_QUADS {
        qs.push(qs.last().cloned().unwrap_or_else(|| hash_embed("empty", DL_EMBED_DIM, HASH_SEED_TEXT)));
    }

    let embed_dim = DL_EMBED_DIM;
    let mut flat_mean = vec![0.0f64; embed_dim];
    for q in &qs {
        let padded = pad_dim(q, embed_dim);
        for i in 0..embed_dim {
            flat_mean[i] += padded[i];
        }
    }
    for x in &mut flat_mean {
        *x /= NUM_QUADS as f64;
    }
    normalize(&mut flat_mean);

    let hub_mix = mix_vectors(&qs[0], &qs[1], embed_dim);
    let tile_mix = mix_vectors(&qs[1], &qs[2], embed_dim);
    let stem_mix = mix_vectors(&qs[2], &qs[3], embed_dim);

    BodyFrontPorts {
        hub: project_port(&hub_mix, BODY_PORT_SEED_HUB, port_cap),
        tile: project_port(&tile_mix, BODY_PORT_SEED_TILE, port_cap),
        stem: project_port(&stem_mix, BODY_PORT_SEED_STEM, port_cap),
        embedding: flat_mean,
        embed_dim,
        encoder: encoder.to_string(),
        modality_label: modality_label.to_string(),
    }
}

/// Match Python `grind_text(..., backend="hash")` ports for native QA.
pub fn grind_text_hash(text: &str, port_cap: usize) -> BodyFrontPorts {
    let (quads, enc) = grind_hash_quads(text);
    ports_from_quads(&quads, enc, "text", port_cap)
}

pub fn default_port_cap() -> usize {
    std::env::var("VALHALLA_GRINDER_SIGNALS_PER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(128)
        .max(16)
}
