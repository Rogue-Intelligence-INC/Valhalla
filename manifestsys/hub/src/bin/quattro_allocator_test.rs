//! 四元内存分配器对比测试
//! 
//! 测试 BlackHole、MeshBrain、MultiNova、BaseForce 各自作为内存分配器的性能

use std::time::Instant;
use std::alloc::{alloc, dealloc, Layout};

const ITERATIONS: usize = 10000;
const SMALL_SIZE: usize = 64;      // 64 bytes
const MEDIUM_SIZE: usize = 1024;   // 1 KB
const LARGE_SIZE: usize = 65536;   // 64 KB

// ============================================================
// BlackHole 分配器 - 高密度大块存储
// ============================================================
struct BlackHoleAllocator {
    pool: Vec<u8>,
    offset: usize,
    total_size: usize,
    alloc_count: usize,
}

impl BlackHoleAllocator {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            offset: 0,
            total_size: size,
            alloc_count: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        // 8字节对齐
        let aligned_size = (size + 7) & !7;
        if self.offset + aligned_size > self.total_size {
            return None;
        }
        let ptr = unsafe { self.pool.as_mut_ptr().add(self.offset) };
        self.offset += aligned_size;
        self.alloc_count += 1;
        Some(ptr)
    }
    
    fn reset(&mut self) {
        self.offset = 0;
        self.alloc_count = 0;
    }
    
    fn usage(&self) -> f64 {
        self.offset as f64 / self.total_size as f64 * 100.0
    }
}

// ============================================================
// MeshBrain 分配器 - 智能符号存储 (带索引)
// ============================================================
struct MeshBrainAllocator {
    pool: Vec<u8>,
    index: Vec<(usize, usize)>,  // (offset, size)
    offset: usize,
    total_size: usize,
    alloc_count: usize,
}

impl MeshBrainAllocator {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            index: Vec::with_capacity(1024),
            offset: 0,
            total_size: size,
            alloc_count: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let aligned_size = (size + 7) & !7;
        if self.offset + aligned_size > self.total_size {
            return None;
        }
        let ptr = unsafe { self.pool.as_mut_ptr().add(self.offset) };
        self.index.push((self.offset, aligned_size));
        self.offset += aligned_size;
        self.alloc_count += 1;
        Some(ptr)
    }
    
    fn reset(&mut self) {
        self.offset = 0;
        self.alloc_count = 0;
        self.index.clear();
    }
    
    fn usage(&self) -> f64 {
        self.offset as f64 / self.total_size as f64 * 100.0
    }
}

// ============================================================
// MultiNova 分配器 - 并行分区存储
// ============================================================
struct MultiNovaAllocator {
    partitions: Vec<Vec<u8>>,
    offsets: Vec<usize>,
    partition_size: usize,
    current_partition: usize,
    alloc_count: usize,
}

impl MultiNovaAllocator {
    fn new(total_size: usize, num_partitions: usize) -> Self {
        let partition_size = total_size / num_partitions;
        let mut partitions = Vec::with_capacity(num_partitions);
        let mut offsets = Vec::with_capacity(num_partitions);
        for _ in 0..num_partitions {
            partitions.push(vec![0u8; partition_size]);
            offsets.push(0);
        }
        Self {
            partitions,
            offsets,
            partition_size,
            current_partition: 0,
            alloc_count: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let aligned_size = (size + 7) & !7;
        
        // 尝试当前分区
        for i in 0..self.partitions.len() {
            let idx = (self.current_partition + i) % self.partitions.len();
            if self.offsets[idx] + aligned_size <= self.partition_size {
                let ptr = unsafe { self.partitions[idx].as_mut_ptr().add(self.offsets[idx]) };
                self.offsets[idx] += aligned_size;
                self.current_partition = (idx + 1) % self.partitions.len();
                self.alloc_count += 1;
                return Some(ptr);
            }
        }
        None
    }
    
    fn reset(&mut self) {
        for offset in &mut self.offsets {
            *offset = 0;
        }
        self.current_partition = 0;
        self.alloc_count = 0;
    }
    
    fn usage(&self) -> f64 {
        let total_used: usize = self.offsets.iter().sum();
        let total_size = self.partition_size * self.partitions.len();
        total_used as f64 / total_size as f64 * 100.0
    }
}

// ============================================================
// BaseForce 分配器 - 小块碎片优化
// ============================================================
struct BaseForceAllocator {
    pool: Vec<u8>,
    free_list: Vec<(usize, usize)>,  // (offset, size)
    offset: usize,
    total_size: usize,
    alloc_count: usize,
    reuse_count: usize,
}

impl BaseForceAllocator {
    fn new(size: usize) -> Self {
        Self {
            pool: vec![0u8; size],
            free_list: Vec::with_capacity(256),
            offset: 0,
            total_size: size,
            alloc_count: 0,
            reuse_count: 0,
        }
    }
    
    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let aligned_size = (size + 7) & !7;
        
