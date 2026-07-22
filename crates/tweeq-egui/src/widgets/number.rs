use egui::{
    Align2, Color32, CornerRadius, CursorGrab, CursorIcon, FontFamily, FontId, Key, LayerId, Order,
    Response, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, ViewportCommand,
};
use tweeq_core::{
    EditOperation, GestureModifiers, NumberConstraints, ParamId, ParamKind, quantize,
};

use crate::TweeqContext;

/// Cursor behavior requested for unbounded number scrubbing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PointerPolicy {
    /// Use ordinary pointer motion without grabbing the cursor.
    #[default]
    Disabled,
    /// Request a locked, hidden cursor and fall back to ordinary motion.
    TryLocked,
}

/// Result of showing a Tweeq widget.
pub struct TweakResponse {
    /// Underlying egui response.
    pub response: Response,
    /// Value changed during this frame.
    pub changed: bool,
    /// An edit session committed during this frame.
    pub committed: bool,
    /// An edit session was cancelled during this frame.
    pub cancelled: bool,
}

impl TweakResponse {
    /// Returns whether the widget changed its bound value.
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }

    /// Returns whether an edit transaction committed.
    #[must_use]
    pub const fn committed(&self) -> bool {
        self.committed
    }
}

/// Tweeq numeric field with click-to-edit and drag-to-tweak interaction.
pub struct Number<'a> {
    id: ParamId,
    value: &'a mut f64,
    constraints: NumberConstraints,
    snap: f64,
    prefix: &'a str,
    suffix: &'a str,
    default: Option<f64>,
    width: f32,
    show_bar: bool,
    enabled: bool,
    pointer_policy: PointerPolicy,
}

impl<'a> Number<'a> {
    /// Creates a numeric field for a stable parameter ID.
    pub fn new(id: ParamId, value: &'a mut f64) -> Self {
        Self {
            id,
            value,
            constraints: NumberConstraints::default(),
            snap: 10.0,
            prefix: "",
            suffix: "",
            default: None,
            width: 240.0,
            show_bar: true,
            enabled: true,
            pointer_policy: PointerPolicy::Disabled,
        }
    }

    /// Sets an inclusive display and clamp range.
    #[must_use]
    pub fn range(mut self, range: std::ops::RangeInclusive<f64>) -> Self {
        self.constraints.min = Some(*range.start());
        self.constraints.max = Some(*range.end());
        self
    }

