use egui::{
    Color32, CornerRadius, FontFamily, FontId, Key, LayerId, Order, Pos2, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2, ViewportCommand,
};
use tweeq_core::{EditOperation, ParamId, ParamKind, quantize};

use crate::{Number, TweeqContext, context::RotaryMode};

const SNAP_INNER_RADIUS_FACTOR: f32 = 4.0;
const SNAP_OUTER_RADIUS: f32 = 160.0;
const ARC_RADIUS_STEP_FACTOR: f32 = 0.25;

/// Circular angle scrubber with absolute (A) and relative (R/default) modes.
pub struct Rotary<'a> {
    id: ParamId,
    value: &'a mut f64,
    snap: f64,
    angle_offset: f64,
    size: f32,
    enabled: bool,
    invalid: bool,
}

impl<'a> Rotary<'a> {
    pub fn new(id: ParamId, value: &'a mut f64) -> Self {
        Self {
            id,
            value,
            snap: 45.0,
            angle_offset: -90.0,
            size: 24.0,
            enabled: true,
            invalid: false,
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

    #[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_number(self.id, *self.value);
        let theme = context.theme().clone();
        let mut state = context.take_rotary_state(self.id);
        let sense = if self.enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), sense);
        let response = if self.enabled {
            response.on_hover_and_drag_cursor(egui::CursorIcon::Crosshair)
        } else {
            response
        };
        let command = ui.input(|input| input.modifiers.command || input.modifiers.ctrl);
        let shift = ui.input(|input| input.modifiers.shift);

        if self.enabled && response.drag_started() {
            context.activate_selection(self.id, ParamKind::Number, shift, command);
            response.request_focus();
            let pointer = response.interact_pointer_pos().unwrap_or(rect.center());
            state.captured = *self.value;
            state.local = *self.value;
            state.origin = pointer;
            state.previous = pointer;
            state.current = pointer;
            state.pointer_mode =
                pointer_mode(rect.center(), pointer, *self.value, self.angle_offset);
            state.session = Some(context.start_edit(self.id, ParamKind::Number));
            if state.pointer_mode == RotaryMode::Absolute {
                let pointer_angle = screen_angle(pointer - rect.center()) - self.angle_offset;
                state.local += signed_angle_between(pointer_angle, state.local);
                *self.value = state.local;
            }
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::CursorVisible(false));
        }

        let mode = if ui.input(|input| input.key_down(Key::A)) {
            RotaryMode::Absolute
        } else if ui.input(|input| input.key_down(Key::R)) {
            RotaryMode::Relative
        } else {
            state.pointer_mode
        };

        if self.enabled
            && response.dragged()
            && let Some(session) = state.session
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let previous_vector = state.previous - rect.center();
            let current_vector = pointer - rect.center();
            if previous_vector.length_sq() > 1.0 && current_vector.length_sq() > 1.0 {
                state.local += signed_vector_angle(previous_vector, current_vector);
            }
            state.previous = pointer;
            state.current = pointer;

            let radius = current_vector.length();
            let snap_by_radius = theme.input_height * SNAP_INNER_RADIUS_FACTOR <= radius
                && radius <= SNAP_OUTER_RADIUS;
            let do_snap = shift || ui.input(|input| input.key_down(Key::Q)) || snap_by_radius;
            let candidate = if do_snap {
                quantize(state.local, self.snap, 0.0)
            } else {
                state.local
            };
            *self.value = candidate;
            let operation = if mode == RotaryMode::Absolute {
                EditOperation::SetNumber(candidate)
            } else {
                EditOperation::AddNumber(candidate - state.captured)
            };
            context.update_edit(session, operation);
            paint_rotary_overlay(
                ui,
                &theme,
                rect.center(),
                &state,
                candidate,
                self.snap,
                self.angle_offset,
                mode,
                do_snap,
            );
        }

        if self.enabled && response.drag_stopped() {
            if let Some(session) = state.session.take() {
                context.finish_edit(session, true);
            }
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::CursorVisible(true));
        }

        paint_rotary(
            ui,
            &theme,
            rect,
            &response,
            *self.value,
            self.angle_offset,
            mode,
            self.enabled,
            self.invalid,
        );
        context.put_rotary_state(self.id, state);
        response
    }
}

