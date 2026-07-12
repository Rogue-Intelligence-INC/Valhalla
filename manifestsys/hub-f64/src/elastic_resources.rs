//! Elastic resource management — CUDA-aware parallel + ingress budget.
//!
//! Built-in (no external orchestrator):
//! - Merge `IngressBudget::incubation()` with env overrides
//! - Scale parallel workers from resource % and workload size
//! - Detect CUDA link readiness for LM / suite paths

use crate::signal_ingress::IngressBudget;

/// Snapshot of elastic runtime limits for one suite / bench phase.
#[derive(Debug, Clone)]
pub struct ElasticResourcePlan {
    pub budget: IngressBudget,
    pub parallel_workers: usize,
    pub cuda_ready: bool,
    pub use_cpu: bool,
    pub lm_cache_max: usize,
}

impl ElasticResourcePlan {
    /// Build plan from env + workload hint (`n_items` = outer parallel unit count).
    pub fn for_workload(n_items: usize) -> Self {
        let budget = IngressBudget::incubation_with_env();
        let use_cpu = env_truthy("VALHALLA_USE_CPU");
        let cuda_ready = cuda_runtime_ready() && !use_cpu;
        let lm_cache_max = env_usize("VALHALLA_LM_CACHE_MAX", 1).clamp(1, 4);
        let parallel_workers = elastic_parallel_workers(n_items, budget.resource_percent, cuda_ready);
        Self {
            budget,
            parallel_workers,
            cuda_ready,
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
        // Link search paths for Candle CUDA (host must have libcudart).
        if std::env::var("LIBRARY_PATH").is_err() {
            if std::path::Path::new("/opt/cuda/lib64").is_dir() {
                std::env::set_var("LIBRARY_PATH", "/opt/cuda/lib64");
            } else if std::path::Path::new("/opt/cuda/targets/x86_64-linux/lib").is_dir() {
                std::env::set_var(
                    "LIBRARY_PATH",
                    "/opt/cuda/targets/x86_64-linux/lib",
                );
            }
        }
    }
}

impl IngressBudget {
    /// Incubation baseline merged with `VALHALLA_INGRESS_*` (elastic suite default).
    pub fn incubation_with_env() -> Self {
        let mut b = Self::incubation();
        let env = Self::from_env();
        // Env wins when explicitly set (from_env starts from Default — detect via env presence).
        if std::env::var("VALHALLA_INGRESS_MAX_ITEMS").is_ok() {
            b.max_items = env.max_items;
        } else {
            // Soft elastic clamp for suite: keep incubation room but respect typical 48 cap.
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
            b.resource_percent = 50; // suite-safe default when unset
        }
        b
    }
}

/// Parallel workers for isolated per-item benches (multihop, etc.).
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
        return n.min(n_items).clamp(1, 8);
    }
    // Scale with resource %: 50% → ~2–3 workers; 100% → up to 4.
    let from_pct = ((resource_percent as usize).max(25) / 25).clamp(1, 4);
    let cap = if cuda_ready { 4 } else { 3 };
    from_pct.min(cap).min(n_items).clamp(1, 8)
}

/// True when CUDA shared libs look present (link-time readiness, not a device call).
pub fn cuda_runtime_ready() -> bool {
    const CANDIDATES: &[&str] = &[
        "/opt/cuda/lib64/libcudart.so",
        "/opt/cuda/lib64/libcudart.so.13",
        "/opt/cuda/lib64/libcudart.so.12",
        "/usr/local/cuda/lib64/libcudart.so",
        "/opt/cuda/targets/x86_64-linux/lib/libcudart.so",
    ];
    CANDIDATES.iter().any(|p| std::path::Path::new(p).is_file())
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
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
    fn workers_scale_with_resource() {
        std::env::remove_var("VALHALLA_BENCH_SERIAL");
        std::env::remove_var("VALHALLA_BENCH_PARALLEL");
        let w = elastic_parallel_workers(25, 50, true);
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