        // 先检查空闲列表
        for i in 0..self.free_list.len() {
            if self.free_list[i].1 >= aligned_size {
                let (off, _) = self.free_list.remove(i);
                self.alloc_count += 1;
                self.reuse_count += 1;
                return Some(unsafe { self.pool.as_mut_ptr().add(off) });
            }
        }
        
        // 从池末尾分配
        if self.offset + aligned_size > self.total_size {
            return None;
        }
        let ptr = unsafe { self.pool.as_mut_ptr().add(self.offset) };
        self.offset += aligned_size;
        self.alloc_count += 1;
        Some(ptr)
    }
    
    fn free(&mut self, offset: usize, size: usize) {
        self.free_list.push((offset, size));
    }
    
    fn reset(&mut self) {
        self.offset = 0;
        self.alloc_count = 0;
        self.reuse_count = 0;
        self.free_list.clear();
    }
    
    fn usage(&self) -> f64 {
        self.offset as f64 / self.total_size as f64 * 100.0
    }
}

// ============================================================
// 标准 malloc 测试
// ============================================================
fn test_stdlib_malloc(size: usize, iterations: usize) -> (f64, usize) {
    let start = Instant::now();
    let layout = Layout::from_size_align(size, 8).unwrap();
    let mut total_alloc = 0usize;
    
    for _ in 0..iterations {
        unsafe {
            let ptr = alloc(layout);
            if !ptr.is_null() {
                total_alloc += size;
                dealloc(ptr, layout);
            }
        }
    }
    
    (start.elapsed().as_secs_f64() * 1000.0, total_alloc)
}

