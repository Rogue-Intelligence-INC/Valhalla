# External Benchmark Summary

Results dir: `/home/jouly/AI/Valhalla/external_bench/results/gpu_20260616_021357`

**Variant:** `gpu` (`bench_variant.txt`)

## Yaksha

- `yaksha_aead_bench.exit`: `0`
- `yaksha_test.exit` (CPU smoke): `n/a`
- `yaksha_test_gpu.exit` (when GPU crate ran): `0`
- `dieharder_stream_gpu.generate.exit` (GPU stream gen): `0`
- `yaksha_aead_timing_keys.exit`: `0`

### AEAD throughput table (parsed)

| Size | seal MB/s | seal ns/op | open MB/s | open ns/op |
|---|---:|---:|---:|---:|
| 4 KB | 16 | 2.43e+05 | 17 | 2.31e+05 |
| 64 KB | 15 | 4.04e+06 | 17 | 3.69e+06 |
| 256 KB | 29 | 8.53e+06 | 25 | 9.85e+06 |
| 1 MB | 44 | 2.26e+07 | 46 | 2.19e+07 |
| 4 MB | 61 | 6.58e+07 | 60 | 6.62e+07 |
| 16 MB | 62 | 2.59e+08 | 61 | 2.62e+08 |

## OpenSSL AEAD baselines

The harness stores the last `+F:` machine-readable throughput line.

- `openssl_aead_aes-128-gcm.mr`: `+F:25:AES-128-GCM:16364008.62:223459807.31:903553747.49:4619150826.45:9298746580.18:10051291780.36`
- `openssl_aead_aes-256-gcm.mr`: `+F:25:AES-256-GCM:15608948.24:212937507.41:858739226.45:4171573149.50:8029036601.40:8617442786.79`
- `openssl_aead_chacha20-poly1305.mr`: `+F:25:ChaCha20-Poly1305:197991145.49:665324780.06:963671661.72:3321778815.62:3390355552.35:3396413050.10`

## dieharder randomness tests

STS gate + info logs: `dieharder_*.stdout` (single stream per run; CPU vs GPU is determined by `bench_variant.txt`).
