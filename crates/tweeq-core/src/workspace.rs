/// Produces a deterministic value in `[0, 1)` for a seed and stream index.
///
/// This is `SplitMix64` used as a small reproducible generator, not as a
/// cryptographic random number generator.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn seeded_unit(seed: u64, index: u64) -> f64 {
    let mut value = seed
        .wrapping_add(index.wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    let mantissa = value >> 11;
    mantissa as f64 * (1.0 / 9_007_199_254_740_992.0)
}

/// Selects a readable ruler interval from the `1, 2, 5 × 10ⁿ` series.
#[must_use]
pub fn nice_tick_step(units_per_pixel: f64, minimum_pixels: f64) -> f64 {
    let requested = (units_per_pixel.abs() * minimum_pixels.max(1.0)).max(f64::MIN_POSITIVE);
    let exponent = requested.log10().floor();
    let magnitude = 10_f64.powf(exponent);
    let normalized = requested / magnitude;
    let factor = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    factor * magnitude
}

/// Renderer-independent world/screen transform used by viewport adapters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewTransform {
    pub offset: [f64; 2],
    pub scale: f64,
}

impl ViewTransform {
    #[must_use]
    pub fn world_to_screen(self, world: [f64; 2]) -> [f64; 2] {
        [
            (world[0] - self.offset[0]) * self.scale,
            (world[1] - self.offset[1]) * self.scale,
        ]
    }

    #[must_use]
    pub fn screen_to_world(self, screen: [f64; 2]) -> [f64; 2] {
        let scale = self.scale.max(f64::MIN_POSITIVE);
        [
            screen[0] / scale + self.offset[0],
            screen[1] / scale + self.offset[1],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{ViewTransform, nice_tick_step, seeded_unit};

    #[test]
    #[allow(clippy::float_cmp)]
    fn shuffle_is_reproducible_and_bounded() {
        let first = seeded_unit(42, 7);
        assert_eq!(first, seeded_unit(42, 7));
        assert_ne!(first, seeded_unit(42, 8));
        assert!((0.0..1.0).contains(&first));
    }

    #[test]
    fn ruler_uses_nice_intervals() {
        assert!((nice_tick_step(0.1, 50.0) - 5.0).abs() < f64::EPSILON);
        assert!((nice_tick_step(0.03, 50.0) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn viewport_round_trips_coordinates() {
        let transform = ViewTransform {
            offset: [10.0, -5.0],
            scale: 2.5,
        };
        let world = [12.0, 3.0];
        let round_trip = transform.screen_to_world(transform.world_to_screen(world));
        assert!((round_trip[0] - world[0]).abs() < f64::EPSILON);
        assert!((round_trip[1] - world[1]).abs() < f64::EPSILON);
    }
}
