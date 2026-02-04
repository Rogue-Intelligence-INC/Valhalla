//! TileQuattro 对比测试
//! 1. TileQuattro 单独 (无 Fate)
//! 2. TileQuattro + Fate (连接)
//! 3. Fate + 四元 (完全集成)

use std::time::Instant;

const ROUNDS: usize = 10;
const OPS_PER_ROUND: usize = 5000;

// ============================================================
// TileQuattro 单独 (无 Fate) - 只有重用
// ============================================================
struct TileQuattroAlone {
    pool: Vec<u8>,
    offset: usize,
    reuse_list: Vec<(usize, usize)>,
    alloc_count: usize,
    reuse_count: usize,
}

impl TileQuattroAlone {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            offset: 0,
            reuse_list: Vec::with_capacity(4096),
            alloc_count: 0,
            reuse_count: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> (usize, *mut u8) {
        let aligned = (size + 7) & !7;
        self.alloc_count += 1;
        
        // 简单重用: 找第一个够大的
        for i in 0..self.reuse_list.len() {
            if self.reuse_list[i].1 >= aligned {
                let (off, _) = self.reuse_list.swap_remove(i);
                self.reuse_count += 1;
                return (off, unsafe { self.pool.as_mut_ptr().add(off) });
            }
        }
        
        let off = self.offset;
        self.offset += aligned;
        (off, unsafe { self.pool.as_mut_ptr().add(off) })
    }
    
    fn free(&mut self, offset: usize, size: usize) {
        self.reuse_list.push((offset, (size + 7) & !7));
    }
}

// ============================================================
// TileQuattro + Fate (连接版)
// ============================================================
struct TileQuattroWithFate {
    pool: Vec<u8>,
    offset: usize,
    reuse_list: Vec<(usize, usize)>,
    
    // Fate 添加的功能
    size_cache: Vec<(usize, usize)>,  // size -> best_reuse_idx 学习
    alloc_history: Vec<usize>,         // 分配历史用于预测
    
    alloc_count: usize,
    reuse_count: usize,
    cache_hits: usize,
    predict_hits: usize,
}

impl TileQuattroWithFate {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            offset: 0,
            reuse_list: Vec::with_capacity(4096),
            size_cache: Vec::with_capacity(256),
            alloc_history: Vec::with_capacity(1024),
            alloc_count: 0,
            reuse_count: 0,
            cache_hits: 0,
            predict_hits: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> (usize, *mut u8) {
        let aligned = (size + 7) & !7;
        self.alloc_count += 1;
        
        // Fate 学习: 记录历史
        self.alloc_history.push(aligned);
        if self.alloc_history.len() > 1024 {
            self.alloc_history.remove(0);
        }
        
        // Fate 智能重用: 最佳匹配 (减少碎片)
        let mut best_idx = None;
        let mut best_waste = usize::MAX;
        
        // 先检查缓存
        for &(s, idx) in &self.size_cache {
            if s == aligned && idx < self.reuse_list.len() {
                let (_, reuse_size) = self.reuse_list[idx];
                if reuse_size >= aligned {
                    self.cache_hits += 1;
                    let (off, _) = self.reuse_list.swap_remove(idx);
                    self.reuse_count += 1;
                    return (off, unsafe { self.pool.as_mut_ptr().add(off) });
                }
            }
        }
        
        // 最佳匹配搜索
        for (i, &(_, s)) in self.reuse_list.iter().enumerate() {
            if s >= aligned {
                let waste = s - aligned;
                if waste < best_waste {
                    best_waste = waste;
                    best_idx = Some(i);
                    if waste == 0 { break; }  // 完美匹配
                }
            }
        }
        
        if let Some(i) = best_idx {
            // Fate 学习: 缓存这个大小的最佳索引
            if self.size_cache.len() < 256 {
                self.size_cache.push((aligned, i));
            }
            
            let (off, _) = self.reuse_list.swap_remove(i);
            self.reuse_count += 1;
            return (off, unsafe { self.pool.as_mut_ptr().add(off) });
        }
        
        // Fate 预测: 预分配常见大小
        if self.should_preallocate(aligned) {
            self.predict_hits += 1;
        }
        
        let off = self.offset;
        self.offset += aligned;
        (off, unsafe { self.pool.as_mut_ptr().add(off) })
    }
    
    fn should_preallocate(&self, size: usize) -> bool {
        // 统计历史中这个大小的频率
        let count = self.alloc_history.iter().filter(|&&s| s == size).count();
        count > self.alloc_history.len() / 10  // 超过10%就预测
    }
    
    fn free(&mut self, offset: usize, size: usize) {
        self.reuse_list.push((offset, (size + 7) & !7));
    }
}

