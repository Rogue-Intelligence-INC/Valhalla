//! 内存重用性能测试
//! 模拟真实场景: 分配 → 使用 → 释放 → 重用

use std::time::Instant;
use std::alloc::{alloc, dealloc, Layout};

const ROUNDS: usize = 10;
const OPS_PER_ROUND: usize = 5000;

// ============================================================
// TileQuattro (带重用)
// ============================================================
struct TileQuattro {
    pool: Vec<u8>,
    reuse_list: Vec<(usize, usize)>,
    alloc_count: usize,
    reuse_count: usize,
    next_offset: usize,
}

impl TileQuattro {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            reuse_list: Vec::with_capacity(4096),
            alloc_count: 0,
            reuse_count: 0,
            next_offset: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> (usize, *mut u8) {
        let aligned = (size + 7) & !7;
        self.alloc_count += 1;
        
        // 优先重用
        let mut best_idx = None;
        let mut best_waste = usize::MAX;
        
        for (i, &(_, s)) in self.reuse_list.iter().enumerate() {
            if s >= aligned && s - aligned < best_waste {
                best_waste = s - aligned;
                best_idx = Some(i);
                if best_waste == 0 { break; }  // 完美匹配
            }
        }
        
        if let Some(i) = best_idx {
            let (off, s) = self.reuse_list.swap_remove(i);
            self.reuse_count += 1;
            return (off, unsafe { self.pool.as_mut_ptr().add(off) });
        }
        
        // 新分配
        let off = self.next_offset;
        self.next_offset += aligned;
        (off, unsafe { self.pool.as_mut_ptr().add(off) })
    }
    
    fn free(&mut self, offset: usize, size: usize) {
        self.reuse_list.push((offset, (size + 7) & !7));
    }
}

// ============================================================
// Fate + 四元 (带学习和重用)
// ============================================================
struct FateQuattro {
    pools: [Vec<u8>; 4],
    offsets: [usize; 4],
    reuse_lists: [Vec<(usize, usize)>; 4],
    
    // Fate 学习
    size_to_pool: Vec<(usize, usize)>,  // size -> pool_idx
    alloc_count: usize,
    reuse_count: usize,
    cache_hits: usize,
}

impl FateQuattro {
    fn new() -> Self {
        Self {
            pools: [
                vec![0u8; 4 * 1024 * 1024],  // BlackHole
                vec![0u8; 2 * 1024 * 1024],  // MeshBrain
                vec![0u8; 4 * 1024 * 1024],  // MultiNova
                vec![0u8; 2 * 1024 * 1024],  // BaseForce
            ],
            offsets: [0; 4],
            reuse_lists: [
                Vec::with_capacity(1024),
                Vec::with_capacity(1024),
                Vec::with_capacity(1024),
                Vec::with_capacity(2048),
            ],
            size_to_pool: Vec::with_capacity(256),
            alloc_count: 0,
            reuse_count: 0,
            cache_hits: 0,
        }
    }
    
    fn fate_decide(&mut self, size: usize) -> usize {
        // 检查学习缓存
        for &(s, p) in &self.size_to_pool {
            if s == size {
                self.cache_hits += 1;
                return p;
            }
        }
        
        // Fate 决策
        let pool = if size > 16384 { 0 }      // BlackHole
                   else if size > 4096 { 1 }  // MeshBrain
                   else if size > 512 { 2 }   // MultiNova
                   else { 3 };                // BaseForce
        
        // 学习
        if self.size_to_pool.len() < 256 {
            self.size_to_pool.push((size, pool));
        }
        pool
    }
    
    fn alloc(&mut self, size: usize) -> (usize, usize, *mut u8) {
        let aligned = (size + 7) & !7;
        let pool_idx = self.fate_decide(size);
        self.alloc_count += 1;
        
        // 优先重用
        let reuse = &mut self.reuse_lists[pool_idx];
        let mut best_idx = None;
        let mut best_waste = usize::MAX;
        
        for (i, &(_, s)) in reuse.iter().enumerate() {
            if s >= aligned && s - aligned < best_waste {
                best_waste = s - aligned;
                best_idx = Some(i);
                if best_waste == 0 { break; }
            }
        }
        
        if let Some(i) = best_idx {
            let (off, s) = reuse.swap_remove(i);
            self.reuse_count += 1;
            return (pool_idx, off, unsafe { self.pools[pool_idx].as_mut_ptr().add(off) });
        }
        
        // 新分配
        let off = self.offsets[pool_idx];
        self.offsets[pool_idx] += aligned;
        (pool_idx, off, unsafe { self.pools[pool_idx].as_mut_ptr().add(off) })
    }
    
