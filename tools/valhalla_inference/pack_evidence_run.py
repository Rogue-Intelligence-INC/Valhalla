#!/usr/bin/env python3
"""Pack Valhalla benchmark Evidence Pack — suite · logs · legitimacy · hashes.

Uploadable proof for fundraising / independent reproduction:

  reports/valhalla_inference/evidence_<tag>/
    REPRODUCE.md
    MANIFEST.json
    HASHES.sha256
    legitimacy_audit.json
    COMPETITOR_COMPARE.md
    provenance/
    logs/          (selected run logs)
    reports/       (citeable JSON + summaries)
    suite_manifest.json  (bundled file list + sha)

Usage:
  python3 tools/valhalla_inference/pack_evidence_run.py
  python3 tools/valhalla_inference/pack_evidence_run.py --tag 20260713 --zip
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports" / "valhalla_inference"
BUNDLED = REPO / "tools" / "standard_benchmarks" / "bundled"
PROVENANCE = REPO / "tools" / "standard_benchmarks" / "provenance"
DOCS = REPO / "docs" / "valhalla"

sys.path.insert(0, str(HERE))

# Core reports to include when present
REPORT_GLOBS = [
    "BENCHMARK_SUMMARY_*.md",
    "EXPERIMENT_SUMMARY_*.md",
    "VALHALLA_INTEGRATED_STATUS.md",
    "FUNDRAISE_ABLATION_MATRIX.md",
    "EXTERNAL_PARETO_MATRIX.md",
    "FUNDRAISE_BENCHMARK_LATEST.md",
    "industry_benchmarks_summary.json",
    "fundraise_ablation_matrix.json",
    "external_pareto_matrix.json",
    "valhalla_integrated_status.json",
    "ragtruth_v1_test.json",
    "ragtruth_full_v1_test.json",
    "needle_session_v1_test.json",
    "locomo_v1_test.json",
    "multihop_rag_hybrid_test.json",
    "multihop_rag_v2_test.json",
    "multihop_rag_fate_test.json",
    "longbench_v2_v1_test.json",
    "ruler_v1_test.json",
    "lara_v1_test.json",
    "external_holdout_test.json",
    "token_efficiency_v1_test.json",
    "valhalla_base_training_potential_v2.json",
    "new_paradigm_potential_v1.json",
    "valhalla_independence_test.json",
    "competitor_compare.json",
    "COMPETITOR_COMPARE_RESULTS.md",
    "proactive_agency_benchmark.json",
]

LOG_PATTERNS = [
    "full_industry_run_*.log",
    "fundraise_ablation_run.log",
    "fundraise_run_*.log",
    "pareto_run_*.log",
]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def git_sha() -> str:
    try:
        return (
            subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def git_dirty() -> bool:
    try:
        out = subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True)
        return bool(out.strip())
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def run_legitimacy(out_path: Path) -> dict[str, Any]:
    proc = subprocess.run(
        [sys.executable, str(HERE / "audit_benchmark_legitimacy.py"), "--json"],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    doc: dict[str, Any]
    if proc.returncode == 0 and proc.stdout.strip():
        doc = json.loads(proc.stdout)
    else:
        doc = {"error": proc.stderr or "audit failed", "returncode": proc.returncode}
    out_path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    # also mirror into reports/
    REPORTS.mkdir(parents=True, exist_ok=True)
    (REPORTS / "legitimacy_audit.json").write_text(
        json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    return doc


def suite_manifest() -> dict[str, Any]:
    files: list[dict[str, Any]] = []
    if BUNDLED.is_dir():
        for p in sorted(BUNDLED.glob("*.json")):
            files.append(
                {
                    "path": str(p.relative_to(REPO)),
                    "sha256": sha256_file(p),
                    "bytes": p.stat().st_size,
                }
            )
    return {
        "bundled_dir": str(BUNDLED.relative_to(REPO)),
        "n_files": len(files),
        "files": files,
    }


def copy_provenance(dest: Path) -> list[str]:
    dest.mkdir(parents=True, exist_ok=True)
    copied: list[str] = []
    if not PROVENANCE.is_dir():
        return copied
    for p in sorted(PROVENANCE.glob("*.json")):
        shutil.copy2(p, dest / p.name)
        copied.append(p.name)
    return copied


def collect_reports(dest: Path) -> list[str]:
    dest.mkdir(parents=True, exist_ok=True)
    names: list[str] = []
    for pattern in REPORT_GLOBS:
        if "*" in pattern:
            for p in sorted(REPORTS.glob(pattern)):
                if p.is_file():
                    shutil.copy2(p, dest / p.name)
                    names.append(p.name)
        else:
            p = REPORTS / pattern
            if p.is_file():
                shutil.copy2(p, dest / p.name)
                names.append(p.name)
    return names


def collect_logs(dest: Path, *, max_logs: int = 8) -> list[str]:
    dest.mkdir(parents=True, exist_ok=True)
    found: list[Path] = []
    for pattern in LOG_PATTERNS:
        found.extend(REPORTS.glob(pattern))
    # newest first
    found = sorted({p.resolve() for p in found if p.is_file()}, key=lambda p: p.stat().st_mtime, reverse=True)
    names: list[str] = []
    for p in found[:max_logs]:
        shutil.copy2(p, dest / p.name)
        names.append(p.name)
    status = REPORTS / "fundraise_ablation.status"
    if status.is_file():
        shutil.copy2(status, dest / status.name)
        names.append(status.name)
    return names


def write_hashes(root: Path) -> Path:
    lines: list[str] = []
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        if p.name == "HASHES.sha256":
            continue
        rel = p.relative_to(root)
        lines.append(f"{sha256_file(p)}  {rel.as_posix()}")
    out = root / "HASHES.sha256"
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out


def write_reproduce(root: Path, manifest: dict[str, Any]) -> None:
    text = f"""# Valhalla Evidence Pack — Reproduce