// ============================================================
// Fate + 四元 (完全集成)
// ============================================================
struct FateQuattroFull {
    pools: [Vec<u8>; 4],
    offsets: [usize; 4],
    reuse_lists: [Vec<(usize, usize)>; 4],
    
    // Fate 完全控制
    size_to_pool: Vec<(usize, usize)>,
    pool_load: [usize; 4],  // 负载均衡
    
    alloc_count: usize,
    reuse_count: usize,
    cache_hits: usize,
    balance_ops: usize,
}

impl FateQuattroFull {
    fn new() -> Self {
        Self {
            pools: [
                vec![0u8; 4 * 1024 * 1024],
                vec![0u8; 2 * 1024 * 1024],
                vec![0u8; 4 * 1024 * 1024],
                vec![0u8; 2 * 1024 * 1024],
            ],
            offsets: [0; 4],
            reuse_lists: [
                Vec::with_capacity(1024),
                Vec::with_capacity(1024),
                Vec::with_capacity(1024),
                Vec::with_capacity(2048),
            ],
            size_to_pool: Vec::with_capacity(256),
            pool_load: [0; 4],
            alloc_count: 0,
            reuse_count: 0,
            cache_hits: 0,
            balance_ops: 0,
        }
    }
    
    fn fate_decide(&mut self, size: usize) -> usize {
        // 缓存检查
        for &(s, p) in &self.size_to_pool {
            if s == size {
                self.cache_hits += 1;
                return p;
            }
        }
        
        // Fate 智能决策 + 负载均衡
        let base_pool = if size > 16384 { 0 }
                        else if size > 4096 { 1 }
                        else if size > 512 { 2 }
                        else { 3 };
        
        // 检查负载，可能切换到其他池
        let pool = if self.pool_load[base_pool] > 1000 {
            self.balance_ops += 1;
            // 找负载最低的合适池
            let mut best = base_pool;
            for i in 0..4 {
                if self.pool_load[i] < self.pool_load[best] {
                    best = i;
                }
            }
            best
        } else {
            base_pool
        };
        
        // 学习
        if self.size_to_pool.len() < 256 {
            self.size_to_pool.push((size, pool));
        }
        
        self.pool_load[pool] += 1;
        pool
    }
    
    fn alloc(&mut self, size: usize) -> (usize, usize, *mut u8) {
        let aligned = (size + 7) & !7;
        let pool_idx = self.fate_decide(size);
        self.alloc_count += 1;
        
        // 最佳匹配重用
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
            let (off, _) = reuse.swap_remove(i);
            self.reuse_count += 1;
            return (pool_idx, off, unsafe { self.pools[pool_idx].as_mut_ptr().add(off) });
        }
        
