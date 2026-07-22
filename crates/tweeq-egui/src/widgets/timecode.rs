use egui::{Response, Ui};
use tweeq_core::ParamId;

use crate::{Number, TweeqContext};

/// Frame-backed SMPTE-style timecode input.
pub struct Timecode<'a> {
    id: ParamId,
    frames: &'a mut i64,
    frame_rate: u32,
}

impl<'a> Timecode<'a> {
    pub fn new(id: ParamId, frames: &'a mut i64) -> Self {
        Self {
            id,
            frames,
            frame_rate: 24,
        }
    }

    #[must_use]
    pub fn frame_rate(mut self, frame_rate: u32) -> Self {
        self.frame_rate = frame_rate.max(1);
        self
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        let mut numeric = self.frames.to_string().parse::<f64>().unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.monospace(format_timecode(*self.frames, self.frame_rate));
            let response = Number::new(self.id, &mut numeric)
                .step(1.0)
                .snap(f64::from(self.frame_rate))
                .precision(0)
                .suffix(" f")
                .bar(false)
                .width(104.0)
                .show(ui, context);
            if response.changed {
                *self.frames = numeric.round() as i64;
            }
            response.response
        })
        .inner
    }
}

fn format_timecode(frames: i64, frame_rate: u32) -> String {
    let fps = i64::from(frame_rate.max(1));
    let sign = if frames < 0 { "-" } else { "" };
    let absolute = frames.saturating_abs();
    let frame = absolute % fps;
    let seconds_total = absolute / fps;
    let second = seconds_total % 60;
    let minutes_total = seconds_total / 60;
    let minute = minutes_total % 60;
    let hour = minutes_total / 60;
    format!("{sign}{hour:02}:{minute:02}:{second:02}:{frame:02}")
}

#[cfg(test)]
mod tests {
    use super::format_timecode;

    #[test]
    fn formats_frame_timecode() {
        assert_eq!(format_timecode(24 * 3661 + 12, 24), "01:01:01:12");
    }
}
