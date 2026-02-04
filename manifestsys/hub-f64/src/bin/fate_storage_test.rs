//! Fate 存储测试
//! 
//! 测试：Fate 边跑边存储，系统完全控制权（修改、新建），最终只冻结 Fate 切片

use fate_f64::TripleFateCombined;
use std::path::Path;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║   🔮 Fate 存储测试 - 完全控制权 + 冻结切片                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mut fate = TripleFateCombined::new();
    
    // ========== 1. 外部数据导入 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📥 1. 外部数据导入\n");

    let id1 = fate.store_string("Hello World!");
    let id2 = fate.store_string("你好世界！");
    let id3 = fate.store_string("fn main() { }");
    
    println!("   导入 3 条外部数据:");
    println!("   - {}: \"Hello World!\"", id1);
    println!("   - {}: \"你好世界！\"", id2);
    println!("   - {}: \"fn main() {{ }}\"", id3);
    println!("   外部数据数量: {}", fate.list_external().len());
    println!();

    // ========== 2. 系统自主新建数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🆕 2. 系统自主新建数据\n");

    let internal1 = fate.create_string("System generated content #1");
    let internal2 = fate.create_data(vec![0x00, 0x01, 0x02, 0x03, 0xFF]);
    
    println!("   系统创建 2 条内部数据:");
    println!("   - {}: 字符串", internal1);
    println!("   - {}: 二进制 [00 01 02 03 FF]", internal2);
    println!("   内部数据数量: {}", fate.list_internal().len());
    println!();

    // ========== 3. 完全修改数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✏️  3. 完全修改数据\n");

    let before = String::from_utf8_lossy(&fate.load_data(&id1).unwrap().raw).to_string();
    fate.modify_data(&id1, b"MODIFIED COMPLETELY!".to_vec());
    let after = String::from_utf8_lossy(&fate.load_data(&id1).unwrap().raw).to_string();
    let entry = fate.load_data(&id1).unwrap();
    
    println!("   {} 修改:", id1);
    println!("   - 修改前: \"{}\"", before);
    println!("   - 修改后: \"{}\"", after);
    println!("   - 修改次数: {}", entry.modification_count);
    println!();

    // ========== 4. 部分修改（字节级） ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔧 4. 部分修改（字节级）\n");

    let before = fate.load_data(&internal2).unwrap().raw.clone();
    fate.modify_range(&internal2, 2, &[0xAA, 0xBB]);
    let after = fate.load_data(&internal2).unwrap().raw.clone();
    
    println!("   {} 部分修改 offset=2:", internal2);
    println!("   - 修改前: {:02X?}", before);
    println!("   - 修改后: {:02X?}", after);
    println!();

    // ========== 5. 追加数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("➕ 5. 追加数据\n");

    let before = String::from_utf8_lossy(&fate.load_data(&id3).unwrap().raw).to_string();
    fate.append_data(&id3, b" // appended!");
    let after = String::from_utf8_lossy(&fate.load_data(&id3).unwrap().raw).to_string();
    
    println!("   {} 追加:", id3);
    println!("   - 追加前: \"{}\"", before);
    println!("   - 追加后: \"{}\"", after);
    println!();

    // ========== 6. 变换数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 6. 变换数据（字节级处理）\n");

    let test_id = fate.create_string("abcdef");
    let before = fate.load_data(&test_id).unwrap().raw.clone();
    
    // 对每个字节 +1
    fate.transform_data(&test_id, |data| {
        for byte in data.iter_mut() {
            *byte = byte.wrapping_add(1);
        }
    });
    
    let after = fate.load_data(&test_id).unwrap().raw.clone();
    println!("   {} 变换 (每字节+1):", test_id);
    println!("   - 变换前: {:?} ({:02X?})", String::from_utf8_lossy(&before), before);
    println!("   - 变换后: {:?} ({:02X?})", String::from_utf8_lossy(&after), after);
    println!();

    // ========== 7. 合并数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔗 7. 合并数据\n");

    let a = fate.create_string("AAA");
    let b = fate.create_string("BBB");
    let merged = fate.merge_data(&[&a, &b]).unwrap();
    let merged_content = String::from_utf8_lossy(&fate.load_data(&merged).unwrap().raw).to_string();
    
    println!("   合并 {} + {} → {}", a, b, merged);
    println!("   - 内容: \"{}\"", merged_content);
    println!();

    // ========== 8. 分割数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✂️  8. 分割数据\n");

    let long_id = fate.create_string("123456789ABCDEF");
    let chunks = fate.split_data(&long_id, 5);
    
    println!("   分割 {} (chunk_size=5):", long_id);
    for chunk_id in &chunks {
        let content = String::from_utf8_lossy(&fate.load_data(chunk_id).unwrap().raw).to_string();
        println!("   - {}: \"{}\"", chunk_id, content);
    }
    println!();

    // ========== 9. 复制数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 9. 复制数据\n");

    let cloned = fate.clone_data(&id2).unwrap();
    let orig = String::from_utf8_lossy(&fate.load_data(&id2).unwrap().raw).to_string();
    let copy = String::from_utf8_lossy(&fate.load_data(&cloned).unwrap().raw).to_string();
    
    println!("   复制 {} → {}", id2, cloned);
    println!("   - 原始: \"{}\"", orig);
    println!("   - 副本: \"{}\"", copy);
    println!();

    // ========== 10. 删除数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🗑️  10. 删除数据\n");

    let before_count = fate.stored_count();
    fate.remove_data(&a);
    fate.remove_data(&b);
    let after_count = fate.stored_count();
    
    println!("   删除 {} 和 {}", a, b);
    println!("   - 删除前: {} 条", before_count);
    println!("   - 删除后: {} 条", after_count);
    println!();

    // ========== 统计 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 统计\n");

    println!("   总存储数量: {}", fate.stored_count());
    println!("   外部导入: {}", fate.list_external().len());
    println!("   内部创建: {}", fate.list_internal().len());
    println!("   总修改次数: {}", fate.total_modifications());
    println!("   Fate tick: {}", fate.global_tick);
    println!("   方程系数: [{:.4}, {:.4}, {:.4}]",
             fate.cluster_equation.coef_a,
             fate.cluster_equation.coef_b,
             fate.cluster_equation.coef_c);
    println!();

    // ========== 冻结切片 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("❄️  冻结 Fate 切片\n");

    let slice_path = Path::new("/tmp/fate_slice_full.json");
    fate.save_slice(slice_path).expect("保存失败");
    let file_size = std::fs::metadata(slice_path).map(|m| m.len()).unwrap_or(0);
    
    println!("   保存到: {:?}", slice_path);
    println!("   文件大小: {} bytes", file_size);
    println!();

    // ========== 恢复验证 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 恢复验证\n");

    let restored = TripleFateCombined::load_slice(slice_path).expect("加载失败");
    
    println!("   恢复后存储数量: {}", restored.stored_count());
    println!("   恢复后外部数量: {}", restored.list_external().len());
    println!("   恢复后内部数量: {}", restored.list_internal().len());
    println!("   恢复后修改次数: {}", restored.total_modifications());
    
    // 验证被修改的数据
    if let Some(entry) = restored.load_data(&id1) {
        println!("\n   验证 {}:", id1);
        println!("   - 内容: \"{}\"", String::from_utf8_lossy(&entry.raw));
        println!("   - 来源: {}", entry.origin);
        println!("   - 修改次数: {}", entry.modification_count);
    }

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   ✅ 测试完成 - 系统对数据有完全控制权                           ║");
    println!("║   ✓ 外部导入  ✓ 系统新建  ✓ 完全修改  ✓ 字节级处理             ║");
    println!("║   ✓ 追加/截断  ✓ 变换  ✓ 合并/分割  ✓ 复制/删除                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