    /// Sets the committed quantization and arrow-key increment.
    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        self.constraints.step = (step > 0.0).then_some(step);
        self
    }

    /// Sets the Q-key snap interval and Shift speed multiplier.
    #[must_use]
    pub fn snap(mut self, snap: f64) -> Self {
        if snap.is_finite() && snap > 0.0 {
            self.snap = snap;
        }
        self
    }

    /// Sets the maximum displayed decimal places.
    #[must_use]
    pub fn precision(mut self, precision: u8) -> Self {
        self.constraints.precision = precision.min(12);
        self
    }

    /// Sets display text before the number.
    #[must_use]
    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    /// Sets display text after the number.
    #[must_use]
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    /// Sets the value offered by the context-menu reset action.
    #[must_use]
    pub fn default_value(mut self, value: f64) -> Self {
        self.default = Some(value);
        self
    }

    /// Sets the field width.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(48.0);
        self
    }

    /// Enables or disables the range bar.
    #[must_use]
    pub fn bar(mut self, show: bool) -> Self {
        self.show_bar = show;
        self
    }

    /// Enables or disables interaction.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets cursor-grab behavior for unbounded fields.
    #[must_use]
    pub fn pointer_policy(mut self, policy: PointerPolicy) -> Self {
        self.pointer_policy = policy;
        self
    }

    /// Shows the field and updates the bound value.
    #[allow(clippy::too_many_lines)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> TweakResponse {
        context.register_number(self.id, *self.value);
        let theme = context.theme().clone();
        let initial_buffer = format_number(*self.value, self.constraints.precision);
        let mut state = context.take_number_state(self.id, *self.value, initial_buffer);
        let sense = if self.enabled && !state.editing {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(self.width, theme.input_height), sense);
        if self.enabled && !state.editing {
            response = response.on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);
        }

        let mut changed = false;
        let mut committed = false;
        let mut cancelled = false;
        let command_modifier = ui.input(|input| input.modifiers.command || input.modifiers.ctrl);
        let shift_modifier = ui.input(|input| input.modifiers.shift);

        if self.enabled && response.clicked() && !state.editing {
            context.activate_selection(
                self.id,
                ParamKind::Number,
                shift_modifier,
                command_modifier,
            );
            state.captured = *self.value;
            state.buffer = format_number(*self.value, self.constraints.precision);
            state.invalid = false;
            state.editing = true;
            state.session = Some(context.start_edit(self.id, ParamKind::Number));
        }

        if self.enabled && response.drag_started() {
            context.activate_selection(
                self.id,
                ParamKind::Number,
                shift_modifier,
                command_modifier,
            );
            state.captured = *self.value;
            state.gesture.reset();
            state.last_drag_total = Vec2::ZERO;
            state.virtual_position = response.interact_pointer_pos();
            state.session = Some(context.start_edit(self.id, ParamKind::Number));
            if self.pointer_policy == PointerPolicy::TryLocked && !self.has_complete_range() {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::CursorGrab(CursorGrab::Locked));
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::CursorVisible(false));
            }
        }

        if self.enabled
            && response.dragged()
            && let Some(session) = state.session
        {
            let motion =
                if self.pointer_policy == PointerPolicy::TryLocked && !self.has_complete_range() {
                    response.drag_motion()
                } else {
                    let total = response.total_drag_delta().unwrap_or_default();
                    let frame_motion = total - state.last_drag_total;
                    state.last_drag_total = total;
                    frame_motion
                };
            if let Some(position) = &mut state.virtual_position {
                *position += motion;
            }
            let base_speed = self.base_speed(rect.width());
            let modifiers = GestureModifiers {
                fine: ui.input(|input| input.modifiers.alt),
                fast: shift_modifier,
                snap: ui.input(|input| input.key_down(Key::Q)),
            };
            let update = state.gesture.update(
                [motion.x, motion.y],
                base_speed,
                modifiers,
                self.snap,
                self.speed_range(rect.width()),
            );
            let mut candidate = state.captured + update.accumulated_delta;
            if update.snap {
                candidate = quantize(candidate, self.snap, self.constraints.min.unwrap_or(0.0));
            }
            candidate = self.constraints.validate(candidate, false).value;
            if values_differ(candidate, *self.value) {
                *self.value = candidate;
                changed = true;
                response.mark_changed();
            }
            context.update_edit(
                session,
                EditOperation::AddNumber(candidate - state.captured),
            );
            paint_tweak_overlay(ui, &theme, &state, candidate, self.constraints.precision);
        }

        if self.enabled && response.drag_stopped() {
            if let Some(session) = state.session.take() {
                let validated = self.constraints.validate(*self.value, true).value;
                if values_differ(validated, *self.value) {
                    *self.value = validated;
                    context.update_edit(session, EditOperation::SetNumber(validated));
                    changed = true;
                    response.mark_changed();
                }
                context.finish_edit(session, true);
                committed = true;
            }
            state.buffer = format_number(*self.value, self.constraints.precision);
            state.virtual_position = None;
            release_pointer(ui);
        }

        paint_number_background(
            ui,
            &theme,
            rect,
            &response,
            NumberPaintState {
                value: *self.value,
                constraints: self.constraints,
                show_bar: self.show_bar,
                selected: context.is_selected(self.id),
                invalid: state.invalid,
                enabled: self.enabled,
            },
        );

        if state.editing {
            let edit = TextEdit::singleline(&mut state.buffer)
                .id_source(("tweeq-number-edit", self.id.as_u64()))
                .font(FontId::new(12.0, FontFamily::Monospace))
                .horizontal_align(egui::Align::Center)
                .frame(egui::Frame::NONE)
                .margin(Vec2::new(5.0, 3.0));
            let edit_response = ui.put(rect, edit);
            if response.clicked() {
                edit_response.request_focus();
            }

            state.invalid = state.buffer.trim().parse::<f64>().is_err();
            let escape =
                edit_response.has_focus() && ui.input(|input| input.key_pressed(Key::Escape));
            let enter =
                edit_response.has_focus() && ui.input(|input| input.key_pressed(Key::Enter));
            let arrow =
                if edit_response.has_focus() && ui.input(|input| input.key_pressed(Key::ArrowUp)) {
                    1.0
                } else if edit_response.has_focus()
                    && ui.input(|input| input.key_pressed(Key::ArrowDown))
                {
                    -1.0
                } else {
                    0.0
                };

            if arrow != 0.0 {
                let base = state.buffer.trim().parse::<f64>().unwrap_or(*self.value);
                let mut increment = self.constraints.step.unwrap_or(1.0);
                if ui.input(|input| input.modifiers.shift) {
                    increment *= self.snap;
                }
                if ui.input(|input| input.modifiers.alt) {
                    increment *= 0.1;
                }
                let candidate = self
                    .constraints
                    .validate(base + arrow * increment, true)
                    .value;
                *self.value = candidate;
                state.buffer = format_number(candidate, self.constraints.precision);
                state.invalid = false;
                changed = true;
                if let Some(session) = state.session {
                    context.update_edit(session, EditOperation::SetNumber(candidate));
                }
            }

            if escape {
                *self.value = state.captured;
                if let Some(session) = state.session.take() {
                    context.finish_edit(session, false);
                }
                state.editing = false;
                state.invalid = false;
                state.buffer = format_number(*self.value, self.constraints.precision);
                cancelled = true;
                edit_response.surrender_focus();
            } else if enter || edit_response.lost_focus() {
                let parsed = state.buffer.trim().parse::<f64>();
                if let Ok(value) = parsed {
                    let validated = self.constraints.validate(value, true).value;
                    *self.value = validated;
                    state.buffer = format_number(validated, self.constraints.precision);
                    changed = values_differ(validated, state.captured);
                    if let Some(session) = state.session.take() {
                        context.update_edit(session, EditOperation::SetNumber(validated));
                        context.finish_edit(session, true);
                    }
                    committed = true;
                } else {
                    *self.value = state.captured;
                    if let Some(session) = state.session.take() {
                        context.finish_edit(session, false);
                    }
                    cancelled = true;
                }
                state.editing = false;
                state.invalid = false;
                if enter {
                    edit_response.surrender_focus();
                }
            }
            response = response.union(edit_response);
        } else {
            let color = if self.enabled {
                theme.text
            } else {
                theme.text_muted
            };
            let display = format!(
                "{}{}{}",
                self.prefix,
                format_number(*self.value, self.constraints.precision),
                self.suffix
            );
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                display,
                FontId::new(12.0, FontFamily::Monospace),
                color,
            );
        }

        if let Some(default) = self.default {
            response.context_menu(|ui| {
                if ui.button("Reset to default").clicked() {
                    let session = context.start_edit(self.id, ParamKind::Number);
                    *self.value = self.constraints.validate(default, true).value;
                    state.buffer = format_number(*self.value, self.constraints.precision);
                    context.update_edit(session, EditOperation::SetNumber(*self.value));
                    context.finish_edit(session, true);
                    changed = true;
                    committed = true;
                    ui.close();
                }
            });
        }

        context.put_number_state(self.id, state);
        TweakResponse {
            response,
            changed,
            committed,
            cancelled,
        }
    }

    fn has_complete_range(&self) -> bool {
        matches!((self.constraints.min, self.constraints.max), (Some(min), Some(max)) if min < max)
    }

    fn base_speed(&self, width: f32) -> f64 {
        match (self.constraints.min, self.constraints.max) {
            (Some(min), Some(max)) if max > min && width > 0.0 => (max - min) / f64::from(width),
            _ => self.constraints.step.map_or(1.0, |step| step / 20.0),
        }
    }

    fn speed_range(&self, width: f32) -> std::ops::RangeInclusive<f64> {
        let minimum = self.constraints.step.map_or_else(
            || 10_f64.powi(-i32::from(self.constraints.precision)),
            |step| (step / f64::from(width.max(1.0))).max(1e-9),
        );
        minimum..=if self.has_complete_range() {
            1.0
        } else {
            1_000.0
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy)]
struct NumberPaintState {
    value: f64,
    constraints: NumberConstraints,
    show_bar: bool,
    selected: bool,
    invalid: bool,
    enabled: bool,
}

