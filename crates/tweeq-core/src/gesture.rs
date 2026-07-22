/// Keyboard modifiers that affect a tweak gesture.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GestureModifiers {
    pub fine: bool,
    pub fast: bool,
    pub snap: bool,
}

/// Per-frame output from a tweak gesture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureUpdate {
    pub delta: f64,
    pub accumulated_delta: f64,
    pub speed: f64,
    pub snap: bool,
}

/// Stateful conversion from two-dimensional pointer motion to a scalar delta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TweakGesture {
    speed: f64,
    accumulated_delta: f64,
    direction: [f64; 2],
    horizontal_weight: f64,
}

impl Default for TweakGesture {
    fn default() -> Self {
        Self {
            speed: 1.0,
            accumulated_delta: 0.0,
            direction: [1.0, 0.0],
            horizontal_weight: 1.0,
        }
    }
}

impl TweakGesture {
    /// Resets accumulated motion and sensitivity.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Current gesture-only sensitivity multiplier.
    #[must_use]
    pub const fn speed(&self) -> f64 {
        self.speed
    }

    /// How strongly the latest motion is interpreted as horizontal value input.
    #[must_use]
    pub const fn horizontal_weight(&self) -> f64 {
        self.horizontal_weight
    }

    /// Converts one pointer-motion sample into a value delta.
    pub fn update(
        &mut self,
        motion: [f32; 2],
        base_speed: f64,
        modifiers: GestureModifiers,
        fast_multiplier: f64,
        speed_range: std::ops::RangeInclusive<f64>,
    ) -> GestureUpdate {
        let [dx, dy] = motion;
        let mixed_x = self.direction[0].mul_add(0.9, f64::from(dx.abs()) * 0.1);
        let mixed_y = self.direction[1].mul_add(0.9, f64::from(dy.abs()) * 0.1);
        let length = mixed_x.hypot(mixed_y);
        if length > f64::EPSILON {
            self.direction = [mixed_x / length, mixed_y / length];
        }
        self.horizontal_weight = smoothstep(0.4, 0.6, self.direction[0].abs());

        let vertically_adjusted = self.speed * 0.98_f64.powf(f64::from(dy));
        self.speed = lerp(vertically_adjusted, self.speed, self.horizontal_weight)
            .clamp(*speed_range.start(), *speed_range.end());

        let key_speed = if modifiers.fine { 0.1 } else { 1.0 }
            * if modifiers.fast {
                fast_multiplier.max(1.0)
            } else {
                1.0
            };
        let delta = f64::from(dx) * base_speed * self.speed * key_speed * self.horizontal_weight;
        self.accumulated_delta += delta;

        GestureUpdate {
            delta,
            accumulated_delta: self.accumulated_delta,
            speed: self.speed,
            snap: modifiers.snap,
        }
    }
}

fn lerp(from: f64, to: f64, amount: f64) -> f64 {
    (to - from).mul_add(amount, from)
}

fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let amount = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

#[cfg(test)]
mod tests {
    use super::{GestureModifiers, TweakGesture};

    #[test]
    fn horizontal_motion_accumulates_from_capture() {
        let mut gesture = TweakGesture::default();
        let first = gesture.update(
            [2.0, 0.0],
            0.5,
            GestureModifiers::default(),
            10.0,
            0.01..=100.0,
        );
        let second = gesture.update(
            [3.0, 0.0],
            0.5,
            GestureModifiers::default(),
            10.0,
            0.01..=100.0,
        );
        assert!((first.accumulated_delta - 1.0).abs() < f64::EPSILON);
        assert!((second.accumulated_delta - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn fine_and_fast_modifiers_change_scale() {
        let mut fine = TweakGesture::default();
        let mut fast = TweakGesture::default();
        let fine_delta = fine.update(
            [10.0, 0.0],
            1.0,
            GestureModifiers {
                fine: true,
                ..Default::default()
            },
            10.0,
            0.01..=100.0,
        );
        let fast_delta = fast.update(
            [10.0, 0.0],
            1.0,
            GestureModifiers {
                fast: true,
                ..Default::default()
            },
            10.0,
            0.01..=100.0,
        );
        assert!((fine_delta.delta - 1.0).abs() < f64::EPSILON);
        assert!((fast_delta.delta - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vertical_motion_changes_sensitivity() {
        let mut gesture = TweakGesture::default();
        let slower = gesture.update(
            [0.0, 20.0],
            1.0,
            GestureModifiers::default(),
            10.0,
            0.01..=100.0,
        );
        assert!(slower.speed < 1.0);
        assert!(fine_enough(slower.delta, 0.0));
        assert!(gesture.horizontal_weight() < 0.1);
    }

    #[test]
    fn diagonal_motion_blends_value_and_sensitivity() {
        let mut gesture = TweakGesture::default();
        let update = gesture.update(
            [10.0, 10.0],
            1.0,
            GestureModifiers::default(),
            10.0,
            0.01..=100.0,
        );
        assert!(update.delta > 0.0);
        assert!(update.delta <= 10.0);
        assert!(update.speed <= 1.0);
    }

    fn fine_enough(left: f64, right: f64) -> bool {
        (left - right).abs() < 1e-9
    }
}
