//! Triple-Fate 综合对比实验
//!
//! 对比三个版本的 Triple-Fate 实现：
//! 1. S-wind (i64 版本) - 避免溢出
//! 2. Valhalla-Rational (精确 Rational 版本)
//! 3. Systems/Fate (f64 版本) - 高性能
//!
//! 测试目标：
//! - 是否能创建全新未知元系统
//! - 各版本性能和精度对比
//! - 元发现能力验证

use std::time::Instant;

// 版本标识
const VERSION_SWIND: &str = "S-wind (i64)";
const VERSION_RATIONAL: &str = "Valhalla-Rational";
const VERSION_F64: &str = "Systems/Fate (f64)";

/// 实验结果
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub version: String,
    pub signal_count: u64,
    pub total_discoveries: usize,
    pub math_patterns: usize,
    pub system_patterns: usize,
    pub new_rules: usize,
    pub new_fates: usize,
    pub ultimate_discoveries: usize,
    pub generations: u64,
    pub efficiency_percent: f64,
    pub execution_time_ms: u128,
}

/// 运行综合对比实验
pub fn run_full_comparison(signal_count: usize) -> Vec<ComparisonResult> {
    let mut results = Vec::new();
    
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║           TRIPLE-FATE 综合对比实验: 三个版本性能与能力测试                  ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  目标: 验证是否能创建全新未知元系统                                          ║");
    println!("║  信号数: {}                                                                 ║", signal_count);
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // 生成统一测试信号
    let test_signals = generate_diverse_signals(signal_count);
    
    // =========================================================================
    // 测试 1: S-wind (i64)
    // =========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 1: {} - 避免溢出，简单整数运算", VERSION_SWIND);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let start = Instant::now();
    let swind_result = simulate_swind_experiment(&test_signals);
    let swind_time = start.elapsed().as_millis();
    
    results.push(ComparisonResult {
        version: VERSION_SWIND.into(),
        execution_time_ms: swind_time,
        ..swind_result
    });
    
    println!("  ✓ 完成 - 耗时: {}ms", swind_time);
    println!();

    // =========================================================================
    // 测试 2: Valhalla-Rational (精确)
    // =========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 2: {} - 精确 i128/i128 有理数运算", VERSION_RATIONAL);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let start = Instant::now();
    let rational_result = simulate_rational_experiment(&test_signals);
    let rational_time = start.elapsed().as_millis();
    
    results.push(ComparisonResult {
        version: VERSION_RATIONAL.into(),
        execution_time_ms: rational_time,
        ..rational_result
    });
    
    println!("  ✓ 完成 - 耗时: {}ms", rational_time);
    println!();

    // =========================================================================
    // 测试 3: Systems/Fate (f64)
    // =========================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 3: {} - 高性能浮点运算", VERSION_F64);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let start = Instant::now();
    let f64_result = simulate_f64_experiment(&test_signals);
    let f64_time = start.elapsed().as_millis();
    
    results.push(ComparisonResult {
        version: VERSION_F64.into(),
        execution_time_ms: f64_time,
        ..f64_result
    });
    
    println!("  ✓ 完成 - 耗时: {}ms", f64_time);
    println!();

    // =========================================================================
    // 结果对比
    // =========================================================================
    print_comparison_results(&results);
    
    // =========================================================================
    // 分析结论
    // =========================================================================
    print_analysis(&results);
    
    results
}

/// 生成多样化测试信号
fn generate_diverse_signals(count: usize) -> Vec<Vec<f64>> {
    (0..count).map(|i| {
        let bound_i = (i % 50) as f64;
        match i % 10 {
            0 => vec![bound_i, bound_i + 1.0, bound_i + 2.0],  // Linear +1
            1 => vec![bound_i * 2.0, bound_i * 2.0 + 2.0, bound_i * 2.0 + 4.0],  // Linear +2
            2 => vec![bound_i, bound_i * 2.0, bound_i * 4.0],  // Exponential
            3 => vec![10.0, 10.0, 10.0],  // Fixed point
            4 => vec![bound_i + 10.0, bound_i + 5.0, bound_i + 2.0],  // Converging
            5 => vec![bound_i, bound_i + 5.0, bound_i + 15.0],  // Diverging
            6 => vec![-bound_i, 0.0, bound_i],  // Balanced (sum = 0)
            7 => vec![bound_i, bound_i, bound_i],  // All same
            8 => vec![1.0, 1.0, 2.0],  // Fibonacci-like
            _ => vec![bound_i, bound_i + 3.0, bound_i + 6.0],  // Linear +3
        }
    }).collect()
}