#[allow(clippy::too_many_arguments, clippy::cast_possible_truncation)]
fn paint_rotary(
    ui: &Ui,
    theme: &crate::TweeqTheme,
    rect: egui::Rect,
    response: &Response,
    value: f64,
    angle_offset: f64,
    mode: RotaryMode,
    enabled: bool,
    invalid: bool,
) {
    let active = response.hovered() || response.dragged();
    let radius = rect.width() * 0.5 * if active { 1.8 } else { 1.0 };
    let center = rect.center();
    let fill = if !enabled {
        theme.input
    } else if active && mode == RotaryMode::Absolute {
        theme.accent.gamma_multiply(0.28)
    } else if active {
        theme.accent_hover
    } else {
        theme.accent
    };
    ui.painter().circle_filled(center, radius, fill);
    if invalid {
        ui.painter().circle_stroke(
            center,
            radius + 1.0,
            Stroke::new(1.0, Color32::from_rgb(238, 79, 87)),
        );
    } else if response.has_focus() {
        ui.painter()
            .circle_stroke(center, radius + 3.0, Stroke::new(1.0, theme.accent_hover));
    }

    let direction = angle_direction(value + angle_offset);
    let tip_color = if mode == RotaryMode::Absolute && active {
        theme.accent_hover
    } else {
        theme.input
    };
    ui.painter().line_segment(
        [
            center + direction * radius * 0.25,
            center + direction * radius * 0.88,
        ],
        Stroke::new(3.0, tip_color),
    );
}

#[allow(clippy::too_many_arguments, clippy::cast_possible_truncation)]
fn paint_rotary_overlay(
    ui: &Ui,
    theme: &crate::TweeqTheme,
    center: Pos2,
    state: &crate::context::RotaryState,
    value: f64,
    snap: f64,
    angle_offset: f64,
    mode: RotaryMode,
    do_snap: bool,
) {
    let painter = ui.ctx().layer_painter(LayerId::new(
        Order::Foreground,
        egui::Id::new("tweeq-rotary-overlay"),
    ));
    let inner_radius = theme.input_height * SNAP_INNER_RADIUS_FACTOR;
    let outer_radius = SNAP_OUTER_RADIUS;
    let snap = snap.max(f64::EPSILON);
    let meter_count = (360.0 / snap).ceil().clamp(1.0, 720.0) as usize;
    for index in 0..meter_count {
        let angle = index as f64 * snap;
        let direction = angle_direction(angle + angle_offset);
        painter.line_segment(
            [
                center + direction * inner_radius,
                center + direction * outer_radius,
            ],
            Stroke::new(
                if do_snap { 2.0 } else { 1.0 },
                if do_snap {
                    theme.accent.gamma_multiply(0.36)
                } else {
                    theme.border
                },
            ),
        );
    }

    if do_snap && nearly_multiple(value, snap) {
        let direction = angle_direction(value + angle_offset);
        painter.line_segment(
            [
                center + direction * inner_radius,
                center + direction * outer_radius,
            ],
            Stroke::new(2.5, theme.accent),
        );
    }

    if mode == RotaryMode::Absolute {
        let direction = angle_direction(value + angle_offset);
        let distance = state.current.distance(center).max(theme.input_height);
        painter.line_segment(
            [
                center + direction * theme.input_height,
                center + direction * distance,
            ],
            Stroke::new(2.0, theme.accent),
        );
    } else {
        paint_relative_arc(
            &painter,
            center,
            state.captured + angle_offset,
            state.local + angle_offset,
            theme.input_height * SNAP_INNER_RADIUS_FACTOR,
            theme.input_height * ARC_RADIUS_STEP_FACTOR,
            theme.accent,
        );
    }

    paint_angle_label(ui, &painter, theme, state.origin, state.current, value);
}

