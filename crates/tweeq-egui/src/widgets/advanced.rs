use egui::{FontId, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use tweeq_core::{EditOperation, ParamId, ParamKind, seeded_unit};

use crate::{Number, TweeqContext};

/// Cubic Bézier timing curve with draggable handles and numeric alternatives.
pub struct CubicBezier<'a> {
    id: ParamId,
    value: &'a mut [f64; 4],
}

impl<'a> CubicBezier<'a> {
    pub fn new(id: ParamId, value: &'a mut [f64; 4]) -> Self {
        Self { id, value }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        let theme = context.theme().clone();
        let width = ui.available_width().clamp(220.0, 420.0);
        let (rect, preview) = ui.allocate_exact_size(Vec2::new(width, 126.0), Sense::hover());
        let plot = rect.shrink2(Vec2::new(18.0, 14.0));
        ui.painter()
            .rect_filled(rect, theme.input_radius, theme.input);

        for index in 0..=4 {
            let fraction = index as f32 / 4.0;
            let x = egui::lerp(plot.x_range(), fraction);
            let y = egui::lerp(plot.y_range(), fraction);
            ui.painter().line_segment(
                [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
                Stroke::new(1.0, theme.border.gamma_multiply(0.45)),
            );
            ui.painter().line_segment(
                [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
                Stroke::new(1.0, theme.border.gamma_multiply(0.45)),
            );
        }

        let start = Pos2::new(plot.left(), plot.bottom());
        let end = Pos2::new(plot.right(), plot.top());
        let first = curve_to_screen(plot, self.value[0], self.value[1]);
        let second = curve_to_screen(plot, self.value[2], self.value[3]);
        ui.painter()
            .line_segment([start, first], Stroke::new(1.0, theme.text_muted));
        ui.painter()
            .line_segment([end, second], Stroke::new(1.0, theme.text_muted));

        let mut points = Vec::with_capacity(33);
        for index in 0..=32 {
            let t = f64::from(index) / 32.0;
            let one_minus_t = 1.0 - t;
            let x = 3.0 * one_minus_t.powi(2) * t * self.value[0]
                + 3.0 * one_minus_t * t.powi(2) * self.value[2]
                + t.powi(3);
            let y = 3.0 * one_minus_t.powi(2) * t * self.value[1]
                + 3.0 * one_minus_t * t.powi(2) * self.value[3]
                + t.powi(3);
            points.push(curve_to_screen(plot, x, y));
        }
        ui.painter()
            .add(Shape::line(points, Stroke::new(2.0, theme.accent)));

        let first_response = drag_handle(
            ui,
            context,
            self.id.child(101),
            &mut self.value[0..2],
            first,
            plot,
        );
        let second_response = drag_handle(
            ui,
            context,
            self.id.child(102),
            &mut self.value[2..4],
            second,
            plot,
        );

        let numeric = ui
            .horizontal(|ui| {
                let mut combined = None;
                for (index, component) in self.value.iter_mut().enumerate() {
                    let response = Number::new(self.id.child(index as u64 + 1), component)
                        .range(0.0..=1.0)
                        .step(0.01)
                        .precision(3)
                        .bar(false)
                        .width(72.0)
                        .show(ui, context)
                        .response;
                    combined = Some(combined.map_or(response.clone(), |previous: Response| {
                        previous.union(response)
                    }));
                }
                combined.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
            })
            .inner;

        preview
            .union(first_response)
            .union(second_response)
            .union(numeric)
    }
}

fn drag_handle(
    ui: &mut Ui,
    context: &mut TweeqContext,
    id: ParamId,
    value: &mut [f64],
    center: Pos2,
    plot: Rect,
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
        state.captured = [value[0], value[1]];
        state.session = Some(context.start_edit(id, ParamKind::Vector));
    }
    if response.dragged()
        && let Some(pointer) = response.interact_pointer_pos()
        && let Some(session) = state.session
    {
        let next = screen_to_curve(plot, pointer);
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
    ui.painter().circle_filled(
        center,
        if response.hovered() { 7.0 } else { 5.0 },
        theme.accent,
    );
    context.put_vector_drag_state(id, state);
    response
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