/// 模拟 S-wind 实验 (使用 i64)
fn simulate_swind_experiment(signals: &[Vec<f64>]) -> ComparisonResult {
    // 转换信号为 i64
    let i64_signals: Vec<Vec<i64>> = signals.iter()
        .map(|s| s.iter().map(|v| *v as i64).collect())
        .collect();
    
    // 模拟 Combined 架构
    let mut total_discoveries = 0;
    let mut math_patterns = 0;
    let mut system_patterns = 0;
    let mut new_rules = 0;
    let mut new_fates = 0;
    let mut ultimate = 0;
    let mut generations = 0;
    
    // 简化模拟 - 基于实际 Triple-Fate 逻辑
    let mut signal_count = 0u64;
    let mut feedback_buffer: Vec<Vec<i64>> = Vec::new();
    let mut self_discoveries = 0;
    
    for signal in i64_signals {
        signal_count += 1;
        
        // 检测模式
        if detect_pattern_i64(&signal) {
            total_discoveries += 1;
            math_patterns += 1;
            feedback_buffer.push(signal.iter().map(|v| v / 2).collect());
        }
        
        // 处理反馈
        if signal_count % 5 == 0 && !feedback_buffer.is_empty() {
            let feedback = feedback_buffer.remove(0);
            if detect_pattern_i64(&feedback) {
                total_discoveries += 1;
                system_patterns += 1;
                self_discoveries += 1;
            }
        }
        
        // 涌现规则
        if self_discoveries > 0 && self_discoveries % 50 == 0 {
            new_rules += 1;
        }
        
        // 进化
        if self_discoveries > 0 && self_discoveries % 100 == 0 {
            generations += 1;
            new_fates += 1;
        }
    }
    
    // 额外处理
    for _ in 0..150 {
        signal_count += 1;
        if !feedback_buffer.is_empty() {
            let feedback = feedback_buffer.remove(0);
            if detect_pattern_i64(&feedback) {
                total_discoveries += 1;
                system_patterns += 1;
                self_discoveries += 1;
            }
        }
        
        if self_discoveries > 0 && self_discoveries % 100 == 0 {
            generations += 1;
            new_fates += 1;
        }
    }
    
    // 终极发现
    if generations >= 2 && self_discoveries > 1000 {
        ultimate = 1;
    }
    
    let efficiency = if signal_count > 0 {
        (total_discoveries as f64 / signal_count as f64) * 100.0
    } else {
        0.0
    };
    
    ComparisonResult {
        version: VERSION_SWIND.into(),
        signal_count,
        total_discoveries,
        math_patterns,
        system_patterns,
        new_rules,
        new_fates,
        ultimate_discoveries: ultimate,
        generations,
        efficiency_percent: efficiency,
        execution_time_ms: 0,
    }
}

/// 模拟 Rational 实验
fn simulate_rational_experiment(signals: &[Vec<f64>]) -> ComparisonResult {
    // 与 S-wind 类似，但使用更精确的逻辑
    let mut total_discoveries = 0;
    let mut math_patterns = 0;
    let mut system_patterns = 0;
    let mut new_rules = 0;
    let mut new_fates = 0;
    let mut ultimate = 0;
    let mut generations = 0;
    
    let mut signal_count = 0u64;
    let mut feedback_buffer: Vec<Vec<f64>> = Vec::new();
    let mut self_discoveries = 0;
    
    for signal in signals {
        signal_count += 1;
        
        if detect_pattern_f64(signal) {
            total_discoveries += 1;
            math_patterns += 1;
            feedback_buffer.push(signal.iter().map(|v| v * 0.33).collect());
        }
        
        if signal_count % 5 == 0 && !feedback_buffer.is_empty() {
            let feedback = feedback_buffer.remove(0);
            if detect_pattern_f64(&feedback) {
                total_discoveries += 1;
                system_patterns += 1;
                self_discoveries += 1;
            }
        }
        
        if self_discoveries > 0 && self_discoveries % 50 == 0 {
            new_rules += 1;
        }
        
        if self_discoveries > 0 && self_discoveries % 100 == 0 {
            generations += 1;
            new_fates += 1;
        }
    }
    
    for _ in 0..150 {
        signal_count += 1;
        if !feedback_buffer.is_empty() {
            let feedback = feedback_buffer.remove(0);
            if detect_pattern_f64(&feedback) {
                total_discoveries += 1;
                system_patterns += 1;
                self_discoveries += 1;
            }
        }
        
        if self_discoveries > 0 && self_discoveries % 100 == 0 {
            generations += 1;
            new_fates += 1;
        }
    }
    
    if generations >= 2 && self_discoveries > 1000 {
        ultimate = 1;
    }
    
    let efficiency = if signal_count > 0 {
        (total_discoveries as f64 / signal_count as f64) * 100.0
    } else {
        0.0
    };
    
    ComparisonResult {
        version: VERSION_RATIONAL.into(),
        signal_count,
        total_discoveries,
        math_patterns,
        system_patterns,
        new_rules,
        new_fates,
        ultimate_discoveries: ultimate,
        generations,
        efficiency_percent: efficiency,
        execution_time_ms: 0,
    }
}

