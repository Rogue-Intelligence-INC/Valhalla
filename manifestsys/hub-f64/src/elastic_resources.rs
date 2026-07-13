//! Elastic resource management — CUDA-aware parallel + ingress budget.
//!
//! Built-in (no external orchestrator):
//! - Merge `IngressBudget::incubation()` with env overrides
//! - Scale parallel workers from resource %, CUDA readiness, and LM safety
//! - Probe CUDA link + Candle device availability
//! - Process-level CUDA infer lock for safe multi-thread LM access

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::signal_ingress::IngressBudget;

/// Snapshot of elastic runtime limits for one suite / bench phase.
#[derive(Debug, Clone)]
pub struct ElasticResourcePlan {
    pub budget: IngressBudget,
    pub parallel_workers: usize,
    pub cuda_ready: bool,
    pub cuda_device: usize,
    pub use_cpu: bool,
    pub lm_cache_max: usize,
}

impl ElasticResourcePlan {
    /// Build plan from env + workload hint (`n_items` = outer parallel unit count).
    pub fn for_workload(n_items: usize) -> Self {
        Self::ensure_cuda_runtime_env();
        let budget = IngressBudget::incubation_with_env();
        let use_cpu = env_truthy("VALHALLA_USE_CPU");
        let cuda_device = cuda_device_index();
        let cuda_ready = !use_cpu && cuda_runtime_ready() && candle_cuda_available();
        let lm_cache_max = env_usize("VALHALLA_LM_CACHE_MAX", 1).clamp(1, 4);
        let parallel_workers =
            elastic_parallel_workers(n_items, budget.resource_percent, cuda_ready);
        Self {
            budget,
            parallel_workers,
            cuda_ready,
            cuda_device,
            use_cpu,
            lm_cache_max,
        }
    }

    /// Apply process-global CUDA / resource env defaults (idempotent-friendly).
    pub fn ensure_cuda_runtime_env() {
        set_env_default("VALHALLA_USE_CPU", "0");
        set_env_default("VALHALLA_LM_CACHE_MAX", "1");
        set_env_default("VALHALLA_INGRESS_RESOURCE_PERCENT", "50");
        set_env_default("VALHALLA_INGRESS_MAX_ITEMS", "48");
        set_env_default("VALHALLA_CUDA_DEVICE", "0");
        // Link search paths for Candle CUDA (host must have libcudart).
        if std::env::var("LIBRARY_PATH").is_err() {
            if Path::new("/opt/cuda/lib64").is_dir() {
                std::env::set_var("LIBRARY_PATH", "/opt/cuda/lib64");
            } else if Path::new("/opt/cuda/targets/x86_64-linux/lib").is_dir() {
                std::env::set_var(
                    "LIBRARY_PATH",
                    "/opt/cuda/targets/x86_64-linux/lib",
                );
            }
        }
        if std::env::var("RUSTFLAGS").is_err() && Path::new("/opt/cuda/lib64").is_dir() {
            std::env::set_var("RUSTFLAGS", "-L /opt/cuda/lib64");
        }
    }
}

impl IngressBudget {
    /// Incubation baseline merged with `VALHALLA_INGRESS_*` (elastic suite default).
    pub fn incubation_with_env() -> Self {
        let mut b = Self::incubation();
        let env = Self::from_env();
        if std::env::var("VALHALLA_INGRESS_MAX_ITEMS").is_ok() {
            b.max_items = env.max_items;
        } else {
            b.max_items = b.max_items.min(512);
        }
        if std::env::var("VALHALLA_INGRESS_MAX_SIGNALS").is_ok() {
            b.max_signals_total = env.max_signals_total;
        }
        if std::env::var("VALHALLA_INGRESS_MAX_SIGNALS_PER_ITEM").is_ok() {
            b.max_signals_per_item = env.max_signals_per_item;
        }
        if std::env::var("VALHALLA_INGRESS_SAMPLE_STRIDE").is_ok() {
            b.sample_stride = env.sample_stride;
        }
        if std::env::var("VALHALLA_INGRESS_RESOURCE_PERCENT").is_ok() {
            b.resource_percent = env.resource_percent;
        } else {
            b.resource_percent = 50;
        }
        b
    }
}

