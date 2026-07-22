use egui::{Align2, Color32, FontId, Response, Sense, Stroke, Ui, Vec2};
use tweeq_core::{EditOperation, ParamId, ParamKind, quantize};

use crate::{Number, TweeqContext};

/// Circular angle scrubber with absolute (A) and relative (R/default) modes.
pub struct Rotary<'a> {
    id: ParamId,
    value: &'a mut f64,
    snap: f64,
    angle_offset: f64,
    size: f32,
}

impl<'a> Rotary<'a> {
    pub fn new(id: ParamId, value: &'a mut f64) -> Self {
        Self {
            id,
            value,
            snap: 15.0,
            angle_offset: 0.0,
            size: 52.0,
        }
    }

    #[must_use]
    pub fn snap(mut self, snap: f64) -> Self {
        self.snap = snap.max(f64::EPSILON);
        self
    }

    #[must_use]
    pub fn angle_offset(mut self, angle_offset: f64) -> Self {
        self.angle_offset = angle_offset;
        self
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_number(self.id, *self.value);
        let theme = context.theme().clone();
        let mut state = context.take_scalar_drag_state(self.id);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(self.size), Sense::click_and_drag());
        let response = response.on_hover_and_drag_cursor(egui::CursorIcon::ResizeHorizontal);
        let command = ui.input(|input| input.modifiers.command || input.modifiers.ctrl);
        let shift = ui.input(|input| input.modifiers.shift);
        if response.drag_started() {
            context.activate_selection(self.id, ParamKind::Number, shift, command);
            state.captured = *self.value;
            state.last_total = Vec2::ZERO;
            state.session = Some(context.start_edit(self.id, ParamKind::Number));
        }
        if response.dragged()
            && let Some(session) = state.session
        {
            let total = response.total_drag_delta().unwrap_or_default();
            let frame_delta = total - state.last_total;
            state.last_total = total;
            let absolute = ui.input(|input| input.key_down(egui::Key::A));
            let fine = if ui.input(|input| input.modifiers.alt) {
                0.1
            } else {
                1.0
            };
            let fast = if shift { self.snap } else { 1.0 };
            let mut candidate = if absolute {
                response
                    .interact_pointer_pos()
                    .map_or(*self.value, |position| {
                        let vector = position - rect.center();
                        f64::from(vector.y.atan2(vector.x).to_degrees()) + 90.0 - self.angle_offset
                    })
            } else {
                *self.value + f64::from(frame_delta.x - frame_delta.y) * 0.5 * fine * fast
            };
            if shift || ui.input(|input| input.key_down(egui::Key::Q)) {
                candidate = quantize(candidate, self.snap, 0.0);
            }
            *self.value = candidate;
            context.update_edit(
                session,
                EditOperation::AddNumber(candidate - state.captured),
            );
        }
        if response.drag_stopped()
            && let Some(session) = state.session.take()
        {
            context.finish_edit(session, true);
        }

        let center = rect.center();
        let radius = self.size * 0.38;
        ui.painter().circle_filled(center, radius, theme.input);
        ui.painter()
            .circle_stroke(center, radius, Stroke::new(1.0, theme.border));
        let radians = ((*self.value + self.angle_offset - 90.0) as f32).to_radians();
        let direction = Vec2::angled(radians);
        ui.painter().line_segment(
            [center, center + direction * radius * 0.78],
            Stroke::new(2.0, theme.accent),
        );
        ui.painter().text(
            center,
            Align2::CENTER_CENTER,
            format!("{:.0}°", *self.value),
            FontId::monospace(10.0),
            if response.hovered() {
                theme.text
            } else {
                Color32::TRANSPARENT
            },
        );
        context.put_scalar_drag_state(self.id, state);
        response
    }
}

/// Responsive composition of a Rotary and Number field.
pub struct Angle<'a> {
    id: ParamId,
    value: &'a mut f64,
    snap: f64,
}

impl<'a> Angle<'a> {
    pub fn new(id: ParamId, value: &'a mut f64) -> Self {
        Self {
            id,
            value,
            snap: 15.0,
        }
    }

    #[must_use]
    pub fn snap(mut self, snap: f64) -> Self {
        self.snap = snap;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        ui.horizontal(|ui| {
            let rotary = Rotary::new(self.id.child(1), self.value)
                .snap(self.snap)
                .show(ui, context);
            let number = Number::new(self.id.child(2), self.value)
                .step(1.0)
                .snap(self.snap)
                .precision(1)
                .suffix("°")
                .width(118.0)
                .bar(false)
                .show(ui, context)
                .response;
            rotary.union(number)
        })
        .inner
    }
}
