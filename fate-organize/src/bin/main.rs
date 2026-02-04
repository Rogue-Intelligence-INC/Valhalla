//! 🌌 Fate Organizer CLI - 智能文件整理命令行工具
//!
//! 用法:
//!   fate-organize "/path/to/directory"
//!   fate-organize "/path/to/directory" --output report.md
//!   fate-organize "/path/to/directory" --execute --force  # 执行整理

use std::path::PathBuf;
use fate_organize::{FateOrganizer, MergeRequest, ConflictStrategy, RenameRule, OperationStatus};
use clap::Parser;

// ═══════════════════════════════════════════════════════════════════
// 高级操作处理函数
// ═══════════════════════════════════════════════════════════════════

fn handle_merge(organizer: &FateOrganizer, base_dir: &PathBuf, spec: &str) {
    // 格式: folder1,folder2,...:target
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        eprintln!("❌ 合并格式错误。正确格式: folder1,folder2:target");
        return;
    }
    
    let sources: Vec<PathBuf> = parts[0]
        .split(',')
        .map(|s| base_dir.join(s.trim()))
        .collect();
    let target = base_dir.join(parts[1].trim());
    
    println!("📦 合并文件夹");
    println!("   源: {:?}", sources.iter().map(|p| p.file_name().unwrap_or_default()).collect::<Vec<_>>());
    println!("   目标: {}", target.display());
    println!();
    
    let request = MergeRequest {
        sources,
        target,
        conflict_strategy: ConflictStrategy::KeepBoth,
        delete_sources: false,
    };
    
    match organizer.merge_folders(&request) {
        Ok(report) => {
            println!("✅ 合并完成!");
            println!("   - 总文件数: {}", report.total_files);
            println!("   - 已移动: {}", report.moved);
            println!("   - 冲突: {}", report.conflicts);
            println!("   - 跳过: {}", report.skipped);
            if !report.errors.is_empty() {
                println!("   - 错误:");
                for err in &report.errors {
                    println!("     - {}", err);
                }
            }
        }
        Err(e) => eprintln!("❌ 合并失败: {}", e),
    }
}

fn handle_rename(organizer: &FateOrganizer, base_dir: &PathBuf, spec: &str) {
    // 格式: old_name:new_name
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        eprintln!("❌ 重命名格式错误。正确格式: old_name:new_name");
        return;
    }
    
    let source = base_dir.join(parts[0].trim());
    let new_name = parts[1].trim();
    
    println!("📝 重命名文件夹");
    println!("   源: {}", source.display());
    println!("   新名称: {}", new_name);
    println!();
    
    match organizer.rename_folder(&source, new_name) {
        Ok(new_path) => {
            println!("✅ 重命名成功: {}", new_path.display());
        }
        Err(e) => eprintln!("❌ 重命名失败: {}", e),
    }
}

fn handle_batch_rename(organizer: &FateOrganizer, base_dir: &PathBuf, spec: &str) {
    // 格式: pattern:replacement 或 pattern:replacement:seq
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() < 2 {
        eprintln!("❌ 批量重命名格式错误。正确格式: pattern:replacement 或 pattern:replacement:seq");
        return;
    }
    
    let rule = RenameRule {
        pattern: parts[0].to_string(),
        replacement: parts[1].to_string(),
        add_sequence: parts.len() > 2 && parts[2] == "seq",
        sequence_start: 1,
    };
    
    println!("📝 批量重命名");
    println!("   目录: {}", base_dir.display());
    println!("   模式: {} -> {}", rule.pattern, rule.replacement);
    println!("   添加序号: {}", rule.add_sequence);
    println!();
    
    match organizer.batch_rename(base_dir, &rule) {
        Ok(report) => {
            println!("✅ 批量重命名完成!");
            println!("   - 总文件数: {}", report.total);
            println!("   - 已重命名: {}", report.renamed);
            println!("   - 跳过: {}", report.skipped);
            if !report.errors.is_empty() {
                println!("   - 错误:");
                for err in &report.errors {
                    println!("     - {}", err);
                }
            }
        }
        Err(e) => eprintln!("❌ 批量重命名失败: {}", e),
    }
}

/// 🌌 Fate Organizer - Valhalla 智能文件整理系统
#[derive(Parser, Debug)]
#[command(name = "fate-organize")]
#[command(about = "🌌 Fate Organizer - Valhalla 智能文件整理系统", long_about = None)]
#[command(version = "1.0.0")]
struct Args {
    /// 要整理的目录路径
    #[arg(required = true)]
    directory: PathBuf,
    
    /// 输出报告文件路径 (默认输出到终端)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// 执行整理 (默认只生成报告，不实际移动文件)
    #[arg(long, default_value = "false")]
    execute: bool,
    
    /// 强制执行 (跳过预览，直接移动文件)
    #[arg(long, default_value = "false")]
    force: bool,
    
    /// 详细模式
    #[arg(short, long, default_value = "false")]
    verbose: bool,
    