/// Parallel workers for isolated per-item benches (multihop, etc.).
///
/// CUDA + shared LM cache: default 1 worker (VRAM-safe) unless `VALHALLA_BENCH_PARALLEL`
/// is set. Structure-only / CPU can scale with resource %.
pub fn elastic_parallel_workers(n_items: usize, resource_percent: u8, cuda_ready: bool) -> usize {
    if n_items <= 1 {
        return 1;
    }
    if env_truthy("VALHALLA_BENCH_SERIAL") {
        return 1;
    }
    let explicit = std::env::var("VALHALLA_BENCH_PARALLEL")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 1);
    if let Some(n) = explicit {
        // Even with explicit parallel, CUDA LM paths hard-cap at 4 (infer lock serializes GPU).
        let hard = if cuda_ready { 4 } else { 8 };
        return n.min(n_items).clamp(1, hard);
    }
    // CUDA default: serial outer loop — LM weight cache is process-global.
    // Structure-heavy benches that only need CPU can still fan out via explicit PARALLEL.
    if cuda_ready && !env_truthy("VALHALLA_BENCH_PARALLEL_CUDA") {
        return 1;
    }
    let from_pct = ((resource_percent as usize).max(25) / 25).clamp(1, 4);
    let cap = if cuda_ready { 2 } else { 4 };
    from_pct.min(cap).min(n_items).clamp(1, 8)
}

/// True when CUDA shared libs look present (link-time readiness).
pub fn cuda_runtime_ready() -> bool {
    const CANDIDATES: &[&str] = &[
        "/opt/cuda/lib64/libcudart.so",
        "/opt/cuda/lib64/libcudart.so.13",
        "/opt/cuda/lib64/libcudart.so.12",
        "/usr/local/cuda/lib64/libcudart.so",
        "/opt/cuda/targets/x86_64-linux/lib/libcudart.so",
    ];
    CANDIDATES.iter().any(|p| Path::new(p).is_file())
}

/// Candle-reported CUDA availability (compile-time feature + runtime).
pub fn candle_cuda_available() -> bool {
    #[cfg(feature = "patch-lm")]
    {
        candle_core::utils::cuda_is_available()
    }
    #[cfg(not(feature = "patch-lm"))]
    {
        false
    }
}

/// `VALHALLA_CUDA_DEVICE` (default 0).
pub fn cuda_device_index() -> usize {
    env_usize("VALHALLA_CUDA_DEVICE", 0).min(7)
}

/// Process-global lock for Candle CUDA infer / weight load (safe multi-thread).
static CUDA_INFER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn with_cuda_infer_lock<R>(f: impl FnOnce() -> R) -> R {
    let lock = CUDA_INFER_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on" || v == "yes"
        })
        .unwrap_or(false)
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn set_env_default(key: &str, value: &str) {
    if std::env::var(key).is_err() {
        std::env::set_var(key, value);
    }
}

/// Run isolated item work with an elastic rayon pool (order-preserving collect).
pub fn par_map_items<T, R, F>(items: &[T], workers: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    use rayon::prelude::*;
    if workers <= 1 || items.len() <= 1 {
        return items.iter().map(|x| f(x)).collect();
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .unwrap_or_else(|_| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build()
                .expect("rayon pool")
        });
    pool.install(|| items.par_iter().map(|x| f(x)).collect())
}

/// Sequential map with pause checks between items (benchmark control path).
pub fn map_items_with_pause<T, R, F>(
    items: &[T],
    pause_file: Option<&Path>,
    mut f: F,
) -> (Vec<R>, bool)
where
    F: FnMut(&T) -> R,
{
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if pause_requested(pause_file) {
            return (out, true);
        }
        out.push(f(item));
    }
    (out, false)
}

fn pause_requested(pause_file: Option<&Path>) -> bool {
    let path = pause_file.map(|p| p.to_path_buf()).or_else(|| {
        std::env::var("VALHALLA_PAUSE_FILE")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(std::path::PathBuf::from)
    });
    path.as_ref().is_some_and(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workers_serial_when_forced() {
        std::env::set_var("VALHALLA_BENCH_SERIAL", "1");
        assert_eq!(elastic_parallel_workers(25, 50, true), 1);
        std::env::remove_var("VALHALLA_BENCH_SERIAL");
    }

    #[test]
    fn workers_cuda_default_serial() {
        std::env::remove_var("VALHALLA_BENCH_SERIAL");
        std::env::remove_var("VALHALLA_BENCH_PARALLEL");
        std::env::remove_var("VALHALLA_BENCH_PARALLEL_CUDA");
        assert_eq!(elastic_parallel_workers(25, 50, true), 1);
    }

    #[test]
    fn workers_cpu_scale_with_resource() {
        std::env::remove_var("VALHALLA_BENCH_SERIAL");
        std::env::remove_var("VALHALLA_BENCH_PARALLEL");
        let w = elastic_parallel_workers(25, 50, false);
        assert!((2..=4).contains(&w), "w={w}");
    }

    #[test]
    fn incubation_with_env_merges_percent() {
        std::env::set_var("VALHALLA_INGRESS_RESOURCE_PERCENT", "40");
        let b = IngressBudget::incubation_with_env();
        assert_eq!(b.resource_percent, 40);
        std::env::remove_var("VALHALLA_INGRESS_RESOURCE_PERCENT");
    }
}