#[allow(clippy::cast_possible_truncation, clippy::too_many_arguments)]
fn paint_relative_arc(
    painter: &egui::Painter,
    center: Pos2,
    start: f64,
    end: f64,
    base_radius: f32,
    radius_step: f32,
    color: Color32,
) {
    let total = end - start;
    let turns = (total.abs() / 360.0).floor() as usize;
    let sign = total.signum() as f32;
    for index in 0..turns {
        let radius = base_radius + sign * index as f32 * radius_step;
        painter.circle_stroke(center, radius.max(8.0), Stroke::new(2.0, color));
    }
    let signed_turns = turns as f64 * total.signum();
    let remainder = total - signed_turns * 360.0;
    let arc_radius = (base_radius + sign * turns as f32 * radius_step).max(8.0);
    let points = arc_points(center, arc_radius, start, start + remainder);
    if points.len() >= 2 {
        painter.add(egui::Shape::line(points.clone(), Stroke::new(2.0, color)));
        let end_point = *points.last().expect("arc has at least two points");
        let direction = angle_direction(start + remainder);
        let tangent = if remainder >= 0.0 {
            Vec2::new(-direction.y, direction.x)
        } else {
            Vec2::new(direction.y, -direction.x)
        };
        let normal = Vec2::new(-tangent.y, tangent.x);
        painter.add(egui::Shape::convex_polygon(
            vec![
                end_point + tangent * 5.0,
                end_point - tangent * 3.0 + normal * 3.0,
                end_point - tangent * 3.0 - normal * 3.0,
            ],
            color,
            Stroke::NONE,
        ));
    }
}

fn paint_angle_label(
    ui: &Ui,
    painter: &egui::Painter,
    theme: &crate::TweeqTheme,
    origin: Pos2,
    pointer: Pos2,
    value: f64,
) {
    let bounds = ui.ctx().content_rect().shrink(40.0);
    let position = clamp_along_ray(origin, pointer, bounds);
    let text = display_angle(value);
    let font = FontId::new(13.0, FontFamily::Monospace);
    let galley = painter.layout_no_wrap(text, font, theme.text);
    let label_size = galley.size() + Vec2::new(18.0, 10.0);
    let label_rect = egui::Rect::from_center_size(position, label_size);
    painter.rect_filled(label_rect, CornerRadius::same(4), theme.surface);
    painter.rect_stroke(
        label_rect,
        CornerRadius::same(4),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );
    painter.galley(position - galley.size() * 0.5, galley, theme.text);

    let drag_direction = (pointer - origin).normalized();
    if drag_direction != Vec2::ZERO {
        let normal = Vec2::new(-drag_direction.y, drag_direction.x);
        let left = position - drag_direction * (label_size.x * 0.5 + 6.0);
        let right = position + drag_direction * (label_size.x * 0.5 + 6.0);
        for (tip, sign) in [(left, 1.0), (right, -1.0)] {
            painter.line_segment(
                [tip, tip + drag_direction * sign * 5.0 + normal * 4.0],
                Stroke::new(1.5, theme.accent),
            );
            painter.line_segment(
                [tip, tip + drag_direction * sign * 5.0 - normal * 4.0],
                Stroke::new(1.5, theme.accent),
            );
        }
    }
}

fn pointer_mode(center: Pos2, pointer: Pos2, value: f64, angle_offset: f64) -> RotaryMode {
    let pointer_vector = pointer - center;
    let indicator = angle_direction(value + angle_offset);
    if pointer_vector.dot(indicator) >= 0.0 {
        RotaryMode::Absolute
    } else {
        RotaryMode::Relative
    }
}

fn screen_angle(vector: Vec2) -> f64 {
    f64::from(vector.y.atan2(vector.x).to_degrees())
}

