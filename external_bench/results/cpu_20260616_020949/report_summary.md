# External Benchmark Summary

Results dir: `/home/jouly/AI/Valhalla/external_bench/results/cpu_20260616_020949`

**Variant:** `cpu` (`bench_variant.txt`)

## Yaksha

- `yaksha_aead_bench.exit`: `0`
- `yaksha_test.exit` (CPU smoke): `0`
- `yaksha_test_gpu.exit` (when GPU crate ran): `n/a`
- `dieharder_stream_gpu.generate.exit` (GPU stream gen): `n/a`
- `yaksha_aead_timing_keys.exit`: `0`

### AEAD throughput table (parsed)

| Size | seal MB/s | seal ns/op | open MB/s | open ns/op |
|---|---:|---:|---:|---:|
| 4 KB | 15 | 2.54e+05 | 11 | 3.54e+05 |
| 64 KB | 27 | 2.3e+06 | 19 | 3.3e+06 |
| 256 KB | 33 | 7.54e+06 | 31 | 8.05e+06 |
| 1 MB | 43 | 2.3e+07 | 41 | 2.44e+07 |
| 4 MB | 60 | 6.72e+07 | 60 | 6.67e+07 |
| 16 MB | 60 | 2.66e+08 | 62 | 2.56e+08 |

## OpenSSL AEAD baselines

The harness stores the last `+F:` machine-readable throughput line.

- `openssl_aead_aes-128-gcm.mr`: `+F:25:AES-128-GCM:16377485.17:223232405.71:902724515.43:4612302952.55:9294437374.97:10239730763.93`
- `openssl_aead_aes-256-gcm.mr`: `+F:25:AES-256-GCM:15823051.50:218722355.66:887915819.82:4291292790.39:8203727106.31:8851874629.26`
- `openssl_aead_chacha20-poly1305.mr`: `+F:25:ChaCha20-Poly1305:203362557.72:680058487.69:1005257055.46:3407125371.77:3436915020.44:3500494901.30`

## dieharder randomness tests

STS gate + info logs: `dieharder_*.stdout` (single stream per run; CPU vs GPU is determined by `bench_variant.txt`).
