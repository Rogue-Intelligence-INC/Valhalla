//! Non-integer dimension container (Yaksha standard **2.20** → **2D** integer slice + fractional coupling).
//!
//! Ocean (海流) interference is defined on the **2D** lattice inside this container;
//! the fractional part modulates stride and cross-axis coupling.

/// Standard Yaksha effective dimension (integer floor = 2 → 2D ocean grid).
pub const YAKSHA_DIMENSION: f64 = 2.20;

/// Integer dimension of the ocean lattice (floor of [`YAKSHA_DIMENSION`]).
pub const OCEAN_LATTICE_DIM: u32 = 2;

/// Container parameters derived from a dimension value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FracDimContainer {
    pub dimension: f64,
    /// Fractional part in `[0, 1)` — couples the two integer axes.
    pub frac: f64,
    pub dim_factor: u8,
    pub frac_factor: u8,
}

impl FracDimContainer {
    /// Standard **2.20** container used by `core` and global ocean.
    #[inline]
    pub fn standard() -> Self {
        Self::from_dimension(YAKSHA_DIMENSION)
    }

    #[inline]
    pub fn from_dimension(dimension: f64) -> Self {
        let frac = dimension - dimension.floor();
        let dim_factor = ((dimension * 100.0) as u8).wrapping_mul(17);
        let frac_factor = ((frac * 256.0) as u8).wrapping_mul(23);
        Self {
            dimension,
            frac,
            dim_factor,
            frac_factor,
        }
    }

    /// Map linear byte index → **2D** coordinates `(x, y)` inside the container.
    ///
    /// Stride along x is `256 + ⌊frac·256⌋` so the fractional dimension changes grid anisotropy.
    #[inline]
    pub fn index_to_2d(&self, index: u64) -> (u32, u32) {
        let stride = 256u32 + (self.frac * 256.0) as u32;
        let stride = stride.max(257);
        let x = (index % u64::from(stride)) as u32;
        let y = (index / u64::from(stride)) as u32;
        (x, y)
    }

    /// Cross-axis coupling from fractional dimension (used by ocean stream).
    #[inline]
    pub fn frac_coupling(&self, x: u32, y: u32) -> u64 {
        let xf = x as f64;
        let yf = y as f64 * self.frac;
        (xf + yf) as u64
    }
}