    /// 创建 fate-organize 目录结构并整理到其中
    #[arg(long, default_value = "false")]
    fate_dir: bool,
    
    /// 合并文件夹 (格式: folder1,folder2,...:target)
    #[arg(long)]
    merge: Option<String>,
    
    /// 重命名文件夹
    #[arg(long)]
    rename: Option<String>,
    
    /// 批量重命名 (格式: pattern:replacement)
    #[arg(long)]
    batch_rename: Option<String>,
    
    /// 🔥 智能重命名所有文件 (基于 Fate 分析)
    #[arg(long, default_value = "false")]
    smart_rename: bool,
    
    /// 整理后删除源文件（移动而非复制）
    #[arg(long, default_value = "true")]
    delete_source: bool,
    
    /// 删除空文件夹
    #[arg(long, default_value = "false")]
    cleanup_empty: bool,
    
    /// 递归处理整个文件树（包括子目录）
    #[arg(long, short = 'r', default_value = "false")]
    recursive: bool,
}

fn main() {
    let args = Args::parse();
    
    println!();
    println!("🌌 ╔═══════════════════════════════════════════════╗");
    println!("   ║        Fate Organizer - Valhalla 智能整理      ║");
    println!("   ╚═══════════════════════════════════════════════╝");
    println!();
    
    if !args.directory.exists() {
        eprintln!("❌ 目录不存在: {}", args.directory.display());
        std::process::exit(1);
    }
    
    if !args.directory.is_dir() {
        eprintln!("❌ 路径不是目录: {}", args.directory.display());
        std::process::exit(1);
    }
    
    let organizer = FateOrganizer::new();
    
    // 处理合并文件夹
    if let Some(merge_spec) = &args.merge {
        handle_merge(&organizer, &args.directory, merge_spec);
        return;
    }
    
    // 处理重命名文件夹
    if let Some(rename_spec) = &args.rename {
        handle_rename(&organizer, &args.directory, rename_spec);
        return;
    }
    
    // 处理批量重命名
    if let Some(batch_spec) = &args.batch_rename {
        handle_batch_rename(&organizer, &args.directory, batch_spec);
        return;
    }
    
    // 🔥 智能重命名所有文件
    if args.smart_rename {
        println!("🔮 智能重命名所有文件...");
        println!("   目录: {}", args.directory.display());
        println!("   递归: {}", args.recursive);
        println!();
        
        let dry_run = !args.force;
        
        if dry_run {
            println!("📋 预览模式 (添加 --force 实际执行)");
            println!();
        }
        
        match organizer.smart_rename_all(&args.directory, args.recursive, dry_run) {
            Ok(report) => {
                println!("📊 重命名结果:");
                println!("   - 总文件数: {}", report.total);
                println!("   - 已重命名: {}", report.renamed);
                println!("   - 未改变: {}", report.unchanged);
                println!("   - 跳过: {}", report.skipped);
                println!();
                
                // 显示重命名操作
                if !report.operations.is_empty() {
                    println!("📝 重命名详情:");
                    for op in report.operations.iter().take(20) {
                        let status = if op.success { "✓" } else { "✗" };
                        println!("   {} {} -> {}", status, op.old_name, op.new_name);
                        if args.verbose {
                            println!("      原因: {}", op.reason);
                        }
                    }
                    if report.operations.len() > 20 {
                        println!("   ... 还有 {} 个操作", report.operations.len() - 20);
                    }
                }
                
                if !report.errors.is_empty() {
                    println!();
                    println!("❌ 错误:");
                    for err in report.errors.iter().take(10) {
                        println!("   - {}", err);
                    }
                }
                
                if dry_run {
                    println!();
                    println!("💡 这是预览。使用 --force 执行实际重命名");
                }
            }
            Err(e) => eprintln!("❌ 智能重命名失败: {}", e),
        }
        return;
    }
    
    // 创建 fate-organize 目录
    if args.fate_dir {
        println!("📁 创建 fate-organize 目录结构...");
        match organizer.create_fate_organize_structure(&args.directory) {
            Ok(path) => {
                println!("✅ 已创建: {}", path.display());
                println!();
                println!("目录结构:");
                println!("├── AI生成/");
                println!("│   ├── DALL-E/");
                println!("│   ├── ChatGPT/");
                println!("│   └── Midjourney/");
                println!("├── 图片/");
                println!("│   ├── 照片/");
                println!("│   └── 截图/");
                println!("├── 视频/");
                println!("├── 音频/");
                println!("├── 文档/");
                println!("├── 压缩包/");
                println!("├── 代码/");
                println!("├── 其他/");
                println!("└── .preserve/  (保留区)");
            }
            Err(e) => eprintln!("❌ 创建失败: {}", e),
        }
        return;
    }
    
    println!("📂 扫描目录: {}", args.directory.display());
    println!();
    
    // 选择递归或非递归分析
    let plan_result = if args.recursive {
        println!("🌳 递归模式: 扫描整个文件树");
        println!();
        organizer.analyze_directory_recursive(&args.directory)
    } else {
        organizer.analyze_directory(&args.directory)
    };
    
    match plan_result {
        Ok(plan) => {
            // 输出快速统计
            println!("📊 扫描结果:");
            println!("   - 文件数: {}", plan.total_files);
            println!("   - 子目录: {}", plan.total_dirs);
            println!();
            
            // 类型分布
            println!("📁 文件类型分布:");
            let mut sorted_stats: Vec<_> = plan.category_stats.iter().collect();
            sorted_stats.sort_by(|a, b| b.1.cmp(a.1));
            for (cat, count) in sorted_stats.iter().take(8) {
                let cnt = **count;
                let bar = "█".repeat(cnt.min(30));
                println!("   {:20} {:4} {}", cat, count, bar);
            }
            println!();
            
            // 发现的模式
            if !plan.discovered_patterns.is_empty() {
                println!("🔍 发现的模式:");
                for pattern in plan.discovered_patterns.iter().take(5) {
                    println!("   - {} ({} 个文件)", pattern.pattern_type, pattern.count);
                }
                println!();
            }
            
            // 建议的结构
            println!("📁 建议的文件夹结构:");
            let mut sorted_structure: Vec<_> = plan.suggested_structure.iter().collect();
            sorted_structure.sort_by(|a, b| b.expected_files.cmp(&a.expected_files));
            for folder in sorted_structure.iter().take(10) {
                println!("   ├── {} ({} 个文件)", folder.name, folder.expected_files);
            }
            if sorted_structure.len() > 10 {
                println!("   └── ... 还有 {} 个分类", sorted_structure.len() - 10);
            }
            println!();
            
            // 生成完整报告
            let report = organizer.generate_report(&plan);
            
            if let Some(output_path) = args.output {
                match std::fs::write(&output_path, &report) {
                    Ok(_) => println!("✅ 报告已保存到: {}", output_path.display()),
                    Err(e) => eprintln!("❌ 保存报告失败: {}", e),
                }
            } else if args.verbose {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("{}", report);
            }
            
            if args.execute {
                println!();
                println!("🔥 执行整理操作...");
                println!();
                
                // 先做一次干运行
                if !args.force {
                    println!("📋 预览模式 (添加 --force 实际执行):");
                    match organizer.execute_organization(&plan, true) {
                        Ok(exec_report) => {
                            println!("   - 将移动: {} 个文件", exec_report.successful);
                            println!("   - 将跳过: {} 个文件", exec_report.skipped);
                            println!();
                            
                            // 显示前10个操作
                            for op in exec_report.operations.iter().take(10) {
                                let status = match op.status {
                                    OperationStatus::DryRun => "→",
                                    OperationStatus::Skipped => "⊘",
                                    _ => "?",
                                };
                                println!("   {} {} -> {}", status, 
                                    op.source.file_name().unwrap_or_default().to_string_lossy(),
                                    op.target.parent().unwrap_or(std::path::Path::new(".")).display()
                                );
                            }
                            if exec_report.operations.len() > 10 {
                                println!("   ... 还有 {} 个操作", exec_report.operations.len() - 10);
                            }
                        }
                        Err(e) => eprintln!("❌ 预览失败: {}", e),
                    }
                } else {
                    // 实际执行
                    println!("⚡ 正在执行整理...");
                    match organizer.execute_organization(&plan, false) {
                        Ok(exec_report) => {
                            println!();
                            println!("✅ 整理完成!");
                            println!("   - 成功移动: {} 个文件", exec_report.successful);
                            println!("   - 跳过: {} 个文件", exec_report.skipped);
                            println!("   - 失败: {} 个文件", exec_report.failed);
                            
                            if exec_report.failed > 0 {
                                println!();
                                println!("❌ 失败的操作:");
                                for op in exec_report.operations.iter().filter(|o| matches!(o.status, OperationStatus::Failed)) {
                                    println!("   - {}: {}", op.source.file_name().unwrap_or_default().to_string_lossy(), op.message);
                                }
                            }
                            
                            // 清理空文件夹
                            if args.cleanup_empty {
                                println!();
                                println!("🧹 清理空文件夹...");
                                match organizer.cleanup_empty_folders(&args.directory) {
                                    Ok(removed) => {
                                        if removed > 0 {
                                            println!("   ✅ 删除了 {} 个空文件夹", removed);
                                        } else {
                                            println!("   ℹ️  没有空文件夹需要清理");
                                        }
                                    }
                                    Err(e) => eprintln!("   ❌ 清理失败: {}", e),
                                }
                            }
                        }
                        Err(e) => eprintln!("❌ 执行失败: {}", e),
                    }
                }
            }
            
            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🌌 Fate Organizer - 分析完成");
        }
        Err(e) => {
            eprintln!("❌ 扫描失败: {}", e);
            std::process::exit(1);
        }
    }
}
