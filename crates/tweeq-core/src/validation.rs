/// Result of applying numeric constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberValidation {
    pub value: f64,
    pub clamped: bool,
    pub quantized: bool,
}

/// Validation and formatting constraints for a numeric parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub clamp_min: bool,
    pub clamp_max: bool,
    pub precision: u8,
}

impl Default for NumberConstraints {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            step: None,
            clamp_min: true,
            clamp_max: true,
            precision: 4,
        }
    }
}

impl NumberConstraints {
    /// Applies clamp and optional step quantization.
    #[must_use]
    pub fn validate(self, value: f64, apply_step: bool) -> NumberValidation {
        if !value.is_finite() {
            return NumberValidation {
                value,
                clamped: false,
                quantized: false,
            };
        }

        let mut output = value;
        let mut clamped = false;
        if self.clamp_min
            && let Some(minimum) = self.min
            && output < minimum
        {
            output = minimum;
            clamped = true;
        }
        if self.clamp_max
            && let Some(maximum) = self.max
            && output > maximum
        {
            output = maximum;
            clamped = true;
        }

        let before_quantize = output;
        if apply_step && let Some(step) = self.step {
            output = quantize(output, step, self.min.unwrap_or(0.0));
        }

        NumberValidation {
            value: normalize_zero(output),
            clamped,
            quantized: (output - before_quantize).abs()
                > f64::EPSILON * output.abs().max(before_quantize.abs()).max(1.0),
        }
    }
}

/// Snaps `value` to a positive interval relative to `origin`.
#[must_use]
pub fn quantize(value: f64, step: f64, origin: f64) -> f64 {
    if !value.is_finite() || !step.is_finite() || step <= 0.0 || !origin.is_finite() {
        return value;
    }
    normalize_zero(((value - origin) / step).round().mul_add(step, origin))
}

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

#[cfg(test)]
mod tests {
    use super::{NumberConstraints, quantize};

    #[test]
    fn quantizes_relative_to_minimum() {
        assert!((quantize(0.26, 0.1, 0.05) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn constraints_clamp_and_quantize() {
        let constraints = NumberConstraints {
            min: Some(0.0),
            max: Some(1.0),
            step: Some(0.1),
            ..Default::default()
        };
        let high = constraints.validate(2.0, true);
        assert!((high.value - 1.0).abs() < f64::EPSILON);
        assert!(high.clamped);
        let middle = constraints.validate(0.26, true);
        assert!((middle.value - 0.3).abs() < 1e-12);
        assert!(middle.quantized);
    }

    #[test]
    fn non_finite_values_are_preserved_for_host_policy() {
        let result = NumberConstraints::default().validate(f64::NAN, true);
        assert!(result.value.is_nan());
    }
}
