//! 检查 Fate 状态

use std::path::Path;
use stem_cell::StemCellComplete;

fn main() {
    let slice_path = Path::new("/tmp/massive_stemcell.stemcell");
    let stemcell = match StemCellComplete::load_slice(slice_path) {
        Ok(s) => s,
        Err(e) => {
            println!("加载失败: {}", e);
            return;
        }
    };

    println!("Fate 状态:");
    println!("  global_tick: {}", stemcell.triple_fate.global_tick);
    println!("  tick_count (tick()调用次数): {}", stemcell.triple_fate.tick_count);
    println!("  base.discovery_count: {}", stemcell.triple_fate.base.state.discovery_count);
    println!("  base.signal_count: {}", stemcell.triple_fate.base.state.signal_count);
    println!("  base.efficiency: {}", stemcell.triple_fate.base.state.efficiency);
    println!("  meta.discovery_count: {}", stemcell.triple_fate.meta.state.discovery_count);
    println!("  cluster_equation.triggered: {}", stemcell.triple_fate.cluster_equation.triggered);
    println!("  spontaneous_create_count: {}", stemcell.triple_fate.spontaneous_create_count);
    println!();
    
    // 统计内部创建
    let mut internal = 0;
    let mut external = 0;
    for id in stemcell.triple_fate.list_stored() {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            if entry.origin == "internal" {
                internal += 1;
                println!("  内部创建: {} -> {}", id, String::from_utf8_lossy(&entry.raw[..50.min(entry.raw.len())]));
            } else {
                external += 1;
            }
        }
    }
    println!();
    println!("  内部创建: {}", internal);
    println!("  外部导入: {}", external);
}
