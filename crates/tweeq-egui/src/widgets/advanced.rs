use egui::{
    Color32, CornerRadius, FontId, Key, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui,
    Vec2,
};
use tweeq_core::{EditOperation, ParamId, ParamKind, seeded_unit};

use crate::{Number, TweeqContext};

/// Cubic Bézier timing curve with draggable handles and numeric alternatives.
pub struct CubicBezier<'a> {
    id: ParamId,
    value: &'a mut [f64; 4],
    step: f64,
    enabled: bool,
    invalid: bool,
}

impl<'a> CubicBezier<'a> {
    pub fn new(id: ParamId, value: &'a mut [f64; 4]) -> Self {
        Self {
            id,
            value,
            step: 0.01,
            enabled: true,
            invalid: false,
        }
    }

    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step;
        }
        self
    }

    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        let theme = context.theme().clone();
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, button) = ui.allocate_exact_size(Vec2::splat(theme.input_height), sense);
        let fill = if button.hovered() && self.enabled {
            theme.input_hover
        } else {
            theme.accent_hover
        };
        ui.painter().rect(
            rect,
            CornerRadius::same(theme.input_radius),
            fill,
            Stroke::new(
                1.0,
                if self.invalid {
                    Color32::from_rgb(239, 92, 92)
                } else {
                    fill
                },
            ),
            StrokeKind::Inside,
        );
        paint_curve(
            ui,
            rect.shrink(3.0),
            self.value,
            Stroke::new(
                1.5,
                if self.enabled {
                    theme.accent
                } else {
                    theme.text_muted
                },
            ),
        );

        if !self.enabled {
            return button;
        }

        let popup = egui::Popup::from_toggle_button_response(&button)
            .id(ui.make_persistent_id(("tweeq-bezier-popup", self.id.as_u64())))
            .gap(4.0)
            .width(240.0)
            .show(|ui| {
                ui.set_min_width(220.0);
                let editor =
                    show_bezier_editor(ui, context, self.id, self.value, self.step, self.invalid);
                let numeric = ui
                    .horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        let mut combined = None;
                        for (index, component) in self.value.iter_mut().enumerate() {
                            let prefix = ["X1 ", "Y1 ", "X2 ", "Y2 "][index];
                            let response = Number::new(self.id.child(index as u64 + 1), component)
                                .range(0.0..=1.0)
                                .step(self.step)
                                .precision(3)
                                .prefix(prefix)
                                .bar(false)
                                .width(52.0)
                                .invalid(self.invalid)
                                .show(ui, context)
                                .response;
                            combined =
                                Some(combined.map_or(response.clone(), |previous: Response| {
                                    previous.union(response)
                                }));
                        }
                        combined.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
                    })
                    .inner;
                editor.union(numeric)
            });

        popup.map_or(button.clone(), |popup| {
            button.union(popup.response).union(popup.inner)
        })
    }
}

fn show_bezier_editor(
    ui: &mut Ui,
    context: &mut TweeqContext,
    id: ParamId,
    value: &mut [f64; 4],
    step: f64,
    invalid: bool,
) -> Response {
    let theme = context.theme().clone();
    let (rect, preview) = ui.allocate_exact_size(Vec2::splat(220.0), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(theme.input_radius),
        theme.surface,
        Stroke::new(
            1.0,
            if invalid {
                Color32::from_rgb(239, 92, 92)
            } else {
                theme.border
            },
        ),
        StrokeKind::Inside,
    );
    let plot = rect.shrink(16.0);
    let start = Pos2::new(plot.left(), plot.bottom());
    let end = Pos2::new(plot.right(), plot.top());
    let first = curve_to_screen(plot, value[0], value[1]);
    let second = curve_to_screen(plot, value[2], value[3]);
    ui.painter()
        .line_segment([start, first], Stroke::new(1.0, theme.accent));
    ui.painter()
        .line_segment([end, second], Stroke::new(1.0, theme.accent));
    paint_curve(ui, plot, value, Stroke::new(2.0, theme.accent));

    let first_response = drag_handle(
        ui,
        context,
        id.child(101),
        &mut value[0..2],
        first,
        plot,
        step,
    );
    let second_response = drag_handle(
        ui,
        context,
        id.child(102),
        &mut value[2..4],
        second,
        plot,
        step,
    );
    preview.union(first_response).union(second_response)
}