    fn free(&mut self, pool_idx: usize, offset: usize, size: usize) {
        self.reuse_lists[pool_idx].push((offset, (size + 7) & !7));
    }
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📊 内存重用性能测试 (分配→释放→重用)");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("测试: {} 轮 × {} 次分配/释放循环", ROUNDS, OPS_PER_ROUND);
    println!();
    
    let sizes = [64, 128, 256, 512, 1024, 2048, 4096];
    
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
                std::ptr::write_bytes(ptr, 0xAB, size);  // 使用内存
                dealloc(ptr, layout);
            }
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        malloc_times.push(elapsed);
        let change = if round > 0 { 
            let diff = ((elapsed - malloc_times[0]) / malloc_times[0] * 100.0) as i32;
            format!("({:+}%)", diff)
        } else { String::new() };
        println!("   轮 {:2}: {:>6.3} ms {}", round + 1, elapsed, change);
    }
    
    // ==================== TileQuattro ====================
    println!();
    println!("🔹 TileQuattro (带重用):");
    let mut tile = TileQuattro::new(64 * 1024 * 1024);
    let mut tile_times = Vec::new();
    let mut allocs: Vec<(usize, usize)> = Vec::with_capacity(OPS_PER_ROUND);
    
    for round in 0..ROUNDS {
        allocs.clear();
        let start = Instant::now();
        
        // 分配
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            let (off, ptr) = tile.alloc(size);
            unsafe { std::ptr::write_bytes(ptr, 0xAB, size); }
            allocs.push((off, size));
        }
        
        // 释放 (触发重用列表)
        for &(off, size) in &allocs {
            tile.free(off, size);
        }
        
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tile_times.push(elapsed);
        let change = if round > 0 { 
            let diff = ((elapsed - tile_times[0]) / tile_times[0] * 100.0) as i32;
            if diff < 0 { format!("({:+}% ⚡)", diff) }
            else { format!("({:+}%)", diff) }
        } else { String::new() };
        println!("   轮 {:2}: {:>6.3} ms {}  [重用: {}]", round + 1, elapsed, change, tile.reuse_count);
    }
    
    // ==================== Fate + 四元 ====================
    println!();
    println!("🔹 Fate + 四元 (学习+重用):");
    let mut fate = FateQuattro::new();
    let mut fate_times = Vec::new();
    let mut fate_allocs: Vec<(usize, usize, usize)> = Vec::with_capacity(OPS_PER_ROUND);
    
    for round in 0..ROUNDS {
        fate_allocs.clear();
        let start = Instant::now();
        
        // 分配
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            let (pool, off, ptr) = fate.alloc(size);
            unsafe { std::ptr::write_bytes(ptr, 0xAB, size); }
            fate_allocs.push((pool, off, size));
        }
        
        // 释放
        for &(pool, off, size) in &fate_allocs {
            fate.free(pool, off, size);
        }
        
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        fate_times.push(elapsed);
        let change = if round > 0 { 
            let diff = ((elapsed - fate_times[0]) / fate_times[0] * 100.0) as i32;
            if diff < 0 { format!("({:+}% ⚡)", diff) }
            else { format!("({:+}%)", diff) }
        } else { String::new() };
        println!("   轮 {:2}: {:>6.3} ms {}  [重用: {}, 缓存命中: {}]", 
                 round + 1, elapsed, change, fate.reuse_count, fate.cache_hits);
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
    println!("  系统原生:   {:>6.3} ms → {:>6.3} ms  ({:+.1}%)", 
             malloc_first, malloc_last, (malloc_last - malloc_first) / malloc_first * 100.0);
    println!("  TileQuattro: {:>6.3} ms → {:>6.3} ms  ({:+.1}%)", 
             tile_first, tile_last, (tile_last - tile_first) / tile_first * 100.0);
    println!("  Fate+四元:  {:>6.3} ms → {:>6.3} ms  ({:+.1}%)", 
             fate_first, fate_last, (fate_last - fate_first) / fate_first * 100.0);
    
    println!();
    println!("末轮速度对比:");
    println!("  TileQuattro vs 系统: {:.2}x {}", 
             malloc_last / tile_last,
             if tile_last < malloc_last { "⚡ 更快" } else { "" });
    println!("  Fate+四元 vs 系统:  {:.2}x {}", 
             malloc_last / fate_last,
             if fate_last < malloc_last { "⚡ 更快" } else { "" });
    
    println!();
    println!("学习效果:");
    println!("  TileQuattro 总重用: {}", tile.reuse_count);
    println!("  Fate 总重用: {}", fate.reuse_count);
    println!("  Fate 缓存命中: {}", fate.cache_hits);
    println!("  Fate 学习规则: {} 条", fate.size_to_pool.len());
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
}
