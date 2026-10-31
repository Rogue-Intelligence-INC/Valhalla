//! 同一份材料两条路：拆开后组成一个新模型，或融进同一个模型。
//! 看交回来的是不是身体自己的话。分数和块数增加不算反证。

use hub_f64::free_multimodal_fate::mix_ports_weighted;
use hub_f64::grinder::grind_text_hash;
use hub_f64::model_fate::{run_model_fate_pipeline, ModelDecomposePlan, ModelDecomposeShard, MODEL_DECOMPOSE_PROTOCOL};
use hub_f64::{BodyTarget, IngressBudget, TriadSession};

fn shard(label: &str, text: &str) -> ModelDecomposeShard {
    ModelDecomposeShard {
        label: label.to_string(),
        body: "triad".to_string(),
        phase: "decompose".to_string(),
        ports: grind_text_hash(text, 32),
    }
}

fn plan(name: &str, texts: &[(&str, &str)]) -> ModelDecomposePlan {
    let shards: Vec<_> = texts.iter().map(|(label, text)| shard(label, text)).collect();
    let refs: Vec<_> = shards.iter().map(|item| &item.ports).collect();
    let weights = vec![1.0 / refs.len() as f64; refs.len()];
    let whole = if refs.len() == 1 {
        refs[0].clone()
    } else {
        mix_ports_weighted(&refs, &weights)
    };
    ModelDecomposePlan {
        protocol: MODEL_DECOMPOSE_PROTOCOL.to_string(),
        base_model: name.to_string(),
        layer_count: shards.len(),
        whole_ports: whole,
        shards,
    }
}

fn cosine(left: &[f64], right: &[f64]) -> f64 {
    let n = left.len().min(right.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut ll = 0.0;
    let mut rr = 0.0;
    for i in 0..n {
        dot += left[i] * right[i];
        ll += left[i] * left[i];
        rr += right[i] * right[i];
    }
    if ll == 0.0 || rr == 0.0 {
        0.0
    } else {
        dot / (ll.sqrt() * rr.sqrt())
    }
}

fn run_arm(name: &str, texts: &[(&str, &str)]) -> (usize, usize, f64, f64, Vec<f64>, String) {
    let mut session = TriadSession::with_body(IngressBudget::incubation(), BodyTarget::Triad);
    let outcome = run_model_fate_pipeline(&mut session, &plan(name, texts), name, 1);
    let report = &outcome.report;
    (
        report.tile_completed,
        report.stem_cells,
        outcome.decompose_score,
        outcome.model_closure_score,
        report.model_patch.patch_vector.clone(),
        outcome.stopped_reason,
    )
}

fn main() {
    let poem = "錐刀既無棄，轉化日已盈";
    let sum = "6 * 8 = <<6*8=48>>48 orange crayons";
    let (new_tiles, new_cells, new_score, new_closure, new_patch, new_stop) = run_arm("新模型", &[("诗", poem)]);
    let (fuse_tiles, fuse_cells, fuse_score, fuse_closure, fuse_patch, fuse_stop) =
        run_arm("融合模型", &[("诗", poem), ("算式", sum)]);
    let alike = cosine(&new_patch, &fuse_patch);
    println!(
        "{}",
        serde_json::json!({
            "对照": "拆解后组新模型 vs 融进一个模型",
            "材料": [poem, sum],
            "新模型": {
                "瓦片": new_tiles,
                "干细胞": new_cells,
                "拆解分": new_score,
                "闭合分": new_closure,
                "补丁维": new_patch.len(),
                "停": new_stop,
            },
            "融合模型": {
                "瓦片": fuse_tiles,
                "干细胞": fuse_cells,
                "拆解分": fuse_score,
                "闭合分": fuse_closure,
                "补丁维": fuse_patch.len(),
                "停": fuse_stop,
            },
            "两路补丁余弦": alike,
            "交回正文": false,
            "反证培育": false,
        })
    );
}