**Tag:** `{manifest['tag']}`  
**Git SHA:** `{manifest['git_sha']}`  
**Packed:** {manifest['timestamp']}  
**Backbone (industry calibration):** `{manifest.get('backbone', 'Qwen/Qwen2.5-3B-Instruct')}`

## What this pack proves

1. **Suite** — bundled industry slices + runner scripts (see `suite_manifest.json`)
2. **Logs** — timestamped execution traces under `logs/`
3. **Reports** — machine-readable JSON under `reports/`
4. **Legitimacy** — `legitimacy_audit.json` marks citeable vs Tier-C (non-citeable)
5. **Provenance** — upstream dataset citations under `provenance/`
6. **Hashes** — `HASHES.sha256` for integrity

## One-shot industry suite (from repo root)

```bash
export RUSTFLAGS="-L /opt/cuda/lib64"
export LIBRARY_PATH="/opt/cuda/lib64:${{LIBRARY_PATH:-}}"
export HF_ENDPOINT="${{HF_ENDPOINT:-https://hf-mirror.com}}"
export VALHALLA_BASE_NO_SERVE=1
cargo build -p hub-f64 --release --bin valhalla_base
.venv-llm/bin/python3 tools/valhalla_inference/run_industry_benchmarks.py --all --fresh --verbose
.venv-llm/bin/python3 tools/valhalla_inference/audit_benchmark_legitimacy.py --json
.venv-llm/bin/python3 tools/valhalla_inference/pack_evidence_run.py --tag reproduce
```

## Ablation + competitor baselines

```bash
./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only
.venv-llm/bin/python3 tools/valhalla_inference/run_competitor_compare.py --moderate
```

## Citeability rule

- Only reports with `citeable=true` in `legitimacy_audit.json` should appear on pitch homepage.
- FRAMES: run `./scripts/prefetch_frames_wiki.sh` (parasail HF wiki dump) before citing; without extracts = Tier C.
- TPI / NPPI are **mechanism** metrics (secondary), not industry SOTA claims.

