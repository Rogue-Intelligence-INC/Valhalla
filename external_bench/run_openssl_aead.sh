#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:?out_dir missing}"
OPENSSL_SECONDS="${2:-2}"

OPENSSL_CIPHERS=(
  aes-128-gcm
  aes-256-gcm
  chacha20-poly1305
)

for cipher in "${OPENSSL_CIPHERS[@]}"; do
  out="$OUT_DIR/openssl_aead_${cipher}.mr"
  echo "OpenSSL AEAD: $cipher -> $out"
  # -mr => machine-readable, -aead => AEAD EVP in TLS-like sequence
  openssl speed -aead -evp "$cipher" -seconds "$OPENSSL_SECONDS" -mr >"$out" 2>"$out.stderr"
done

