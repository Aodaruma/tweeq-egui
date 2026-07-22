use egui::{Align2, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use tweeq_core::{EditOperation, ParamId, ParamKind};

use crate::{Number, TweeqContext};

/// Two-dimensional position pad with X/Y constraint shortcuts.
pub struct Position<'a> {
    id: ParamId,
    value: &'a mut [f64; 2],
    scale: f64,
}

impl<'a> Position<'a> {
    pub fn new(id: ParamId, value: &'a mut [f64; 2]) -> Self {
        Self {
            id,
            value,
            scale: 1.0,
        }
    }

    #[must_use]
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_vector(self.id, *self.value);
        let theme = context.theme().clone();
        let mut state = context.take_vector_drag_state(self.id);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(180.0, 82.0), Sense::drag());
        let response = response.on_hover_and_drag_cursor(egui::CursorIcon::Crosshair);
        if response.drag_started() {
            context.activate_selection(self.id, ParamKind::Vector, false, false);
            state.captured = *self.value;
            state.last_total = Vec2::ZERO;
            state.session = Some(context.start_edit(self.id, ParamKind::Vector));
        }
        if response.dragged()
            && let Some(session) = state.session
        {
            let total = response.total_drag_delta().unwrap_or_default();
            state.last_total = total;
            let fast = if ui.input(|input| input.modifiers.shift) {
                10.0
            } else {
                1.0
            };
            let fine = if ui.input(|input| input.modifiers.alt) {
                0.1
            } else {
                1.0
            };
            let only_x =
                ui.input(|input| input.key_down(egui::Key::X) || input.key_down(egui::Key::Num0));
            let only_y =
                ui.input(|input| input.key_down(egui::Key::Y) || input.key_down(egui::Key::Num1));
            let mut delta = [
                f64::from(total.x) * self.scale * fast * fine,
                f64::from(total.y) * self.scale * fast * fine,
            ];
            if only_x {
                delta[1] = 0.0;
            }
            if only_y {
                delta[0] = 0.0;
            }
            *self.value = [state.captured[0] + delta[0], state.captured[1] + delta[1]];
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
        ui.painter().rect(
            rect,
            CornerRadius::same(theme.input_radius),
            if response.hovered() {
                theme.input_hover
            } else {
                theme.input
            },
            Stroke::new(
                1.0,
                if context.is_selected(self.id) {
                    theme.accent
                } else {
                    theme.border
                },
            ),
            StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [
                egui::pos2(rect.center().x, rect.top() + 8.0),
                egui::pos2(rect.center().x, rect.bottom() - 8.0),
            ],
            Stroke::new(1.0, theme.border),
        );
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + 8.0, rect.center().y),
                egui::pos2(rect.right() - 8.0, rect.center().y),
            ],
            Stroke::new(1.0, theme.border),
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("x {:.1}   y {:.1}", self.value[0], self.value[1]),
            FontId::monospace(11.0),
            theme.text,
        );
        context.put_vector_drag_state(self.id, state);
        response
    }
}

/// Alias-style translation pad with the same semantics as Position.
pub struct Translate<'a>(Position<'a>);

impl<'a> Translate<'a> {
    pub fn new(id: ParamId, value: &'a mut [f64; 2]) -> Self {
        Self(Position::new(id, value))
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        self.0.show(ui, context)
    }
}

/// Compact numeric vector editor.
pub struct Vector<'a, const N: usize> {
    id: ParamId,
    value: &'a mut [f64; N],
    step: f64,
}

impl<'a, const N: usize> Vector<'a, N> {
    pub fn new(id: ParamId, value: &'a mut [f64; N]) -> Self {
        Self {
            id,
            value,
            step: 0.1,
        }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        ui.horizontal(|ui| {
            let mut combined = None;
            for (index, component) in self.value.iter_mut().enumerate() {
                let response = Number::new(self.id.child(index as u64 + 1), component)
                    .step(self.step)
                    .precision(2)
                    .bar(false)
                    .width(78.0)
                    .show(ui, context)
                    .response;
                combined = Some(combined.map_or(response.clone(), |previous: Response| {
                    previous.union(response)
                }));
            }
            combined.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
        })
        .inner
    }
}

/// Two-component size editor with optional aspect-ratio lock.
pub struct Size<'a> {
    id: ParamId,
    value: &'a mut [f64; 2],
    locked: &'a mut bool,
}

impl<'a> Size<'a> {
    pub fn new(id: ParamId, value: &'a mut [f64; 2], locked: &'a mut bool) -> Self {
        Self { id, value, locked }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        let before = *self.value;
        let ratio = if before[1].abs() > f64::EPSILON {
            before[0] / before[1]
        } else {
            1.0
        };
        ui.horizontal(|ui| {
            let width = Number::new(self.id.child(1), &mut self.value[0])
                .step(1.0)
                .precision(1)
                .bar(false)
                .width(72.0)
                .show(ui, context);
            let lock = ui.selectable_label(*self.locked, "🔗");
            if lock.clicked() {
                *self.locked = !*self.locked;
            }
            let height = Number::new(self.id.child(2), &mut self.value[1])
                .step(1.0)
                .precision(1)
                .bar(false)
                .width(72.0)
                .show(ui, context);
            if *self.locked && width.changed {
                self.value[1] = self.value[0] / ratio;
            }
            if *self.locked && height.changed {
                self.value[0] = self.value[1] * ratio;
            }
            width.response.union(lock).union(height.response)
        })
        .inner
    }
}
