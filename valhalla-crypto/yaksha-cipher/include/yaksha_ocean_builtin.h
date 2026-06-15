/**
 * Yaksha built-in ocean (海流) — semantic contract for FFI / C ports.
 *
 * Reference: Rust `yaksha_cipher::ocean_builtin` (adaptive mask v3).
 *
 * 2D container (D = 2.20): index i → (x,y); S_i = F(key, session, phase, x, y, frac).
 * Mix rounds R(x,y) ∈ [YAKSHA_OCEAN_MIX_ROUNDS_MIN, YAKSHA_OCEAN_MIX_ROUNDS_MAX]
 * from fractional coupling — not a flat constant.
 * Phase base seed is precomputed once per buffer (OceanMaskContext in Rust).
 */
#ifndef YAKSHA_OCEAN_BUILTIN_H
#define YAKSHA_OCEAN_BUILTIN_H

#include <stdint.h>

#define YAKSHA_DIMENSION 2.20
#define YAKSHA_OCEAN_LATTICE_DIM 2

#define YAKSHA_OCEAN_MIX_ROUNDS_MIN 10
#define YAKSHA_OCEAN_MIX_ROUNDS_MAX 28
/** Legacy average — not used as flat round count in adaptive v3. */
#define YAKSHA_OCEAN_MIX_ROUNDS 19

#endif /* YAKSHA_OCEAN_BUILTIN_H */
