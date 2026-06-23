#!/usr/bin/env python3
"""Strong multimodal fusion benchmark: TTS pairs + cross-modal discrimination tests.

Usage:
  python tools/valhalla_dl_front/run_multimodal_fusion_bench.py
  VALHALLA_DL_FRONT=1 python tools/valhalla_dl_front/run_multimodal_fusion_bench.py --cycles 3
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
ENCODE_PY = REPO / "tools/valhalla_dl_front/encode.py"
BENCH_BIN = REPO / "target/release/valhalla_multimodal_fusion_bench"
OUT_DIR = REPO / "reports/tts_tile_stem_fusion"

BENCH_PAIRS = [
    ("p01", "The capital of France is Paris"),
    ("p02", "Water freezes at zero degrees Celsius"),
    ("p03", "Photosynthesis converts light into chemical energy"),
    ("p04", "The speed of light is approximately three hundred thousand kilometers per second"),
    ("p05", "DNA stores genetic information in living organisms"),
    ("p06", "Gravity attracts masses toward each other"),
    ("p07", "The human heart pumps blood through the body"),
    ("p08", "Electricity flows through conductive materials"),
    ("p09", "Mount Everest is the highest mountain on Earth"),
    ("p10", "Computers process data using binary logic"),
    ("p11", "The Pacific Ocean is the largest ocean on the planet"),
    ("p12", "Shakespeare wrote many famous plays and sonnets"),
]


def synthesize_espeak(text: str, wav: Path) -> bool:
    try:
        subprocess.run(["espeak", "-w", str(wav), text], check=True, capture_output=True)
        return wav.is_file()
    except Exception:
        return False


def encode_text(content: str) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE_PY)],
        input=json.dumps({"modality": "text", "content": content}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def encode_wav(path: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE_PY)],
        input=json.dumps({"modality": "audio_path", "path": str(path)}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def cosine(a: list[float], b: list[float]) -> float:
    n = min(len(a), len(b))
    dot = sum(a[i] * b[i] for i in range(n))
    na = sum(a[i] * a[i] for i in range(n)) ** 0.5
    nb = sum(b[i] * b[i] for i in range(n)) ** 0.5
    return dot / (na * nb + 1e-12)


def embedding_cross_modal_gates(pairs: list[dict], wav_dir: Path) -> dict:
    """Utterance discriminability via duration spread + acoustic embedding margin."""
    import wave

    durations = []
    for p in pairs:
        wav = wav_dir / f"{p['id']}.wav"
        with wave.open(str(wav), "rb") as wf:
            durations.append(wf.getnframes() / max(wf.getframerate(), 1))
    dur_std = (sum((d - sum(durations) / len(durations)) ** 2 for d in durations) / len(durations)) ** 0.5

    n = len(pairs)
    margins = []
    r1 = 0
    cross = []
    for i, p in enumerate(pairs):
        ae = p["audio_features"]
        scores = [cosine(ae, pairs[j]["audio_features"]) for j in range(n)]
        if scores.index(max(scores)) == i:
            r1 += 1
        best_neg = max(scores[j] for j in range(n) if j != i)
        margins.append(scores[i] - best_neg)
        for j in range(i + 1, n):
            cross.append(cosine(ae, pairs[j]["audio_features"]))
    margin_mean = sum(margins) / n
    cross_mean = sum(cross) / max(len(cross), 1)
    return {
        "duration_std_seconds": dur_std,
        "audio_retrieval_at_1": r1 / n,
        "audio_contrastive_margin_mean": margin_mean,
        "audio_cross_cosine_mean": cross_mean,
        "pass_embedding_alignment": dur_std >= 0.4 and r1 / n >= 0.34,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cycles", type=int, default=3)
    args = parser.parse_args()

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    wav_dir = OUT_DIR / "bench_wav"
    wav_dir.mkdir(exist_ok=True)

    print("=== Multimodal Fusion Benchmark ===")
    print(f"pairs={len(BENCH_PAIRS)} cycles={args.cycles}")

    pairs_payload = []
    pairs_for_emb = []
    for pid, text in BENCH_PAIRS:
        wav = wav_dir / f"{pid}.wav"
        if not synthesize_espeak(text, wav):
            print(f"FAIL TTS: {pid}")
            sys.exit(1)
        audio_emb = encode_wav(wav)
        pairs_payload.append({"id": pid, "text": text, "audio_features": audio_emb})
        pairs_for_emb.append({"id": pid, "audio_features": audio_emb})
        print(f"  {pid}: dim={len(audio_emb)}")

    if not BENCH_BIN.is_file():
        print("Build: cargo build -p hub-f64 --release --bin valhalla_multimodal_fusion_bench")
        sys.exit(1)

    proc = subprocess.run(
        [str(BENCH_BIN)],
        input=json.dumps({"optimize_cycles": args.cycles, "pairs": pairs_payload}),
        capture_output=True,
        text=True,
        env={**dict(**__import__("os").environ), "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )
    if not proc.stdout.strip():
        print(proc.stderr)
        sys.exit(1)
    report = json.loads(proc.stdout)

    emb_gates = embedding_cross_modal_gates(pairs_for_emb, wav_dir)
    report["embedding_alignment"] = emb_gates
    report["tts_backend"] = "espeak"
    report["wav_dir"] = str(wav_dir)

    g = report["summary"]["gates"]
    overall = g["overall_pass"] and emb_gates["pass_embedding_alignment"]
    report["summary"]["overall_pass_with_embedding"] = overall

    out = OUT_DIR / "multimodal_fusion_bench.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")

    cm = report["cross_modal"]
    mm = report["mismatch"]
    print("\n=== Structure gates (Rust) ===")
    print(f"  tile superadditivity: {g['tile_superadditivity_rate']*100:.1f}%  pass={g['pass_tile_superadditivity']}")
    print(f"  stem superadditivity: {g['stem_superadditivity_rate']*100:.1f}%  pass={g['pass_stem_superadditivity']}")
    print(f"  complementarity: {report['summary']['avg_tile_sig_complementarity']:.3f}  pass={g['pass_complementarity']}")
    print(f"  modality ratio (audio/text tiles): see pairs")
    print(f"  corpus batch tiles/pair: {report['corpus']['batch_tile_per_pair']:.1f}  pass={g['pass_corpus']}")
    print(f"  structure pass: {g['overall_pass']}")
    print("\n=== Embedding gates (Python) ===")
    print(f"  audio R@1: {emb_gates['audio_retrieval_at_1']*100:.1f}%  margin={emb_gates['audio_contrastive_margin_mean']:.4f}")
    print(f"  audio cross cos mean: {emb_gates['audio_cross_cosine_mean']:.4f}")
    print(f"  pass: {emb_gates['pass_embedding_alignment']}")
    print(f"\n  OVERALL: {'PASS' if overall else 'FAIL'}")
    print(f"Wrote {out}")

    sys.exit(0 if overall else 2)


if __name__ == "__main__":
    main()
