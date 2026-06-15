use yaksha_cipher_gpu::{GpuContext, wave_particle::WaveParticle};
fn main() {
    let ctx = GpuContext::new().expect("No GPU");
    let wp = WaveParticle::new(&ctx);
    let key = b"test_key";
    let size = 16384;
    let data: Vec<u8> = (0..size).map(|i| i as u8).collect();
    let mut data_mod = data.clone();
    data_mod[0] ^= 1;
    
    for rounds in [1, 2, 3, 4, 6, 8] {
        let e1 = wp.apply(&ctx, &data, key, rounds);
        let e2 = wp.apply(&ctx, &data_mod, key, rounds);
        let diff: u32 = e1.iter().zip(&e2).map(|(a,b)| (a^b).count_ones()).sum();
        let pct = diff as f64 / (size * 8) as f64 * 100.0;
        println!("{} rounds: {:.1}%", rounds, pct);
    }
}