// ============================================================
// 主测试
// ============================================================
fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📊 四元内存分配器对比测试");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("测试参数:");
    println!("  迭代次数: {}", ITERATIONS);
    println!("  小块: {} bytes", SMALL_SIZE);
    println!("  中块: {} bytes", MEDIUM_SIZE);
    println!("  大块: {} bytes", LARGE_SIZE);
    println!();
    
    let pool_size = 16 * 1024 * 1024;  // 16MB
    
    // ==================== 小块分配测试 ====================
    println!("═══════════════════════════════════════════════════════════════════");
    println!("🔹 小块分配测试 ({} bytes × {} 次)", SMALL_SIZE, ITERATIONS);
    println!("═══════════════════════════════════════════════════════════════════");
    
    // stdlib
    let (malloc_time, _) = test_stdlib_malloc(SMALL_SIZE, ITERATIONS);
    println!("  stdlib malloc:   {:>8.3} ms", malloc_time);
    
    // BlackHole
    let mut blackhole = BlackHoleAllocator::new(pool_size);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        blackhole.alloc(SMALL_SIZE);
    }
    let bh_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  BlackHole:       {:>8.3} ms  (usage: {:.1}%)", bh_time, blackhole.usage());
    blackhole.reset();
    
    // MeshBrain
    let mut meshbrain = MeshBrainAllocator::new(pool_size);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        meshbrain.alloc(SMALL_SIZE);
    }
    let mb_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  MeshBrain:       {:>8.3} ms  (usage: {:.1}%)", mb_time, meshbrain.usage());
    meshbrain.reset();
    
    // MultiNova (4分区)
    let mut multinova = MultiNovaAllocator::new(pool_size, 4);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        multinova.alloc(SMALL_SIZE);
    }
    let mn_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  MultiNova:       {:>8.3} ms  (usage: {:.1}%)", mn_time, multinova.usage());
    multinova.reset();
    
    // BaseForce
    let mut baseforce = BaseForceAllocator::new(pool_size);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        baseforce.alloc(SMALL_SIZE);
    }
    let bf_time = start.elapsed().as_secs_f64() * 1000.0;
    println!("  BaseForce:       {:>8.3} ms  (usage: {:.1}%)", bf_time, baseforce.usage());
    baseforce.reset();
    
    // ==================== 中块分配测试 ====================
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("🔹 中块分配测试 ({} bytes × {} 次)", MEDIUM_SIZE, ITERATIONS);
    println!("═══════════════════════════════════════════════════════════════════");
    
    let (malloc_time, _) = test_stdlib_malloc(MEDIUM_SIZE, ITERATIONS);
    println!("  stdlib malloc:   {:>8.3} ms", malloc_time);
    
    let start = Instant::now();
    for _ in 0..ITERATIONS { blackhole.alloc(MEDIUM_SIZE); }
    println!("  BlackHole:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, blackhole.usage());
    blackhole.reset();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS { meshbrain.alloc(MEDIUM_SIZE); }
    println!("  MeshBrain:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, meshbrain.usage());
    meshbrain.reset();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS { multinova.alloc(MEDIUM_SIZE); }
    println!("  MultiNova:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, multinova.usage());
    multinova.reset();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS { baseforce.alloc(MEDIUM_SIZE); }
    println!("  BaseForce:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, baseforce.usage());
    baseforce.reset();
    
    // ==================== 大块分配测试 ====================
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("🔹 大块分配测试 ({} bytes × {} 次)", LARGE_SIZE, ITERATIONS / 10);
    println!("═══════════════════════════════════════════════════════════════════");
    
    let (malloc_time, _) = test_stdlib_malloc(LARGE_SIZE, ITERATIONS / 10);
    println!("  stdlib malloc:   {:>8.3} ms", malloc_time);
    
    let start = Instant::now();
    for _ in 0..ITERATIONS/10 { blackhole.alloc(LARGE_SIZE); }
    println!("  BlackHole:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, blackhole.usage());
    blackhole.reset();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS/10 { meshbrain.alloc(LARGE_SIZE); }
    println!("  MeshBrain:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, meshbrain.usage());
    meshbrain.reset();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS/10 { multinova.alloc(LARGE_SIZE); }
    println!("  MultiNova:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, multinova.usage());
    multinova.reset();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS/10 { baseforce.alloc(LARGE_SIZE); }
    println!("  BaseForce:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, baseforce.usage());
    baseforce.reset();
    
    // ==================== 混合分配测试 ====================
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("🔹 混合分配测试 (随机大小 × {} 次)", ITERATIONS);
    println!("═══════════════════════════════════════════════════════════════════");
    
    let sizes = [16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192];
    
    // stdlib
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let size = sizes[i % sizes.len()];
        let layout = Layout::from_size_align(size, 8).unwrap();
        unsafe {
            let ptr = alloc(layout);
            if !ptr.is_null() { dealloc(ptr, layout); }
        }
    }
    println!("  stdlib malloc:   {:>8.3} ms", start.elapsed().as_secs_f64() * 1000.0);
    
    let start = Instant::now();
    for i in 0..ITERATIONS { blackhole.alloc(sizes[i % sizes.len()]); }
    println!("  BlackHole:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, blackhole.usage());
    blackhole.reset();
    
    let start = Instant::now();
    for i in 0..ITERATIONS { meshbrain.alloc(sizes[i % sizes.len()]); }
    println!("  MeshBrain:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, meshbrain.usage());
    meshbrain.reset();
    
    let start = Instant::now();
    for i in 0..ITERATIONS { multinova.alloc(sizes[i % sizes.len()]); }
    println!("  MultiNova:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, multinova.usage());
    multinova.reset();
    
    let start = Instant::now();
    for i in 0..ITERATIONS { baseforce.alloc(sizes[i % sizes.len()]); }
    println!("  BaseForce:       {:>8.3} ms  (usage: {:.1}%)", start.elapsed().as_secs_f64() * 1000.0, baseforce.usage());
    
    // ==================== 结论 ====================
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📈 分配器特点总结");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  BlackHole  : 最简单快速，适合大块连续分配");
    println!("  MeshBrain  : 带索引，适合符号表/字符串池");
    println!("  MultiNova  : 多分区并行，适合并发场景");
    println!("  BaseForce  : 碎片重用，适合频繁分配释放");
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
}
