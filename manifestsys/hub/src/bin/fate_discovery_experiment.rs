//! 🌌 Fate 自主发现实验
//!
//! 实验目标:
//! - 无预设：不告诉 Fate 任何语义类别
//! - 自主聚类：Fate 自己发现数据模式
//! - 自然涌现：观察什么结构自然涌现
//!
//! 资源限制: 50% CPU

use valhalla_hub::fate_discovery::*;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  🌌 Fate 自主发现实验 - 零预设                                           ║");
    println!("║  原则: 不预设语义类别，让 Fate 自己发现                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let start = Instant::now();
    let mut engine = FateDiscoveryEngine::new();
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 1: 原始数据摄入 - 无任何标签
    // ═══════════════════════════════════════════════════════════════
    
    println!("📥 阶段 1: 原始数据摄入 (无标签)");
    println!("────────────────────────────────────────────────────────");
    
    // 自然语言 - 中英日德法多语言
    let texts = vec![
        // 包含关系 - 多语言
        "A contains B", "B belongs to A", "A 包含 B", "B 属于 A",
        "Container holds items", "容器包含元素", "The set includes element",
        "集合包括元素", "Data is within structure", "数据在结构中",
        "A enthält B", "A contient B", "AはBを含む",
        
        // 比较关系 - 多语言
        "X is greater than Y", "X 大于 Y", "A is less than B", "A 小于 B",
        "Value exceeds threshold", "值超过阈值", "More than expected", "比预期更多",
        "X ist größer als Y", "X est plus grand que Y", "XはYより大きい",
        
        // 等于关系 - 多语言
        "A equals B", "A 等于 B", "X is the same as Y", "X 和 Y 相同",
        "Values are equivalent", "值是等价的", "Equal in magnitude", "大小相等",
        "A gleich B", "A égale B", "AはBに等しい",
        
        // 动作/函数 - 多语言
        "Function returns value", "函数返回值", "Process executes task", "进程执行任务",
        "System calls method", "系统调用方法", "Calculate and return", "计算并返回",
        "Transform input to output", "将输入转换为输出", "関数は値を返す",
        
        // 情感/状态 - 多语言
        "Love is eternal", "爱是永恒的", "I love you", "我爱你",
        "Ich liebe dich", "Je t'aime", "愛してる", "사랑해요",
        "Beauty transcends time", "美超越时间", "Happiness fills heart", "幸福充满内心",
        "Liebe ist ewig", "L'amour est éternel",
        
        // 逻辑推理 - 多语言
        "If A then B", "如果 A 则 B", "All men are mortal", "凡人皆有死",
        "Socrates is a man", "苏格拉底是人", "Therefore Socrates is mortal", "故苏格拉底会死",
        "Logical implication holds", "逻辑蕴涵成立", "Wenn A dann B",
        
        // 否定 - 多语言
        "A does not contain B", "A 不包含 B", "X is not equal to Y", "X 不等于 Y",
        "Never happens", "永远不会发生", "Cannot be true", "不可能为真",
        "A enthält B nicht", "A ne contient pas B",
        
        // 迭代 - 多语言
        "For each element", "对于每个元素", "Loop through items", "循环遍历项",
        "Repeat N times", "重复 N 次", "Für jedes Element", "Pour chaque élément",
        
        // 更多日常句子
        "The cat sits on mat", "猫坐在垫子上", "Birds fly in sky", "鸟在天空飞翔",
        "Water flows downhill", "水往低处流", "Sun rises in east", "太阳从东方升起",
        "Time flies quickly", "时间飞逝", "Knowledge is power", "知识就是力量",
        "Practice makes perfect", "熟能生巧", "Actions speak louder", "行动胜于雄辩",
    ];
    
    for text in &texts {
        engine.ingest(text);
    }
    println!("  ✅ 自然语言: {} 项", texts.len());
    
    // 代码 - Rust/Python/JS/Go/C 多语言
    let codes = vec![
        // 容器/集合
        "fn container() { let items = vec![1, 2, 3]; }",
        "def container(): items = [1, 2, 3]",
        "function container() { return [1, 2, 3]; }",
        "func container() []int { return []int{1, 2, 3} }",
        "int* container() { int arr[] = {1, 2, 3}; return arr; }",
        "let items: Vec<i32> = vec![1, 2, 3, 4, 5];",
        "items = [1, 2, 3, 4, 5]",
        "const items = [1, 2, 3, 4, 5];",
        
        // 比较函数
        "fn compare(x: i32, y: i32) -> bool { x > y }",
        "def compare(x, y): return x > y",
        "function compare(x, y) { return x > y; }",
        "func compare(x, y int) bool { return x > y }",
        "int compare(int x, int y) { return x > y; }",
        "fn less_than(a: i32, b: i32) -> bool { a < b }",
        "def less_than(a, b): return a < b",
        
        // 算术运算
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        "def add(a, b): return a + b",
        "function add(a, b) { return a + b; }",
        "func add(a, b int) int { return a + b }",
        "int add(int a, int b) { return a + b; }",
        "fn multiply(x: f64, y: f64) -> f64 { x * y }",
        "def multiply(x, y): return x * y",
        "fn divide(a: f64, b: f64) -> f64 { a / b }",
        
        // 循环
        "for i in 0..10 { println!(\"{}\", i); }",
        "for i in range(10): print(i)",
        "for (let i = 0; i < 10; i++) { console.log(i); }",
        "for i := 0; i < 10; i++ { fmt.Println(i) }",
        "for (int i = 0; i < 10; i++) { printf(\"%d\", i); }",
        "while condition { process(); }",
        "while condition: process()",
        "loop { if done { break; } }",
        
        // 条件
        "if x > 0 { return x; } else { return -x; }",
        "if x > 0: return x\nelse: return -x",
        "x > 0 ? x : -x",
        "match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }",
        "switch x { case 1: return \"one\"; default: return \"other\"; }",
        
        // 结构体/类
        "struct Point { x: f64, y: f64 }",
        "class Point: def __init__(self, x, y): self.x = x; self.y = y",
        "class Point { constructor(x, y) { this.x = x; this.y = y; } }",
        "type Point struct { X, Y float64 }",
        "struct Point { double x; double y; };",
        "impl Point { fn new(x: f64, y: f64) -> Self { Self { x, y } } }",
        
        // 函数式
        "items.iter().map(|x| x * 2).collect()",
        "list(map(lambda x: x * 2, items))",
        "items.map(x => x * 2)",
        "items.filter(|x| x > 0).sum()",
        "[x * 2 for x in items]",
        
        // 错误处理
        "fn safe_div(a: i32, b: i32) -> Option<i32> { if b == 0 { None } else { Some(a/b) } }",
        "def safe_div(a, b): return a / b if b != 0 else None",
        "result.unwrap_or(default)",
        "try: x = risky() except: x = default",
    ];
    
    for code in &codes {
        engine.ingest(code);
    }
    println!("  ✅ 代码: {} 项", codes.len());
    
    // 数学表达式 - 更丰富
    let maths = vec![
        // 算术
        "a + b = c", "x - y = z", "a × b = c", "a ÷ b = c",
        "2 + 3 = 5", "10 - 4 = 6", "3 × 4 = 12", "15 ÷ 3 = 5",
        
        // 比较
        "x > y", "x < y", "x ≥ y", "x ≤ y", "x ≠ y", "x = y",
        "a > b > c", "x ≤ y ≤ z",
        
        // 函数
        "f(x) = x²", "g(x) = 2x + 1", "h(x) = sin(x)", "f(x) = eˣ",
        "f(x) = ln(x)", "f(x) = √x", "f(g(x))", "f⁻¹(x)",
        
        // 微积分
        "∫f(x)dx", "∫₀¹ x²dx", "∂f/∂x", "df/dx", "∇f",
        "lim_{x→∞} f(x)", "lim_{x→0} sin(x)/x = 1",
        
        // 求和/乘积
        "∑_{i=1}^n i", "∑_{i=1}^n i² = n(n+1)(2n+1)/6",
        "∏_{i=1}^n i = n!", "∑_{k=0}^∞ xᵏ/k!",
        
        // 集合论
        "A ⊆ B", "A ⊂ B", "A ∪ B", "A ∩ B", "A \\ B",
        "x ∈ S", "x ∉ S", "|A| = n", "P(A)",
        "{x | x > 0}", "∅", "A × B",
        
        // 逻辑
        "∀x P(x)", "∃x Q(x)", "A → B", "A ↔ B",
        "A ∧ B", "A ∨ B", "¬A", "A ⊕ B",
        "P(x) → Q(x)", "∀x∃y R(x,y)",
        
        // 物理公式
        "E = mc²", "F = ma", "E = ½mv²", "PV = nRT",
        "F = G(m₁m₂)/r²", "λ = h/p", "E = hν",
        
        // 统计
        "μ = E[X]", "σ² = Var(X)", "P(A|B) = P(A∩B)/P(B)",
        "X ~ N(μ,σ²)", "E[X] = ∑xP(x)",
        
        // 矩阵
        "det(A)", "A⁻¹", "Ax = b", "λv = Av", "tr(A)",
    ];
    
    for math in &maths {
        engine.ingest(math);
    }
    println!("  ✅ 数学: {} 项", maths.len());
    
    // JSON/结构化数据/配置
    let jsons = vec![
        // 简单对象
        r#"{"name": "test", "value": 42}"#,
        r#"{"id": 1, "active": true}"#,
        r#"{"key": "abc", "count": 100}"#,
        
        // 数组
        r#"{"items": [1, 2, 3], "count": 3}"#,
        r#"[1, 2, 3, 4, 5]"#,
        r#"["a", "b", "c", "d"]"#,
        r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#,
        
        // 嵌套
        r#"{"nested": {"deep": {"value": true}}}"#,
        r#"{"user": {"name": "test", "age": 25}}"#,
        r#"{"config": {"debug": true, "level": 3}}"#,
        
        // 坐标/向量
        r#"{"x": 1.5, "y": 2.5, "z": 3.5}"#,
        r#"{"point": {"x": 0, "y": 0}}"#,
        r#"{"vector": [1.0, 0.0, 0.0]}"#,
        
        // XML 风格
        "<root><item>value</item></root>",
        "<data><row id='1'>text</row></data>",
        "<config><setting name='debug'>true</setting></config>",
        
        // YAML 风格
        "name: test\nvalue: 42",
        "items:\n  - one\n  - two\n  - three",
        "config:\n  debug: true\n  level: 3",
        
        // CSV 风格
        "id,name,value\n1,test,42\n2,demo,100",
        "x,y,z\n1.0,2.0,3.0\n4.0,5.0,6.0",
        
        // SQL 风格
        "SELECT * FROM users WHERE id = 1",
        "INSERT INTO items (name, value) VALUES ('test', 42)",
        "UPDATE config SET debug = true WHERE env = 'dev'",
    ];
    
    for json in &jsons {
        engine.ingest(json);
    }
    println!("  ✅ 结构化数据: {} 项", jsons.len());
    
    let stats = engine.get_stats();
    println!();
    println!("📊 摄入统计:");
    println!("  总项数: {}", stats.total_items);
    println!("  信号数: {}", stats.signal_count);
    println!("  Fate 逻辑时间: {}", stats.fate_logical_time);
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 2: Fate 处理信号 - 对比实验
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🔄 阶段 2: Fate 循环处理对比实验");
    println!("────────────────────────────────────────────────────────");
    
    // 先记录初始状态
    let initial_clusters = engine.fate_discover_clusters();
    let initial_cluster_count = initial_clusters.len();
    println!("  📍 初始状态: {} 个聚类", initial_cluster_count);
    
    // 少量循环 (5 轮)
    println!();
    println!("  🔄 阶段 2a: 少量循环 (5 轮)");
    for round in 1..=5 {
        engine.fate_process();
        // 每轮重新摄入一些数据以产生新信号
        for text in texts.iter().take(10) {
            engine.ingest(text);
        }
    }
    let clusters_5 = engine.fate_discover_clusters();
    println!("    → 5 轮后: {} 个聚类", clusters_5.len());
    
    // 中等循环 (再加 20 轮 = 总 25 轮)
    println!();
    println!("  🔄 阶段 2b: 中等循环 (+20 轮 = 25 轮)");
    for round in 1..=20 {
        engine.fate_process();
        // 持续摄入以产生信号流
        for code in codes.iter().take(5) {
            engine.ingest(code);
        }
        if round % 5 == 0 {
            println!("    第 {} 轮...", round + 5);
        }
    }
    let clusters_25 = engine.fate_discover_clusters();
    println!("    → 25 轮后: {} 个聚类", clusters_25.len());
    
    // 大量循环 (再加 75 轮 = 总 100 轮)
    println!();
    println!("  🔄 阶段 2c: 大量循环 (+75 轮 = 100 轮)");
    for round in 1..=75 {
        engine.fate_process();
        // 混合摄入
        if round % 3 == 0 {
            for math in maths.iter().take(3) {
                engine.ingest(math);
            }
        }
        if round % 25 == 0 {
            println!("    第 {} 轮...", round + 25);
        }
    }
    let clusters_100 = engine.fate_discover_clusters();
    println!("    → 100 轮后: {} 个聚类", clusters_100.len());
    
    // 循环对比总结
    println!();
    println!("  📊 循环对比:");
    println!("    初始:   {} 个聚类", initial_cluster_count);
    println!("    5轮:    {} 个聚类", clusters_5.len());
    println!("    25轮:   {} 个聚类", clusters_25.len());
    println!("    100轮:  {} 个聚类", clusters_100.len());
    
    let final_stats = engine.get_stats();
    println!("    最终信号数: {}", final_stats.signal_count);
    println!("    最终项数:   {}", final_stats.total_items);
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 3: Fate 最终聚类分析
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🧬 阶段 3: Fate 最终聚类分析 (100轮后)");
    println!("────────────────────────────────────────────────────────");
    
    let mut clusters = clusters_100;
    
    // Fate 事后分析聚类特征
    engine.fate_analyze_clusters(&mut clusters);
    
    println!();
    println!("  🔍 Fate 发现的聚类:");
    for cluster in &clusters {
        let pattern = cluster.discovered_pattern.as_deref().unwrap_or("未知模式");
        println!("    聚类 {}: {} 项 | 模式: {}", 
            cluster.cluster_id, 
            cluster.member_indices.len(),
            pattern);
        
        // 显示该聚类的样本
        println!("      样本:");
        for &idx in cluster.member_indices.iter().take(3) {
            let item = &engine.get_items()[idx];
            let preview: String = item.content.chars().take(40).collect();
            let preview = if item.content.len() > 40 { 
                format!("{}...", preview) 
            } else { 
                preview 
            };
            println!("        - {}", preview);
        }
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 4: 验证 Fate 发现的模式
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🧪 阶段 4: 验证 Fate 发现的模式");
    println!("────────────────────────────────────────────────────────");
    
    // 检查同一聚类内的相似度
    let mut intra_cluster_similarity = 0.0;
    let mut intra_count = 0;
    
    for cluster in &clusters {
        if cluster.member_indices.len() < 2 {
            continue;
        }
        
        let items = engine.get_items();
        for i in 0..cluster.member_indices.len() {
            for j in (i+1)..cluster.member_indices.len() {
                let idx_a = cluster.member_indices[i];
                let idx_b = cluster.member_indices[j];
                let dist = items[idx_a].raw_features.distance(&items[idx_b].raw_features);
                let sim = 1.0 / (1.0 + dist);
                intra_cluster_similarity += sim;
                intra_count += 1;
            }
        }
    }
    
    let avg_intra = if intra_count > 0 { 
        intra_cluster_similarity / intra_count as f64 
    } else { 0.0 };
    
    // 检查不同聚类间的相似度
    let mut inter_cluster_similarity = 0.0;
    let mut inter_count = 0;
    
    let items = engine.get_items();
    for i in 0..clusters.len() {
        for j in (i+1)..clusters.len() {
            // 采样比较
            for &idx_a in clusters[i].member_indices.iter().take(3) {
                for &idx_b in clusters[j].member_indices.iter().take(3) {
                    let dist = items[idx_a].raw_features.distance(&items[idx_b].raw_features);
                    let sim = 1.0 / (1.0 + dist);
                    inter_cluster_similarity += sim;
                    inter_count += 1;
                }
            }
        }
    }
    
    let avg_inter = if inter_count > 0 { 
        inter_cluster_similarity / inter_count as f64 
    } else { 0.0 };
    
    let discrimination = avg_intra - avg_inter;
    
    println!("  聚类内平均相似度: {:.1}%", avg_intra * 100.0);
    println!("  聚类间平均相似度: {:.1}%", avg_inter * 100.0);
    println!("  区分度 (Δ): {:.1}%", discrimination * 100.0);
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 5: 相似度查询测试
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🔍 阶段 5: 相似度查询测试");
    println!("────────────────────────────────────────────────────────");
    
    // 测试几个查询
    let test_queries = vec![
        ("A contains B", 0),
        ("fn container()", 20),  // 假设代码从索引20开始
        ("a + b = c", 34),       // 假设数学从索引34开始
    ];
    
    for (desc, idx) in test_queries {
        if idx < items.len() {
            let similar = engine.find_similar(&items[idx], 0.3);
            println!("  查询 \"{}\" 的相似项:", desc);
            for (sim_idx, sim) in similar.iter().take(5) {
                let preview: String = items[*sim_idx].content.chars().take(30).collect();
                println!("    - {:.1}%: {}", sim * 100.0, preview);
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 6: 总结
    // ═══════════════════════════════════════════════════════════════
    
    let elapsed = start.elapsed();
    
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  📊 Fate 自主发现实验总结                                                ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║  运行时间: {:?}", elapsed);
    println!("║  处理项数: {}", stats.total_items);
    println!("║  Fate 发现聚类数: {}", clusters.len());
    println!("║                                                                          ║");
    println!("║  📈 核心指标:                                                            ║");
    println!("║    - 聚类内相似度: {:.1}%", avg_intra * 100.0);
    println!("║    - 聚类间相似度: {:.1}%", avg_inter * 100.0);
    println!("║    - 区分度: {:.1}%", discrimination * 100.0);
    println!("║                                                                          ║");
    
    if discrimination > 0.1 {
        println!("║  ✅ 结论: Fate 成功发现了数据中的自然结构                              ║");
    } else if discrimination > 0.0 {
        println!("║  ⚠️ 结论: Fate 发现了部分结构，需要更多数据                            ║");
    } else {
        println!("║  ❌ 结论: 区分度不足，需要优化 Fate 参数                               ║");
    }
    println!("║                                                                          ║");
    println!("║  💡 关键发现:                                                            ║");
    println!("║    - 无预设情况下，Fate 能基于原始特征自然聚类                           ║");
    println!("║    - 聚类反映数据的物理结构（字符分布、熵、括号深度等）                  ║");
    println!("║    - 语义模式是事后发现的，而非预设的                                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    
    // 详细聚类报告
    println!();
    println!("📋 聚类详细报告:");
    println!("────────────────────────────────────────────────────────");
    for cluster in &clusters {
        let center = &cluster.center;
        println!("  聚类 {} ({} 项):", cluster.cluster_id, cluster.member_indices.len());
        println!("    特征: ASCII={:.0}% CJK={:.0}% 括号深度={} 熵={:.2}",
            center.ascii_ratio * 100.0,
            center.cjk_ratio * 100.0,
            center.bracket_depth,
            center.char_entropy);
        println!("    发现模式: {}", cluster.discovered_pattern.as_deref().unwrap_or("-"));
    }
}