fn signed_vector_angle(from: Vec2, to: Vec2) -> f64 {
    signed_angle_between(screen_angle(to), screen_angle(from))
}

fn signed_angle_between(target: f64, source: f64) -> f64 {
    (target - source + 180.0).rem_euclid(360.0) - 180.0
}

#[allow(clippy::cast_possible_truncation)]
fn angle_direction(angle: f64) -> Vec2 {
    Vec2::angled((angle as f32).to_radians())
}

fn nearly_multiple(value: f64, step: f64) -> bool {
    let remainder = (value / step).round().mul_add(step, -value).abs();
    remainder <= 1e-8 * value.abs().max(step).max(1.0)
}

#[allow(clippy::cast_possible_truncation)]
fn arc_points(center: Pos2, radius: f32, start: f64, end: f64) -> Vec<Pos2> {
    let sweep = end - start;
    let segments = ((sweep.abs() / 6.0).ceil() as usize).clamp(2, 96);
    (0..=segments)
        .map(|index| {
            let amount = index as f64 / segments as f64;
            center + angle_direction(sweep.mul_add(amount, start)) * radius
        })
        .collect()
}

fn clamp_along_ray(origin: Pos2, target: Pos2, bounds: egui::Rect) -> Pos2 {
    if bounds.contains(target) {
        return target;
    }
    let direction = target - origin;
    if direction == Vec2::ZERO {
        return bounds.clamp(target);
    }
    let mut amount: f32 = 1.0;
    if direction.x > 0.0 {
        amount = amount.min((bounds.right() - origin.x) / direction.x);
    } else if direction.x < 0.0 {
        amount = amount.min((bounds.left() - origin.x) / direction.x);
    }
    if direction.y > 0.0 {
        amount = amount.min((bounds.bottom() - origin.y) / direction.y);
    } else if direction.y < 0.0 {
        amount = amount.min((bounds.top() - origin.y) / direction.y);
    }
    bounds.clamp(origin + direction * amount.clamp(0.0, 1.0))
}

fn display_angle(value: f64) -> String {
    let revolutions = (value / 360.0).trunc() as i64;
    let rotation = value - revolutions as f64 * 360.0;
    if revolutions == 0 {
        format!("{rotation:.1}°")
    } else {
        format!("{revolutions}x {rotation:.1}°")
    }
}

/// Responsive composition of a Rotary and Number field.
pub struct Angle<'a> {
    id: ParamId,
    value: &'a mut f64,
    snap: f64,
    angle_offset: f64,
    enabled: bool,
    invalid: bool,
}

impl<'a> Angle<'a> {
    pub fn new(id: ParamId, value: &'a mut f64) -> Self {
        Self {
            id,
            value,
            snap: 45.0,
            angle_offset: -90.0,
            enabled: true,
            invalid: false,
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

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        ui.horizontal(|ui| {
            let rotary = Rotary::new(self.id, self.value)
                .snap(self.snap)
                .angle_offset(self.angle_offset)
                .enabled(self.enabled)
                .invalid(self.invalid)
                .show(ui, context);
            let number = Number::new(self.id, self.value)
                .snap(self.snap)
                .precision(4)
                .suffix("°")
                .width(207.0)
                .bar(false)
                .enabled(self.enabled)
                .invalid(self.invalid)
                .show(ui, context)
                .response;
            rotary.union(number)
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::{display_angle, signed_angle_between};

    #[test]
    fn signed_angles_take_the_short_path() {
        assert!((signed_angle_between(10.0, 350.0) - 20.0).abs() < f64::EPSILON);
        assert!((signed_angle_between(350.0, 10.0) + 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn angle_labels_include_complete_revolutions() {
        assert_eq!(display_angle(45.0), "45.0°");
        assert_eq!(display_angle(450.0), "1x 90.0°");
        assert_eq!(display_angle(-450.0), "-1x -90.0°");
    }
}
