//! 五轮轻测试 + 新范式下能测的几件事
//!
//! 旧问答测法在小规模下只测背诵，这里测结构本身：
//! 留存（喂过的在四元里找得到）、沉默（没喂过的不出现）、
//! 稳定（同一问法两次找到同一段）、干预（改/删一段后身体状态跟着变）、
//! 结构（连接和连通块在训练中形成）。
//! 这些通过也不等于智能，只说明通道和结构在工作。

use hub_f64::monitor::BodyMonitor;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;

fn find_in(stored: &[Vec<u8>], needle: &[u8]) -> Option<usize> {
    stored.iter().position(|item| item.windows(needle.len()).any(|w| w == needle))
}

fn count_in(stored: &[Vec<u8>], needle: &[u8]) -> usize {
    stored.iter().filter(|item| item.windows(needle.len()).any(|w| w == needle)).count()
}

fn main() {
    let text_a: Vec<u8> = "The quick brown fox jumps over the lazy dog near the river bank. ".repeat(40).into_bytes();
    let text_b: Vec<u8> = "錐刀既無棄，轉化日已盈。立身多戶門，何必燕山銘。".repeat(40).into_bytes();
    let unfed: &[u8] = "量子纠缠如何用于纠错码".as_bytes();

    let mut stem = StemCellComplete::new();
    let mut tile = TileQuattroComplete::new();
    let mut monitor = BodyMonitor::new();

    println!("== 五轮轻训练 ==");
    for round in 1..=5 {
        for text in [&text_a, &text_b] {
            for window in text.chunks(512) {
                tile.quad_keep(window);
                stem.quad_keep(window);
                let mean = window.iter().map(|b| *b as f64).sum::<f64>() / window.len() as f64;
                stem.receive_data(mean);
                tile.ingest(mean);
                tile.promote_from_fate();
            }
        }
        let tile_cycle = tile.process_tiles_cycle();
        let stem_cycle = stem.process_cells_cycle();
        stem.settle_field();
        tile.propagate(1.0);
        monitor.observe_cycle(&tile_cycle, &stem_cycle);
        println!("round {round} {}", monitor.snapshot(&tile, &stem).line());
    }

    println!("\n== 留存与沉默 ==");
    let fox: &[u8] = b"quick brown fox";
    let poem: &[u8] = "錐刀既無棄".as_bytes();
    let in_tile = find_in(&tile.blackhole.stored, fox);
    let in_stem = find_in(&stem.multinova.stored, poem);
    println!("Tile 黑洞里找到 fox: {in_tile:?}");
    println!("Stem 新星里找到诗句: {in_stem:?}");
    assert!(in_tile.is_some() && in_stem.is_some());

    let mut unfed_hits = 0usize;
    for stored in [
        &tile.blackhole.stored, &tile.meshbrain.stored, &tile.multinova.stored, &tile.baseforce.stored,
        &stem.blackhole.stored, &stem.meshbrain.stored, &stem.multinova.stored, &stem.baseforce.stored,
    ] {
        unfed_hits += count_in(stored, unfed);
    }
    println!("没喂过的量子句在八份四元存储里出现: {unfed_hits}");
    assert_eq!(unfed_hits, 0);

    println!("\n== 稳定 ==");
    let first = find_in(&tile.blackhole.stored, fox);
    let second = find_in(&tile.blackhole.stored, fox);
    println!("两次找到同一段: {first:?} / {second:?}");
    assert_eq!(first, second);

    println!("\n== 干预：改一段、删一段，身体跟着变 ==");
    tile.quad_keep(b"UNIQUE-MARKER-12345");
    let marker_at = find_in(&tile.blackhole.stored, b"UNIQUE-MARKER-12345");
    println!("标记段位置: {marker_at:?}");
    let marker_at = marker_at.expect("标记段应当在");
    assert!(tile.blackhole.stored_modify(marker_at, b"REPLACED-CONTENT"));
    let marker_gone = find_in(&tile.blackhole.stored, b"UNIQUE-MARKER-12345").is_none();
    let replaced_at = find_in(&tile.blackhole.stored, b"REPLACED-CONTENT");
    println!("改后旧标记消失: {marker_gone} 新内容位置: {replaced_at:?}");
    assert!(marker_gone && replaced_at.is_some());

    let copies_before = count_in(&stem.blackhole.stored, poem);
    let mut removed = 0usize;
    while let Some(at) = find_in(&stem.blackhole.stored, poem) {
        stem.blackhole.stored_remove(at);
        removed += 1;
    }
    let copies_after = count_in(&stem.blackhole.stored, poem);
    let tile_copies = count_in(&tile.blackhole.stored, poem);
    println!("Stem 黑洞诗句段 {copies_before} -> {copies_after}（删了 {removed}），Tile 黑洞仍有 {tile_copies}");
    assert_eq!(copies_after, 0);
    assert!(tile_copies > 0);

    println!("\n== 结构 ==");
    let snap = monitor.snapshot(&tile, &stem);
    println!("{}", snap.line());
    assert!(snap.tile_edges > 0 || snap.stem_links > 0);
    println!("连接或连通块已经形成: tile_edges {} stem_links {} 最大 Stem 块 {}", snap.tile_edges, snap.stem_links, snap.largest_stem_block());

    println!("\n轻测试完成。以上通过只说明留存、沉默、稳定、干预敏感和结构形成，不等于智能。");
}