        let off = self.offsets[pool_idx];
        self.offsets[pool_idx] += aligned;
        (pool_idx, off, unsafe { self.pools[pool_idx].as_mut_ptr().add(off) })
    }
    
    fn free(&mut self, pool_idx: usize, offset: usize, size: usize) {
        self.reuse_lists[pool_idx].push((offset, (size + 7) & !7));
        self.pool_load[pool_idx] = self.pool_load[pool_idx].saturating_sub(1);
    }
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📊 TileQuattro + Fate 对比测试");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("1. TileQuattro 单独 (无 Fate)");
    println!("2. TileQuattro + Fate (连接)");
    println!("3. Fate + 四元 (完全集成)");
    println!();
    
    let sizes = [64, 128, 256, 512, 1024, 2048, 4096];
    
    // ==================== TileQuattro 单独 ====================
    println!("🔹 TileQuattro 单独 (无 Fate):");
    let mut tile_alone = TileQuattroAlone::new(64 * 1024 * 1024);
    let mut tile_alone_times = Vec::new();
    let mut allocs1: Vec<(usize, usize)> = Vec::with_capacity(OPS_PER_ROUND);
    
    for round in 0..ROUNDS {
        allocs1.clear();
        let start = Instant::now();
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            let (off, ptr) = tile_alone.alloc(size);
            unsafe { std::ptr::write_bytes(ptr, 0xAB, 1); }
            allocs1.push((off, size));
        }
        for &(off, size) in &allocs1 { tile_alone.free(off, size); }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tile_alone_times.push(elapsed);
        let change = if round > 0 { 
            let diff = ((elapsed - tile_alone_times[0]) / tile_alone_times[0] * 100.0) as i32;
            if diff < 0 { format!("({:+}% ⚡)", diff) } else { format!("({:+}%)", diff) }
        } else { String::new() };
        println!("   轮 {:2}: {:>6.3} ms {}  [重用: {}]", 
                 round + 1, elapsed, change, tile_alone.reuse_count);
    }
    
    // ==================== TileQuattro + Fate ====================
    println!();
    println!("🔹 TileQuattro + Fate (连接):");
    let mut tile_fate = TileQuattroWithFate::new(64 * 1024 * 1024);
    let mut tile_fate_times = Vec::new();
    let mut allocs2: Vec<(usize, usize)> = Vec::with_capacity(OPS_PER_ROUND);
    
    for round in 0..ROUNDS {
        allocs2.clear();
        let start = Instant::now();
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            let (off, ptr) = tile_fate.alloc(size);
            unsafe { std::ptr::write_bytes(ptr, 0xAB, 1); }
            allocs2.push((off, size));
        }
        for &(off, size) in &allocs2 { tile_fate.free(off, size); }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tile_fate_times.push(elapsed);
        let change = if round > 0 { 
            let diff = ((elapsed - tile_fate_times[0]) / tile_fate_times[0] * 100.0) as i32;
            if diff < 0 { format!("({:+}% ⚡)", diff) } else { format!("({:+}%)", diff) }
        } else { String::new() };
        println!("   轮 {:2}: {:>6.3} ms {}  [重用: {}, 缓存: {}]", 
                 round + 1, elapsed, change, tile_fate.reuse_count, tile_fate.cache_hits);
    }
    
    // ==================== Fate + 四元 ====================
    println!();
    println!("🔹 Fate + 四元 (完全集成):");
    let mut fate_quattro = FateQuattroFull::new();
    let mut fate_times = Vec::new();
    let mut allocs3: Vec<(usize, usize, usize)> = Vec::with_capacity(OPS_PER_ROUND);
    
    for round in 0..ROUNDS {
        allocs3.clear();
        let start = Instant::now();
        for i in 0..OPS_PER_ROUND {
            let size = sizes[i % sizes.len()];
            let (pool, off, ptr) = fate_quattro.alloc(size);
            unsafe { std::ptr::write_bytes(ptr, 0xAB, 1); }
            allocs3.push((pool, off, size));
        }
        for &(pool, off, size) in &allocs3 { fate_quattro.free(pool, off, size); }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        fate_times.push(elapsed);
        let change = if round > 0 { 
            let diff = ((elapsed - fate_times[0]) / fate_times[0] * 100.0) as i32;
            if diff < 0 { format!("({:+}% ⚡)", diff) } else { format!("({:+}%)", diff) }
        } else { String::new() };
        println!("   轮 {:2}: {:>6.3} ms {}  [重用: {}, 缓存: {}, 均衡: {}]", 
                 round + 1, elapsed, change, 
                 fate_quattro.reuse_count, fate_quattro.cache_hits, fate_quattro.balance_ops);
    }
    
    // ==================== 结果汇总 ====================
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📈 结果汇总");
    println!("═══════════════════════════════════════════════════════════════════");
    
    println!();
    println!("首轮 vs 末轮:");
    println!("  TileQuattro单独:  {:>6.3} ms → {:>6.3} ms  ({:+.1}%)", 
             tile_alone_times[0], tile_alone_times[ROUNDS-1],
             (tile_alone_times[ROUNDS-1] - tile_alone_times[0]) / tile_alone_times[0] * 100.0);
    println!("  TileQuattro+Fate: {:>6.3} ms → {:>6.3} ms  ({:+.1}%)", 
             tile_fate_times[0], tile_fate_times[ROUNDS-1],
             (tile_fate_times[ROUNDS-1] - tile_fate_times[0]) / tile_fate_times[0] * 100.0);
    println!("  Fate+四元:        {:>6.3} ms → {:>6.3} ms  ({:+.1}%)", 
             fate_times[0], fate_times[ROUNDS-1],
             (fate_times[ROUNDS-1] - fate_times[0]) / fate_times[0] * 100.0);
    
    println!();
    println!("末轮对比:");
    let base = tile_alone_times[ROUNDS-1];
    println!("  TileQuattro单独:  {:.3} ms (基准)", base);
    println!("  TileQuattro+Fate: {:.3} ms ({:.1}x {})", 
             tile_fate_times[ROUNDS-1],
             base / tile_fate_times[ROUNDS-1],
             if tile_fate_times[ROUNDS-1] < base { "⚡更快" } else { "" });
    println!("  Fate+四元:        {:.3} ms ({:.1}x {})", 
             fate_times[ROUNDS-1],
             base / fate_times[ROUNDS-1],
             if fate_times[ROUNDS-1] < base { "⚡更快" } else { "" });
    
    println!();
    println!("Fate 增益:");
    println!("  TileQuattro+Fate 缓存命中: {}", tile_fate.cache_hits);
    println!("  Fate+四元 缓存命中: {}", fate_quattro.cache_hits);
    println!("  Fate+四元 负载均衡: {} 次", fate_quattro.balance_ops);
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
}