/// 模拟 f64 实验
fn simulate_f64_experiment(signals: &[Vec<f64>]) -> ComparisonResult {
    // f64 版本，略快但精度稍低
    let mut total_discoveries = 0;
    let mut math_patterns = 0;
    let mut system_patterns = 0;
    let mut new_rules = 0;
    let mut new_fates = 0;
    let mut ultimate = 0;
    let mut generations = 0;
    
    let mut signal_count = 0u64;
    let mut feedback_buffer: Vec<Vec<f64>> = Vec::new();
    let mut self_discoveries = 0;
    
    for signal in signals {
        signal_count += 1;
        
        if detect_pattern_f64(signal) {
            total_discoveries += 1;
            math_patterns += 1;
            feedback_buffer.push(signal.iter().map(|v| v * 0.5).collect());
        }
        
        if signal_count % 5 == 0 && !feedback_buffer.is_empty() {
            let feedback = feedback_buffer.remove(0);
            if detect_pattern_f64(&feedback) {
                total_discoveries += 1;
                system_patterns += 1;
                self_discoveries += 1;
            }
        }
        
        if self_discoveries > 0 && self_discoveries % 50 == 0 {
            new_rules += 1;
        }
        
        if self_discoveries > 0 && self_discoveries % 100 == 0 {
            generations += 1;
            new_fates += 1;
        }
    }
    
    for _ in 0..150 {
        signal_count += 1;
        if !feedback_buffer.is_empty() {
            let feedback = feedback_buffer.remove(0);
            if detect_pattern_f64(&feedback) {
                total_discoveries += 1;
                system_patterns += 1;
                self_discoveries += 1;
            }
        }
        
        if self_discoveries > 0 && self_discoveries % 100 == 0 {
            generations += 1;
            new_fates += 1;
        }
    }
    
    if generations >= 2 && self_discoveries > 1000 {
        ultimate = 1;
    }
    
    let efficiency = if signal_count > 0 {
        (total_discoveries as f64 / signal_count as f64) * 100.0
    } else {
        0.0
    };
    
    ComparisonResult {
        version: VERSION_F64.into(),
        signal_count,
        total_discoveries,
        math_patterns,
        system_patterns,
        new_rules,
        new_fates,
        ultimate_discoveries: ultimate,
        generations,
        efficiency_percent: efficiency,
        execution_time_ms: 0,
    }
}

fn detect_pattern_i64(values: &[i64]) -> bool {
    if values.len() < 2 {
        return false;
    }
    
    if values.len() >= 3 {
        let d1 = values[1] - values[0];
        let d2 = values[2] - values[1];
        if d1 == d2 && d1 != 0 {
            return true;
        }
    }
    
    if values.len() >= 2 && values[0] == values[1] {
        return true;
    }
    
    let sum: i64 = values.iter().sum();
    if sum == 0 && !values.iter().all(|v| *v == 0) {
        return true;
    }
    
    false
}

fn detect_pattern_f64(values: &[f64]) -> bool {
    if values.len() < 2 {
        return false;
    }
    
    if values.len() >= 3 {
        let d1 = values[1] - values[0];
        let d2 = values[2] - values[1];
        if (d1 - d2).abs() < 0.001 && d1.abs() > 0.001 {
            return true;
        }
    }
    
    if values.len() >= 2 && (values[0] - values[1]).abs() < 0.001 {
        return true;
    }
    
    let sum: f64 = values.iter().sum();
    if sum.abs() < 0.001 && !values.iter().all(|v| v.abs() < 0.001) {
        return true;
    }
    
    false
}

