# Hybrid 3.1 Rust Full — End-to-End Report (2026-06-23)

**Product**: `hybrid-3.1-rust-full`  
**Binary**: `target/release/valhalla_hybrid` (feature `patch-lm`)  
**Mode**: 全 Rust — Tier A patch LM + Tier B memory + merge，无 Python

## Architecture

```
Corpus → patch_export (TriadSession in-process)
              ↓
     patch_lm (candle Qwen2 + legacy weight surgery + greedy decode)
              ↓
     hybrid (triad + tile_fate memory + deploy/oracle merge)
```

| Module | Role |
|--------|------|
| `patch_export.rs` | 结构导出（替代 subprocess `valhalla_model_export`） |
| `patch_lm.rs` | Qwen2.5-0.5B legacy patch + 生成 |
| `hybrid.rs` | 路由/合并 |
| `grinder/fair_score/fair_suite` | Fair benchmark 基础设施 |

## Test results (145Q deploy)

| Track | Full Rust | Reuse Python patch JSON |
|-------|-----------|-------------------------|
| **Hybrid deploy** | **80.00%** (116/145) | 77.24% (112/145) |
| open | 95.08% | 95.08% |
| mcq | 52.17% | 52.17% |
| numeric | **89.47%** | 78.95% |
| patch_lm subset | **92.93%** (92/99) | 87.88% (87/99) |

Smoke (12Q): **75.00%** (9/12), ~22s CPU

## Run

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_hybrid

# 全 Rust（默认 --run-patch）
./target/release/valhalla_hybrid --phase test --merge-mode deploy --cpu

# 复用已有 patch JSON
./target/release/valhalla_hybrid --phase test --patch-json reports/.../hybrid_unified_*.json
```

Env: `VALHALLA_QWEN_MODEL_DIR` 指向本地 Qwen snapshot（含 `model.safetensors`）

## Notes

- Patch LM 当前 CPU（candle `cudarc` 暂不支持 CUDA 13.3）；145Q 全量约 9 分钟
- Patch 模式：`legacy`（与 fair benchmark 一致）
- Tier A 仍依赖 HuggingFace 权重文件；Valhalla 结构导出与 patch 逻辑已 100% Rust
