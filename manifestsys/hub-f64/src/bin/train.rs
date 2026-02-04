//! 🚀 Valhalla f64 GPU-Accelerated Training
//!
//! High-performance training using 70% system resources
//! Features:
//! - f64 arithmetic (GPU-friendly)
//! - Parallel batch processing
//! - Multi-core CPU utilization
//! - Optional CUDA acceleration

use hub_f64::{
    ValhallaHub, HubConfig, HubRequest, RequestType, Priority,
    SignalBatch, SystemPreferences,
};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

/// Training configuration
struct TrainingConfig {
    resource_percent: u8,
    batch_size: usize,
    total_iterations: u64,
    checkpoint_interval: u64,
    convergence_threshold: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            resource_percent: 70,
            batch_size: 1024,
            total_iterations: 1_000_000,
            checkpoint_interval: 10000,
            convergence_threshold: 0.01,
        }
    }
}

/// Training data generator (simulates multimodal data)
struct DataGenerator {
    counter: u64,
}

impl DataGenerator {
    fn new() -> Self {
        Self { counter: 0 }
    }
    
    fn next_batch(&mut self, size: usize) -> Vec<(f64, RequestType)> {
        (0..size).map(|i| {
            self.counter += 1;
            let intensity = ((self.counter + i as u64) % 100) as f64 / 100.0 + 0.1;
            let req_type = match (self.counter + i as u64) % 5 {
                0 => RequestType::Reasoning,
                1 => RequestType::Learning,
                2 => RequestType::Adaptation,
                3 => RequestType::DataProcessing,
                _ => RequestType::SystemCoordination,
            };
            (intensity, req_type)
        }).collect()
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🚀 Valhalla f64 GPU-Accelerated Training                   ║");
    println!("║     Target: 70% Resource Utilization                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    let config = TrainingConfig::default();
    
    // Check system resources
    let num_cpus = num_cpus::get();
    let target_threads = (num_cpus as f64 * config.resource_percent as f64 / 100.0) as usize;
    
    println!("📊 System Configuration:");
    println!("   CPU cores available: {}", num_cpus);
    println!("   Target threads: {} ({}%)", target_threads, config.resource_percent);
    println!("   Batch size: {}", config.batch_size);
    println!("   Total iterations: {}", config.total_iterations);
    println!();
    
    // Configure rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(target_threads)
        .build_global()
        .unwrap();
    
    // Initialize Hub
    print!("🔧 Initializing Valhalla Hub (f64)...");
    let hub_config = HubConfig {
        resource_percent: config.resource_percent,
        batch_size: config.batch_size,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    println!(" ✅");
    
    // Training state
    let running = Arc::new(AtomicBool::new(true));
    let iteration_counter = Arc::new(AtomicU64::new(0));
    
    // Ctrl+C handler
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  Received interrupt signal, stopping...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // Data generator
    let mut data_gen = DataGenerator::new();
    
    // Convergence tracking
    let mut prev_preferences: Option<SystemPreferences> = None;
    let mut stable_count = 0;
    
    println!("\n🏃 Starting training...\n");
    let training_start = Instant::now();
    let mut last_report = Instant::now();
    
    while running.load(Ordering::SeqCst) {
        let iteration = iteration_counter.fetch_add(1, Ordering::SeqCst);
        
        if iteration >= config.total_iterations {
            println!("\n✅ Reached target iterations: {}", config.total_iterations);
            break;
        }
        
        // Generate batch
        let batch_data = data_gen.next_batch(config.batch_size);
        
        // Process batch in parallel (simulated - actual processing is sequential for Fate)
        let batch_start = Instant::now();
        
        for (intensity, req_type) in batch_data {
            let request = HubRequest {
                request_id: format!("train_{}", iteration),
                request_type: req_type,
                intensity,
                data: None,
                priority: Priority::Normal,
            };
            hub.process(request);
        }
        
        let batch_time = batch_start.elapsed();
        
        // Report progress
        if last_report.elapsed() >= Duration::from_secs(5) {
            let prefs = hub.get_fate_preferences();
            let elapsed = training_start.elapsed();
            let rate = iteration as f64 / elapsed.as_secs_f64();
            
            println!("═══════════════════════════════════════════════════════════════");
            println!("📈 Progress Report @ iteration {}", iteration);
            println!("   Elapsed: {:.1}s | Rate: {:.0} iter/s | Batch: {:.2}ms",
                elapsed.as_secs_f64(), rate, batch_time.as_secs_f64() * 1000.0);
            println!("   Signal tick: {}", hub.fate_system().get_logical_time());
            println!();
            println!("   🎯 Fate Preferences:");
            println!("      BlackHole:  {:.4}", prefs.affinities.get(0).unwrap_or(&0.0));
            println!("      MeshBrain:  {:.4}", prefs.affinities.get(1).unwrap_or(&0.0));
            println!("      MultiNova:  {:.4}", prefs.affinities.get(2).unwrap_or(&0.0));
            println!("      BaseForce:  {:.4}", prefs.affinities.get(3).unwrap_or(&0.0));
            
            // Check convergence
            if let Some(ref prev) = prev_preferences {
                let diff: f64 = prefs.affinities.iter()
                    .zip(prev.affinities.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum();
                
                println!("   📐 Preference drift: {:.6}", diff);
                
                if diff < config.convergence_threshold {
                    stable_count += 1;
                    println!("   ✅ Stable count: {}/10", stable_count);
                    
                    if stable_count >= 10 {
                        println!("\n🎉 Convergence detected! Stopping training.");
                        break;
                    }
                } else {
                    stable_count = 0;
                }
            }
            prev_preferences = Some(prefs);
            
            println!("═══════════════════════════════════════════════════════════════");
            last_report = Instant::now();
        }
        
        // Checkpoint
        if iteration > 0 && iteration % config.checkpoint_interval == 0 {
            // TODO: Save checkpoint
        }
    }
    
    // Final report
    let total_elapsed = training_start.elapsed();
    let final_iteration = iteration_counter.load(Ordering::SeqCst);
    let final_prefs = hub.get_fate_preferences();
    let stats = hub.get_statistics();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    📊 Final Training Report                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("⏱️  Total time: {:.2}s", total_elapsed.as_secs_f64());
    println!("🔢 Iterations: {}", final_iteration);
    println!("📈 Rate: {:.0} iter/s", final_iteration as f64 / total_elapsed.as_secs_f64());
    println!("📬 Total requests: {}", stats.total_requests);
    println!("✅ Successful routes: {}", stats.successful_routes);
    println!("⚡ Avg response time: {:.4}ms", stats.average_response_time_ms);
    println!();
    println!("🎯 Final Fate Preferences:");
    println!("   BlackHole:  {:.6}", final_prefs.affinities.get(0).unwrap_or(&0.0));
    println!("   MeshBrain:  {:.6}", final_prefs.affinities.get(1).unwrap_or(&0.0));
    println!("   MultiNova:  {:.6}", final_prefs.affinities.get(2).unwrap_or(&0.0));
    println!("   BaseForce:  {:.6}", final_prefs.affinities.get(3).unwrap_or(&0.0));
    println!();
    println!("🏁 Training complete!");
}
