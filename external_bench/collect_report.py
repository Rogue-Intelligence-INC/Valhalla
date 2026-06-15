#!/usr/bin/env python3
import csv
import json
import os
import re
import sys
from pathlib import Path


def parse_yaksha_aead_bench(stdout_text: str):
    # Example line:
    #   4 KB │          10 │      401754 │           9 │      414272
    row_re = re.compile(
        r"^\s*(?P<size_num>\d+)\s*(?P<size_unit>KB|MB)\s*│\s*"
        r"(?P<seal_mb>[0-9]+(?:\.[0-9]+)?)\s*│\s*"
        r"(?P<seal_ns>[0-9]+(?:\.[0-9]+)?)\s*│\s*"
        r"(?P<open_mb>[0-9]+(?:\.[0-9]+)?)\s*│\s*"
        r"(?P<open_ns>[0-9]+(?:\.[0-9]+)?)\s*$"
    )

    rows = []
    for line in stdout_text.splitlines():
        m = row_re.match(line)
        if not m:
            continue

        size_num = int(m.group("size_num"))
        size_unit = m.group("size_unit")
        size_bytes = size_num * 1024 * 1024 if size_unit == "MB" else size_num * 1024

        rows.append(
            {
                "size": f"{size_num} {size_unit}",
                "size_bytes": size_bytes,
                "seal_mb_s": float(m.group("seal_mb")),
                "seal_ns_op": float(m.group("seal_ns")),
                "open_mb_s": float(m.group("open_mb")),
                "open_ns_op": float(m.group("open_ns")),
            }
        )

    return rows


def read_exit_code(path: Path):
    if not path.exists():
        return None
    try:
        return int(path.read_text().strip())
    except Exception:
        return None


def main():
    if len(sys.argv) != 2:
        print("Usage: collect_report.py <out_dir>", file=sys.stderr)
        sys.exit(2)

    out_dir = Path(sys.argv[1])
    if not out_dir.exists():
        raise SystemExit(f"out_dir not found: {out_dir}")

    variant_path = out_dir / "bench_variant.txt"
    bench_variant = (
        variant_path.read_text(errors="ignore").strip()
        if variant_path.exists()
        else None
    )

    # ---- Yaksha AEAD bench ----
    yaksha_stdout = (out_dir / "yaksha_aead_bench.stdout").read_text(errors="ignore")
    yaksha_rows = parse_yaksha_aead_bench(yaksha_stdout)

    csv_path = out_dir / "yaksha_aead_bench.csv"
    with csv_path.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["size", "size_bytes", "seal_mb_s", "seal_ns_op", "open_mb_s", "open_ns_op"])
        for r in yaksha_rows:
            w.writerow(
                [
                    r["size"],
                    r["size_bytes"],
                    f'{r["seal_mb_s"]:.6g}',
                    f'{r["seal_ns_op"]:.6g}',
                    f'{r["open_mb_s"]:.6g}',
                    f'{r["open_ns_op"]:.6g}',
                ]
            )

    def fmt_exit(code):
        return "n/a" if code is None else str(code)

    yaksha_exit = {
        "yaksha_aead_bench_exit": read_exit_code(out_dir / "yaksha_aead_bench.exit"),
        "yaksha_test_exit": read_exit_code(out_dir / "yaksha_test.exit"),
        "yaksha_test_gpu_exit": read_exit_code(out_dir / "yaksha_test_gpu.exit"),
        "yaksha_aead_timing_keys_exit": read_exit_code(out_dir / "yaksha_aead_timing_keys.exit"),
        "dieharder_stream_gpu_generate_exit": read_exit_code(
            out_dir / "dieharder_stream_gpu.generate.exit"
        ),
    }

    # ---- OpenSSL outputs (store only metadata) ----
    openssl_files = sorted(out_dir.glob("openssl_aead_*.mr"))
    openssl_meta = {}
    for p in openssl_files:
        txt = p.read_text(errors="ignore")
        # Extract the last +F: line (throughput summary in machine-readable format)
        last_f = None
        for line in txt.splitlines():
            if line.startswith("+F:"):
                last_f = line
        openssl_meta[p.name] = {"last_plus_F": last_f}

    # ---- dieharder outputs ----
    dieharder_files = sorted(out_dir.glob("*dieharder*.stdout"))
    dieharder_meta = {}
    for p in dieharder_files:
        dieharder_meta[p.name] = p.read_text(errors="ignore")[-4000:]

    report = {
        "out_dir": str(out_dir),
        "yaksha_aead_bench_parsed_rows": yaksha_rows,
        "yaksha_exit_codes": yaksha_exit,
        "openssl_meta": openssl_meta,
        "dieharder_meta_keys": list(dieharder_meta.keys()),
    }
    (out_dir / "report.json").write_text(json.dumps(report, indent=2))

    # ---- Markdown summary ----
    md = out_dir / "report_summary.md"
    with md.open("w") as f:
        f.write(f"# External Benchmark Summary\n\n")
        f.write(f"Results dir: `{out_dir}`\n\n")
        if bench_variant:
            f.write(f"**Variant:** `{bench_variant}` (`bench_variant.txt`)\n\n")
        f.write("## Yaksha\n\n")
        f.write(f"- `yaksha_aead_bench.exit`: `{fmt_exit(yaksha_exit['yaksha_aead_bench_exit'])}`\n")
        f.write(f"- `yaksha_test.exit` (CPU smoke): `{fmt_exit(yaksha_exit['yaksha_test_exit'])}`\n")
        f.write(
            f"- `yaksha_test_gpu.exit` (when GPU crate ran): `{fmt_exit(yaksha_exit['yaksha_test_gpu_exit'])}`\n"
        )
        f.write(
            f"- `dieharder_stream_gpu.generate.exit` (GPU stream gen): `{fmt_exit(yaksha_exit['dieharder_stream_gpu_generate_exit'])}`\n"
        )
        f.write(f"- `yaksha_aead_timing_keys.exit`: `{fmt_exit(yaksha_exit['yaksha_aead_timing_keys_exit'])}`\n\n")

        f.write("### AEAD throughput table (parsed)\n\n")
        f.write("| Size | seal MB/s | seal ns/op | open MB/s | open ns/op |\n")
        f.write("|---|---:|---:|---:|---:|\n")
        for r in yaksha_rows:
            f.write(
                f"| {r['size']} | {r['seal_mb_s']:.3g} | {r['seal_ns_op']:.3g} | {r['open_mb_s']:.3g} | {r['open_ns_op']:.3g} |\n"
            )
        f.write("\n")

        f.write("## OpenSSL AEAD baselines\n\n")
        f.write("The harness stores the last `+F:` machine-readable throughput line.\n\n")
        for name, meta in openssl_meta.items():
            f.write(f"- `{name}`: `{meta.get('last_plus_F')}`\n")

        f.write("\n")
        f.write("## dieharder randomness tests\n\n")
        f.write("STS gate + info logs: `dieharder_*.stdout` (single stream per run; CPU vs GPU is determined by `bench_variant.txt`).\n")


if __name__ == "__main__":
    main()