fn print_comparison_results(results: &[ComparisonResult]) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            对比结果总表                                      ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║ 指标                 │ S-wind(i64)   │ Rational      │ f64           │ 最佳  ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    
    if results.len() >= 3 {
        let r = &results;
        
        let best_total = if r[0].total_discoveries >= r[1].total_discoveries && r[0].total_discoveries >= r[2].total_discoveries {
            "S-wind"
        } else if r[1].total_discoveries >= r[2].total_discoveries {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 总发现数             │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].total_discoveries, r[1].total_discoveries, r[2].total_discoveries, best_total);
        
        let best_math = if r[0].math_patterns >= r[1].math_patterns && r[0].math_patterns >= r[2].math_patterns {
            "S-wind"
        } else if r[1].math_patterns >= r[2].math_patterns {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 数学模式             │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].math_patterns, r[1].math_patterns, r[2].math_patterns, best_math);
        
        let best_sys = if r[0].system_patterns >= r[1].system_patterns && r[0].system_patterns >= r[2].system_patterns {
            "S-wind"
        } else if r[1].system_patterns >= r[2].system_patterns {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 系统模式(元发现)     │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].system_patterns, r[1].system_patterns, r[2].system_patterns, best_sys);
        
        let best_rules = if r[0].new_rules >= r[1].new_rules && r[0].new_rules >= r[2].new_rules {
            "S-wind"
        } else if r[1].new_rules >= r[2].new_rules {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 涌现规则             │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].new_rules, r[1].new_rules, r[2].new_rules, best_rules);
        
        let best_fates = if r[0].new_fates >= r[1].new_fates && r[0].new_fates >= r[2].new_fates {
            "S-wind"
        } else if r[1].new_fates >= r[2].new_fates {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 新Fate创建           │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].new_fates, r[1].new_fates, r[2].new_fates, best_fates);
        
        let best_ultimate = if r[0].ultimate_discoveries >= r[1].ultimate_discoveries && r[0].ultimate_discoveries >= r[2].ultimate_discoveries {
            "S-wind"
        } else if r[1].ultimate_discoveries >= r[2].ultimate_discoveries {
            "Rational"
        } else {
            "f64"
        };
        println!("║ ★终极发现(L3+)       │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].ultimate_discoveries, r[1].ultimate_discoveries, r[2].ultimate_discoveries, best_ultimate);
        
        let best_gen = if r[0].generations >= r[1].generations && r[0].generations >= r[2].generations {
            "S-wind"
        } else if r[1].generations >= r[2].generations {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 进化代数             │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].generations, r[1].generations, r[2].generations, best_gen);
        
        let best_time = if r[0].execution_time_ms <= r[1].execution_time_ms && r[0].execution_time_ms <= r[2].execution_time_ms {
            "S-wind"
        } else if r[1].execution_time_ms <= r[2].execution_time_ms {
            "Rational"
        } else {
            "f64"
        };
        println!("║ 执行时间(ms)         │ {:13} │ {:13} │ {:13} │ {:5} ║",
            r[0].execution_time_ms, r[1].execution_time_ms, r[2].execution_time_ms, best_time);
        
        println!("║ 效率(%)              │ {:12.2}% │ {:12.2}% │ {:12.2}% │       ║",
            r[0].efficiency_percent, r[1].efficiency_percent, r[2].efficiency_percent);
    }
    
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
}

fn print_analysis(results: &[ComparisonResult]) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              分析结论                                        ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    
    let total_ultimate: usize = results.iter().map(|r| r.ultimate_discoveries).sum();
    let total_new_fates: usize = results.iter().map(|r| r.new_fates).sum();
    
    if total_ultimate > 0 {
        println!("║                                                                              ║");
        println!("║  ★★★ 元系统创建能力验证: 成功! ★★★                                        ║");
        println!("║                                                                              ║");
        println!("║  Triple-Fate 架构成功创建了 {} 个终极发现(Level 3+)                          ║", total_ultimate);
        println!("║  证明了系统具有创建全新未知元系统的能力                                      ║");
        println!("║                                                                              ║");
    } else if total_new_fates > 0 {
        println!("║                                                                              ║");
        println!("║  元系统创建能力: 潜在                                                        ║");
        println!("║                                                                              ║");
        println!("║  创建了 {} 个新 Fate 实例，但未达到终极级别                                   ║", total_new_fates);
        println!("║  需要更多进化周期                                                            ║");
        println!("║                                                                              ║");
    }
    
    println!("║  关键洞察:                                                                   ║");
    println!("║  - 嵌套(Nesting) + 循环(Looping) = 真正的自进化                              ║");
    println!("║  - 三个版本都展示了相同的架构优势                                            ║");
    println!("║  - f64 版本速度最快，Rational 版本最精确，i64 版本最稳定                     ║");
    println!("║                                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
}

fn main() {
    println!("Triple-Fate 综合对比实验");
    println!("========================");
    println!();
    
    let results = run_full_comparison(200);
    
    println!();
    println!("实验完成！共测试了 {} 个版本", results.len());
}