#[allow(clippy::cast_possible_truncation)]
fn paint_number_background(
    ui: &Ui,
    theme: &crate::TweeqTheme,
    rect: egui::Rect,
    response: &Response,
    state: NumberPaintState,
) {
    let fill = if response.hovered() && state.enabled {
        theme.input_hover
    } else {
        theme.input
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme.input_radius), fill);

    if state.show_bar
        && let (Some(minimum), Some(maximum)) = (state.constraints.min, state.constraints.max)
        && maximum > minimum
    {
        let fraction = ((state.value - minimum) / (maximum - minimum)).clamp(0.0, 1.0) as f32;
        let bar = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.left() + rect.width() * fraction, rect.bottom()),
        );
        ui.painter().rect_filled(
            bar,
            CornerRadius::same(theme.input_radius),
            theme.accent.gamma_multiply(0.28),
        );
    }

    let stroke = if state.invalid {
        Stroke::new(1.0, Color32::from_rgb(238, 79, 87))
    } else if state.selected {
        Stroke::new(1.0, theme.accent)
    } else {
        Stroke::new(1.0, theme.border)
    };
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme.input_radius),
        stroke,
        StrokeKind::Inside,
    );
}

fn paint_tweak_overlay(
    ui: &Ui,
    theme: &crate::TweeqTheme,
    state: &crate::context::NumberState,
    value: f64,
    precision: u8,
) {
    let Some(position) = state.virtual_position else {
        return;
    };
    let painter = ui.ctx().layer_painter(LayerId::new(
        Order::Foreground,
        egui::Id::new("tweeq-tweak-overlay"),
    ));
    let origin = egui::pos2(ui.ctx().content_rect().center().x, position.y);
    painter.line_segment([origin, position], Stroke::new(1.5, theme.accent));
    painter.circle_filled(position, 4.0, theme.accent);
    painter.text(
        position + Vec2::new(10.0, -12.0),
        Align2::LEFT_BOTTOM,
        format_number(value, precision),
        FontId::new(13.0, FontFamily::Monospace),
        theme.text,
    );
}

fn release_pointer(ui: &Ui) {
    ui.ctx()
        .send_viewport_cmd(ViewportCommand::CursorGrab(CursorGrab::None));
    ui.ctx()
        .send_viewport_cmd(ViewportCommand::CursorVisible(true));
}

fn format_number(value: f64, precision: u8) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let mut output = format!("{value:.precision$}", precision = usize::from(precision));
    if output.contains('.') {
        while output.ends_with('0') {
            output.pop();
        }
        if output.ends_with('.') {
            output.pop();
        }
    }
    if output == "-0" {
        "0".to_owned()
    } else {
        output
    }
}

fn values_differ(left: f64, right: f64) -> bool {
    (left - right).abs() > f64::EPSILON * left.abs().max(right.abs()).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::format_number;

    #[test]
    fn formats_without_redundant_zeroes() {
        assert_eq!(format_number(1.25, 4), "1.25");
        assert_eq!(format_number(-0.0, 4), "0");
    }
}
