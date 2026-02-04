use valhalla_tile_quattro::TileQuattroComplete;
use std::time::Instant;

fn main() {
    println!("测试 Fate 控制收敛...\n");
    
    let mut tile = TileQuattroComplete::new();
    
    // 显示初始方程
    println!("初始方程: {}\n", tile.triple_fate.cluster_equation.equation_str());
    
    let text = "Hello世界";
    let signals: Vec<f64> = text.chars().map(|c| c as u32 as f64).collect();
    
    let n = 1_000_000;
    let start = Instant::now();
    
    for (i, &signal) in signals.iter().cycle().take(n).enumerate() {
        tile.receive_data(signal);
        
        if i % 200_000 == 0 && i > 0 {
            let elapsed = start.elapsed().as_secs_f64();
            println!("  {}K: {:.0}/s | Tile:{}", i/1000, i as f64/elapsed, tile.completed_tiles.len());
        }
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    println!("\n{} 信号: {:.2}s | {:.0}/s | Tile:{}", n, elapsed, n as f64/elapsed, tile.completed_tiles.len());
    
    // 显示最终方程
    println!("最终方程: {}", tile.triple_fate.cluster_equation.equation_str());
}