fn paint_curve(ui: &Ui, plot: Rect, value: &[f64; 4], stroke: Stroke) {
    let mut points = Vec::with_capacity(33);
    for index in 0..=32 {
        let t = f64::from(index) / 32.0;
        let one_minus_t = 1.0 - t;
        let x = 3.0 * one_minus_t.powi(2) * t * value[0]
            + 3.0 * one_minus_t * t.powi(2) * value[2]
            + t.powi(3);
        let y = 3.0 * one_minus_t.powi(2) * t * value[1]
            + 3.0 * one_minus_t * t.powi(2) * value[3]
            + t.powi(3);
        points.push(curve_to_screen(plot, x, y));
    }
    ui.painter().add(Shape::line(points, stroke));
}

fn drag_handle(
    ui: &mut Ui,
    context: &mut TweeqContext,
    id: ParamId,
    value: &mut [f64],
    center: Pos2,
    plot: Rect,
    step: f64,
) -> Response {
    let egui_id = ui.make_persistent_id(("tweeq-bezier-handle", id.as_u64()));
    let response = ui.interact(
        Rect::from_center_size(center, Vec2::splat(24.0)),
        egui_id,
        Sense::drag(),
    );
    let theme = context.theme().clone();
    context.register_vector(id, [value[0], value[1]]);
    let mut state = context.take_vector_drag_state(id);
    if response.drag_started() {
        response.request_focus();
        context.activate_selection(id, ParamKind::Vector, false, false);
        state.captured = [value[0], value[1]];
        state.session = Some(context.start_edit(id, ParamKind::Vector));
    }
    if response.dragged()
        && let Some(pointer) = response.interact_pointer_pos()
        && let Some(session) = state.session
    {
        let mut next = screen_to_curve(plot, pointer);
        if ui.input(|input| input.key_down(Key::Q)) {
            next[0] = quantize_bezier(next[0], step.max(0.1));
            next[1] = quantize_bezier(next[1], step.max(0.1));
        }
        let delta = [next[0] - state.captured[0], next[1] - state.captured[1]];
        value.copy_from_slice(&next);
        context.update_edit(
            session,
            EditOperation::AddVector {
                delta: [delta[0], delta[1], 0.0, 0.0],
                dimensions: 2,
            },
        );
    }
    if response.drag_stopped()
        && let Some(session) = state.session.take()
    {
        context.finish_edit(session, true);
    }
    if response.has_focus() && !response.dragged() {
        let direction = ui.input(|input| {
            let x = f64::from(input.key_pressed(Key::ArrowRight))
                - f64::from(input.key_pressed(Key::ArrowLeft));
            let y = f64::from(input.key_pressed(Key::ArrowUp))
                - f64::from(input.key_pressed(Key::ArrowDown));
            [x, y]
        });
        if direction != [0.0, 0.0] {
            let multiplier = ui.input(|input| {
                if input.modifiers.shift {
                    10.0
                } else if input.modifiers.alt {
                    0.1
                } else {
                    1.0
                }
            });
            let before = [value[0], value[1]];
            value[0] = (value[0] + direction[0] * step * multiplier).clamp(0.0, 1.0);
            value[1] = (value[1] + direction[1] * step * multiplier).clamp(0.0, 1.0);
            if [value[0], value[1]]
                .into_iter()
                .zip(before)
                .any(|(current, previous)| (current - previous).abs() > f64::EPSILON)
            {
                context.immediate_edit(
                    id,
                    ParamKind::Vector,
                    EditOperation::AddVector {
                        delta: [value[0] - before[0], value[1] - before[1], 0.0, 0.0],
                        dimensions: 2,
                    },
                );
            }
        }
    }
    ui.painter().circle(
        center,
        if response.hovered() { 7.0 } else { 6.0 },
        if response.hovered() {
            theme.accent
        } else {
            theme.background
        },
        Stroke::new(2.0, theme.accent),
    );
    context.put_vector_drag_state(id, state);
    response
}

