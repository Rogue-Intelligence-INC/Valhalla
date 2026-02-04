//! 内存管理器长期运行性能测试
//! 测试: 系统原生 vs TileQuattro vs Fate+四元
//! 验证: 是否运行越久越快

use std::time::Instant;
use std::alloc::{alloc, dealloc, Layout};

const ROUNDS: usize = 10;           // 10轮测试
const OPS_PER_ROUND: usize = 10000; // 每轮10000次操作

// ============================================================
// TileQuattro (单池)
// ============================================================
struct TileQuattro {
    pool: Vec<u8>,
    offset: usize,
    total_size: usize,
    alloc_count: usize,
    reuse_list: Vec<(usize, usize)>,
}

impl TileQuattro {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            offset: 0,
            total_size: size,
            alloc_count: 0,
            reuse_list: Vec::with_capacity(1024),
        }
    }
    
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let aligned = (size + 7) & !7;
        
        // 尝试重用
        for i in 0..self.reuse_list.len() {
            if self.reuse_list[i].1 >= aligned {
                let (off, _) = self.reuse_list.swap_remove(i);
                self.alloc_count += 1;
                return Some(unsafe { self.pool.as_mut_ptr().add(off) });
            }
        }
        
        if self.offset + aligned > self.total_size {
            return None;
        }
        let ptr = unsafe { self.pool.as_mut_ptr().add(self.offset) };
        self.offset += aligned;
        self.alloc_count += 1;
        Some(ptr)
    }
    
    fn free(&mut self, offset: usize, size: usize) {
        self.reuse_list.push((offset, size));
    }
    
    fn reset_soft(&mut self) {
        // 软重置: 保留重用列表 (越用越快的关键)
        self.offset = 0;
    }
}

// ============================================================
// Fate + 四元系统
// ============================================================
struct FateQuattro {
    blackhole: Vec<u8>,
    meshbrain: Vec<u8>,
    multinova: Vec<Vec<u8>>,
    baseforce: Vec<u8>,
    
    bh_offset: usize,
    mb_offset: usize,
    mn_offsets: Vec<usize>,
    bf_offset: usize,
    
    // Fate 学习数据
    small_count: usize,
    large_count: usize,
    pattern_cache: Vec<(usize, u8)>,  // (size -> best_pool)
    
    // 重用列表
    bf_reuse: Vec<(usize, usize)>,
    
    alloc_count: usize,
    reuse_count: usize,
}

impl FateQuattro {
    fn new() -> Self {
        Self {
            blackhole: vec![0u8; 4 * 1024 * 1024],
            meshbrain: vec![0u8; 2 * 1024 * 1024],
            multinova: vec![vec![0u8; 1024 * 1024]; 4],
            baseforce: vec![0u8; 1024 * 1024],
            
            bh_offset: 0,
            mb_offset: 0,
            mn_offsets: vec![0; 4],
            bf_offset: 0,
            
            small_count: 0,
            large_count: 0,
            pattern_cache: Vec::with_capacity(256),
            
            bf_reuse: Vec::with_capacity(1024),
            
            alloc_count: 0,
            reuse_count: 0,
        }
    }
    
    fn fate_decide(&mut self, size: usize) -> u8 {
        // 检查缓存 (学习加速)
        for &(s, pool) in &self.pattern_cache {
            if s == size {
                return pool;
            }
        }
        
        // Fate 智能决策
        let pool = if size > 65536 {
            self.large_count += 1;
            0 // BlackHole
        } else if size > 4096 {
            1 // MeshBrain
        } else if size > 1024 {
            2 // MultiNova
        } else {
            self.small_count += 1;
            3 // BaseForce
        };
        
        // 缓存学习结果
        if self.pattern_cache.len() < 256 {
            self.pattern_cache.push((size, pool));
        }
        
        pool
    }
    
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let aligned = (size + 7) & !7;
        let pool = self.fate_decide(size);
        
        self.alloc_count += 1;
        
        match pool {
            0 => { // BlackHole
                if self.bh_offset + aligned > self.blackhole.len() { return None; }
                let ptr = unsafe { self.blackhole.as_mut_ptr().add(self.bh_offset) };
                self.bh_offset += aligned;
                Some(ptr)
            }
            1 => { // MeshBrain
                if self.mb_offset + aligned > self.meshbrain.len() { return None; }
                let ptr = unsafe { self.meshbrain.as_mut_ptr().add(self.mb_offset) };
                self.mb_offset += aligned;
                Some(ptr)
            }
            2 => { // MultiNova (轮询)
                let idx = self.alloc_count % 4;
                if self.mn_offsets[idx] + aligned > self.multinova[idx].len() { return None; }
                let ptr = unsafe { self.multinova[idx].as_mut_ptr().add(self.mn_offsets[idx]) };
                self.mn_offsets[idx] += aligned;
                Some(ptr)
            }
            3 => { // BaseForce (带重用)
                // 尝试重用
                for i in 0..self.bf_reuse.len() {
                    if self.bf_reuse[i].1 >= aligned {
                        let (off, _) = self.bf_reuse.swap_remove(i);
                        self.reuse_count += 1;
                        return Some(unsafe { self.baseforce.as_mut_ptr().add(off) });
                    }
                }
                if self.bf_offset + aligned > self.baseforce.len() { return None; }
                let ptr = unsafe { self.baseforce.as_mut_ptr().add(self.bf_offset) };
                self.bf_offset += aligned;
                Some(ptr)
            }
            _ => None
        }
    }
    
    fn free_baseforce(&mut self, offset: usize, size: usize) {
        self.bf_reuse.push((offset, size));
    }
    
    fn reset_soft(&mut self) {
        // 软重置: 保留学习数据和重用列表
        self.bh_offset = 0;
        self.mb_offset = 0;
        for o in &mut self.mn_offsets { *o = 0; }
        self.bf_offset = 0;
        // pattern_cache 和 bf_reuse 保留!
    }
}

