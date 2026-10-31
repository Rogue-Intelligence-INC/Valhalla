//! Non-integer dimension container (Yaksha standard **2.20** → **2D** slice + fractional coupling).

pub const YAKSHA_DIMENSION: f64 = 2.20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FracDimContainer {
    pub dimension: f64,
    pub frac: f64,
    pub dim_factor: u8,
    pub frac_factor: u8,
}

impl FracDimContainer {
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

    #[inline]
    pub fn index_to_2d(&self, index: u64) -> (u32, u32) {
        let stride = 256u32 + (self.frac * 256.0) as u32;
        let stride = stride.max(257);
        let x = (index % u64::from(stride)) as u32;
        let y = (index / u64::from(stride)) as u32;
        (x, y)
    }

    /// Ocean lattice stride for 2D index mapping (GPU uniform).
    #[inline]
    pub fn ocean_stride(&self) -> u32 {
        let stride = 256u32 + (self.frac * 256.0) as u32;
        stride.max(257)
    }

    /// `f32::to_bits(frac)` for GPU fractional coupling.
    #[inline]
    pub fn frac_bits(&self) -> u32 {
        (self.frac as f32).to_bits()
    }

    /// Fractional part as a fixed-point integer (scale 2^20) for exact
    /// CPU/GPU-identical coupling arithmetic (no f32/f64 divergence).
    #[inline]
    pub fn frac_scaled(&self) -> u64 {
        (self.frac * 1_048_576.0) as u64
    }

    #[inline]
    pub fn frac_coupling(&self, x: u32, y: u32) -> u64 {
        // Integer-exact: x + floor(y * frac), identical on CPU and GPU.
        x as u64 + ((y as u64 * self.frac_scaled()) >> 20)
    }
}