## Competitor compare policy

See `COMPETITOR_COMPARE.md` in this pack.
"""
    (root / "REPRODUCE.md").write_text(text, encoding="utf-8")


def write_competitor_md(root: Path) -> None:
    src = DOCS / "COMPETITOR_COMPARE.md"
    if src.is_file():
        shutil.copy2(src, root / "COMPETITOR_COMPARE.md")
    else:
        (root / "COMPETITOR_COMPARE.md").write_text(
            "# Competitor compare — see docs/valhalla/COMPETITOR_COMPARE.md\n",
            encoding="utf-8",
        )


def make_zip(root: Path) -> Path:
    archive = shutil.make_archive(str(root), "zip", root.parent, root.name)
    return Path(archive)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", default="", help="pack tag (default: UTC date)")
    ap.add_argument("--zip", action="store_true", help="also create .zip")
    ap.add_argument("--max-logs", type=int, default=8)
    args = ap.parse_args()

    tag = args.tag or datetime.now(timezone.utc).strftime("%Y%m%d")
    out_root = REPORTS / f"evidence_{tag}"
    if out_root.exists():
        shutil.rmtree(out_root)
    out_root.mkdir(parents=True)

    print(f"Packing evidence → {out_root}", flush=True)

    legitimacy = run_legitimacy(out_root / "legitimacy_audit.json")
    suite = suite_manifest()
    (out_root / "suite_manifest.json").write_text(
        json.dumps(suite, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    prov = copy_provenance(out_root / "provenance")
    reports = collect_reports(out_root / "reports")
    logs = collect_logs(out_root / "logs", max_logs=args.max_logs)

    # Pointer to suite (do not duplicate multi-MB bundles into pack by default —
    # hashes in suite_manifest are the integrity proof; optional --copy-bundles later)
    (out_root / "suite_README.txt").write_text(
        "Bundled datasets live in repo at tools/standard_benchmarks/bundled/.\n"
        "SHA256 of each file is listed in suite_manifest.json.\n"
        "Clone the matching git SHA and verify hashes before citing scores.\n",
        encoding="utf-8",
    )

    backbone = os.environ.get("VALHALLA_BENCHMARK_LM_MODEL") or os.environ.get(
        "VALHALLA_LM_MODEL", "Qwen/Qwen2.5-3B-Instruct"
    )
    manifest: dict[str, Any] = {
        "protocol": "valhalla-evidence-pack-v1",
        "tag": tag,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "git_sha": git_sha(),
        "git_dirty": git_dirty(),
        "backbone": backbone,
        "cuda_device": os.environ.get("VALHALLA_CUDA_DEVICE", "0"),
        "contents": {
            "provenance_files": prov,
            "reports": reports,
            "logs": logs,
            "bundled_hashed": suite["n_files"],
            "legitimacy_citeable": (legitimacy.get("summary") or {}).get("reports_citeable"),
            "legitimacy_total": (legitimacy.get("summary") or {}).get("reports_total"),
        },
        "upload_hint": (
            "Attach this directory or the .zip to GitHub Release / data room. "
            "Share REPRODUCE.md + legitimacy_audit.json first."
        ),
    }
    (out_root / "MANIFEST.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    write_reproduce(out_root, manifest)
    write_competitor_md(out_root)
    write_hashes(out_root)

    print(f"  provenance: {len(prov)}")
    print(f"  reports:    {len(reports)}")
    print(f"  logs:       {len(logs)}")
    print(
        f"  legitimacy: "
        f"{manifest['contents']['legitimacy_citeable']}/{manifest['contents']['legitimacy_total']} citeable"
    )
    print(f"  git:        {manifest['git_sha'][:12]} dirty={manifest['git_dirty']}")

    if args.zip:
        z = make_zip(out_root)
        print(f"  zip:        {z}")

    print(f"\n→ {out_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