// ============================================================
// 主测试
// ============================================================
fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📊 内存管理器长期运行性能测试");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("测试: {} 轮 × {} 次/轮", ROUNDS, OPS_PER_ROUND);
    println!();
    
    let sizes = [32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384];
    
    // ==================== 系统原生 malloc ====================
    println!("🔹 系统原生 malloc:");
    let mut malloc_times = Vec::new();
    
    for round in 0..ROUNDS {
        let start = Instant::now();
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            let layout = Layout::from_size_align(size, 8).unwrap();
            unsafe {
                let ptr = alloc(layout);
                if !ptr.is_null() { dealloc(ptr, layout); }
            }
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        malloc_times.push(elapsed);
        print!("   轮 {:2}: {:>6.3} ms", round + 1, elapsed);
        if round > 0 {
            let diff = elapsed - malloc_times[round - 1];
            if diff < 0.0 { print!(" (快 {:.3} ms)", -diff); }
            else { print!(" (慢 {:.3} ms)", diff); }
        }
        println!();
    }
    
    // ==================== TileQuattro ====================
    println!();
    println!("🔹 TileQuattro:");
    let mut tile = TileQuattro::new(16 * 1024 * 1024);
    let mut tile_times = Vec::new();
    
    for round in 0..ROUNDS {
        let start = Instant::now();
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            tile.alloc(size);
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tile_times.push(elapsed);
        print!("   轮 {:2}: {:>6.3} ms", round + 1, elapsed);
        if round > 0 {
            let diff = elapsed - tile_times[round - 1];
            if diff < 0.0 { print!(" (快 {:.3} ms) ⚡", -diff); }
            else { print!(" (慢 {:.3} ms)", diff); }
        }
        println!();
        tile.reset_soft();  // 保留重用列表
    }
    
    // ==================== Fate + 四元 ====================
    println!();
    println!("🔹 Fate + 四元:");
    let mut fate = FateQuattro::new();
    let mut fate_times = Vec::new();
    
    for round in 0..ROUNDS {
        let start = Instant::now();
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            fate.alloc(size);
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        fate_times.push(elapsed);
        print!("   轮 {:2}: {:>6.3} ms", round + 1, elapsed);
        if round > 0 {
            let diff = elapsed - fate_times[round - 1];
            if diff < 0.0 { print!(" (快 {:.3} ms) ⚡", -diff); }
            else { print!(" (慢 {:.3} ms)", diff); }
        }
        println!();
        fate.reset_soft();  // 保留学习数据
    }
    
    // ==================== 结果汇总 ====================
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📈 结果汇总");
    println!("═══════════════════════════════════════════════════════════════════");
    
    let malloc_first = malloc_times[0];
    let malloc_last = malloc_times[ROUNDS - 1];
    let tile_first = tile_times[0];
    let tile_last = tile_times[ROUNDS - 1];
    let fate_first = fate_times[0];
    let fate_last = fate_times[ROUNDS - 1];
    
    println!();
    println!("首轮 vs 末轮:");
    println!("  系统原生:   {:>6.3} ms → {:>6.3} ms  (变化: {:+.1}%)", 
             malloc_first, malloc_last, (malloc_last - malloc_first) / malloc_first * 100.0);
    println!("  TileQuattro: {:>6.3} ms → {:>6.3} ms  (变化: {:+.1}%)", 
             tile_first, tile_last, (tile_last - tile_first) / tile_first * 100.0);
    println!("  Fate+四元:  {:>6.3} ms → {:>6.3} ms  (变化: {:+.1}%)", 
             fate_first, fate_last, (fate_last - fate_first) / fate_first * 100.0);
    
    println!();
    println!("Fate 学习统计:");
    println!("  模式缓存: {} 条", fate.pattern_cache.len());
    println!("  重用次数: {}", fate.reuse_count);
    println!("  小块计数: {}", fate.small_count);
    println!("  大块计数: {}", fate.large_count);
    
    println!();
    if fate_last < fate_first * 0.9 {
        println!("✅ Fate+四元 越用越快! (提升 {:.1}%)", (1.0 - fate_last / fate_first) * 100.0);
    }
    if tile_last < tile_first * 0.9 {
        println!("✅ TileQuattro 越用越快! (提升 {:.1}%)", (1.0 - tile_last / tile_first) * 100.0);
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
}