fn quantize_bezier(value: f64, step: f64) -> f64 {
    if step.is_finite() && step > 0.0 {
        ((value / step).round() * step).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn curve_to_screen(rect: Rect, x: f64, y: f64) -> Pos2 {
    Pos2::new(
        egui::lerp(rect.x_range(), x.clamp(0.0, 1.0) as f32),
        egui::lerp(rect.y_range(), (1.0 - y.clamp(0.0, 1.0)) as f32),
    )
}

fn screen_to_curve(rect: Rect, point: Pos2) -> [f64; 2] {
    [
        f64::from(((point.x - rect.left()) / rect.width()).clamp(0.0, 1.0)),
        f64::from((1.0 - (point.y - rect.top()) / rect.height()).clamp(0.0, 1.0)),
    ]
}

/// Reproducible shuffle control backed by a host-owned seed.
pub struct Shuffle<'a> {
    id: ParamId,
    value: &'a mut f64,
    seed: &'a mut u64,
    range: std::ops::RangeInclusive<f64>,
}

impl<'a> Shuffle<'a> {
    pub fn new(id: ParamId, value: &'a mut f64, seed: &'a mut u64) -> Self {
        Self {
            id,
            value,
            seed,
            range: 0.0..=1.0,
        }
    }

    #[must_use]
    pub fn range(mut self, range: std::ops::RangeInclusive<f64>) -> Self {
        self.range = range;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_number(self.id, *self.value);
        ui.horizontal(|ui| {
            let response = ui.button("Shuffle");
            if response.clicked() {
                *self.seed = self.seed.wrapping_add(1);
                let start = *self.range.start();
                let end = *self.range.end();
                *self.value = egui::lerp(start..=end, seeded_unit(*self.seed, 0));
                context.immediate_edit(
                    self.id,
                    ParamKind::Number,
                    EditOperation::SetNumber(*self.value),
                );
            }
            ui.monospace(format!("{:.4} · seed {}", *self.value, *self.seed));
            response
        })
        .inner
    }
}

/// Two-number complex input assembled from the shared Number primitive.
pub struct ComplexInput<'a> {
    id: ParamId,
    value: &'a mut [f64; 2],
}

impl<'a> ComplexInput<'a> {
    pub fn new(id: ParamId, value: &'a mut [f64; 2]) -> Self {
        Self { id, value }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        ui.horizontal(|ui| {
            ui.monospace("Re");
            let real = Number::new(self.id.child(1), &mut self.value[0])
                .step(0.01)
                .precision(3)
                .bar(false)
                .width(90.0)
                .show(ui, context)
                .response;
            ui.monospace("Im");
            let imaginary = Number::new(self.id.child(2), &mut self.value[1])
                .step(0.01)
                .precision(3)
                .bar(false)
                .width(90.0)
                .show(ui, context)
                .response;
            real.union(imaginary)
        })
        .inner
    }
}

/// Lightweight multiline code-editor adapter.
pub struct CodeInput<'a> {
    code: &'a mut String,
    rows: usize,
}

impl<'a> CodeInput<'a> {
    pub fn new(code: &'a mut String) -> Self {
        Self { code, rows: 7 }
    }

    #[must_use]
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(
            egui::TextEdit::multiline(self.code)
                .font(FontId::monospace(12.0))
                .code_editor()
                .desired_rows(self.rows)
                .desired_width(f32::INFINITY),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{curve_to_screen, quantize_bezier, screen_to_curve};
    use egui::{Pos2, Rect};

    #[test]
    fn bezier_quantization_stays_in_unit_square() {
        assert!((quantize_bezier(0.26, 0.1) - 0.3).abs() < 1.0e-12);
        assert!((quantize_bezier(2.0, 0.1) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bezier_screen_mapping_round_trips() {
        let rect = Rect::from_min_max(Pos2::new(10.0, 20.0), Pos2::new(210.0, 220.0));
        let screen = curve_to_screen(rect, 0.25, 0.75);
        let curve = screen_to_curve(rect, screen);
        assert!((curve[0] - 0.25).abs() < 1.0e-6);
        assert!((curve[1] - 0.75).abs() < 1.0e-6);
    }
}
