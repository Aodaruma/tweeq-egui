use egui::{
    Align2, Color32, CornerRadius, CursorGrab, CursorIcon, FontFamily, FontId, Key, Response,
    Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, ViewportCommand,
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

/// Range-bar configuration for a numeric field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberBar {
    /// Do not draw a range bar.
    Hidden,
    /// Fill between this origin and the current value.
    Origin(f64),
}

impl From<bool> for NumberBar {
    fn from(show: bool) -> Self {
        if show {
            Self::Origin(0.0)
        } else {
            Self::Hidden
        }
    }
}

impl From<f64> for NumberBar {
    fn from(origin: f64) -> Self {
        Self::Origin(origin)
    }
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
    bar: NumberBar,
    enabled: bool,
    invalid: bool,
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
            bar: NumberBar::Origin(0.0),
            enabled: true,
            invalid: false,
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

    /// Sets the optional lower bound independently from the upper bound.
    #[must_use]
    pub fn min(mut self, minimum: f64) -> Self {
        self.constraints.min = minimum.is_finite().then_some(minimum);
        self
    }

    /// Sets the optional upper bound independently from the lower bound.
    #[must_use]
    pub fn max(mut self, maximum: f64) -> Self {
        self.constraints.max = maximum.is_finite().then_some(maximum);
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

    /// Enables or disables lower-bound clamping while retaining its bar range.
    #[must_use]
    pub fn clamp_min(mut self, clamp: bool) -> Self {
        self.constraints.clamp_min = clamp;
        self
    }

    /// Enables or disables upper-bound clamping while retaining its bar range.
    #[must_use]
    pub fn clamp_max(mut self, clamp: bool) -> Self {
        self.constraints.clamp_max = clamp;
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
    pub fn bar(mut self, bar: impl Into<NumberBar>) -> Self {
        self.bar = bar.into();
        self
    }

    /// Enables or disables interaction.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Forces invalid-state styling in addition to parse/validation errors.
    #[must_use]
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
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
            state.drag_base = *self.value;
            state.gesture.reset();
            state.last_drag_total = Vec2::ZERO;
            let press_origin = ui
                .input(|input| input.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos());
            state.virtual_position = press_origin;
            state.session = Some(context.start_edit(self.id, ParamKind::Number));
            if self.bar_visible()
                && self.value_inside_range(*self.value)
                && let (Some(position), Some(minimum), Some(maximum)) =
                    (press_origin, self.constraints.min, self.constraints.max)
            {
                let fraction = ((position.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                state.drag_base = f64::from(fraction).mul_add(maximum - minimum, minimum);
                *self.value = self.constraints.validate(state.drag_base, false).value;
                changed = values_differ(*self.value, state.captured);
                if changed {
                    response.mark_changed();
                }
            }
            if self.pointer_policy == PointerPolicy::TryLocked && !self.bar_visible() {
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
            let motion = if self.pointer_policy == PointerPolicy::TryLocked && !self.bar_visible() {
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
            let mut candidate = state.drag_base + update.accumulated_delta;
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
                bar: self.bar,
                selected: context.is_selected(self.id),
                invalid: self.invalid || state.invalid,
                enabled: self.enabled,
                tweaking: response.dragged(),
                gesture_speed: state.gesture.speed(),
                horizontal_weight: state.gesture.horizontal_weight(),
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
                let increment = arrow_increment(
                    self.constraints,
                    self.snap,
                    ui.input(|input| input.modifiers.shift),
                    ui.input(|input| input.modifiers.alt),
                );
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
                if response.dragged() {
                    format_number_fixed(
                        *self.value,
                        self.display_precision(rect.width(), *self.value, true, &state, ui),
                    )
                } else {
                    format_number(
                        *self.value,
                        self.display_precision(rect.width(), *self.value, false, &state, ui),
                    )
                },
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

    fn bar_visible(&self) -> bool {
        !matches!(self.bar, NumberBar::Hidden) && self.has_complete_range()
    }

    fn value_inside_range(&self, value: f64) -> bool {
        matches!((self.constraints.min, self.constraints.max), (Some(min), Some(max)) if min <= value && value <= max)
    }

    fn base_speed(&self, width: f32) -> f64 {
        match (self.bar, self.constraints.min, self.constraints.max) {
            (NumberBar::Origin(_), Some(min), Some(max)) if max > min && width > 0.0 => {
                (max - min) / f64::from(width)
            }
            _ => self.constraints.step.map_or(1.0, |step| step / 20.0),
        }
    }

    fn speed_range(&self, width: f32) -> std::ops::RangeInclusive<f64> {
        let precision = if self.bar_visible()
            && let (Some(step), Some(minimum), Some(maximum)) = (
                self.constraints.step,
                self.constraints.min,
                self.constraints.max,
            ) {
            precision_of(f64::from(width) * step / (maximum - minimum))
        } else {
            self.constraints.precision
        };
        let minimum = 10_f64.powi(-i32::from(precision));
        minimum..=if self.bar_visible() { 1.0 } else { 1_000.0 }
    }

    fn display_precision(
        &self,
        width: f32,
        value: f64,
        tweaking: bool,
        state: &crate::context::NumberState,
        ui: &Ui,
    ) -> u8 {
        if let Some(step) = self.constraints.step {
            return precision_of(step);
        }
        let value_precision = precision_of(value);
        let slider_precision = if self.bar_visible()
            && let (Some(minimum), Some(maximum)) = (self.constraints.min, self.constraints.max)
        {
            precision_of((maximum - minimum).abs() / f64::from(width.max(1.0)))
        } else {
            0
        };
        if tweaking {
            let modifier = modifier_speed(ui, self.snap);
            return value_precision
                .max(slider_precision)
                .max(precision_of(state.gesture.speed() * modifier));
        }
        self.constraints
            .precision
            .min(value_precision.max(slider_precision))
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy)]
struct NumberPaintState {
    value: f64,
    constraints: NumberConstraints,
    bar: NumberBar,
    selected: bool,
    invalid: bool,
    enabled: bool,
    tweaking: bool,
    gesture_speed: f64,
    horizontal_weight: f64,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
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

    let complete_range = match (state.constraints.min, state.constraints.max) {
        (Some(minimum), Some(maximum)) if maximum > minimum => Some((minimum, maximum)),
        _ => None,
    };

    if let (NumberBar::Origin(origin), Some((minimum, maximum))) = (state.bar, complete_range) {
        let origin_fraction = ((origin - minimum) / (maximum - minimum)).clamp(0.0, 1.0) as f32;
        let value_fraction = ((state.value - minimum) / (maximum - minimum)).clamp(0.0, 1.0) as f32;
        let left = rect.left() + rect.width() * origin_fraction.min(value_fraction);
        let right = rect.left() + rect.width() * origin_fraction.max(value_fraction);
        let bar = egui::Rect::from_min_max(
            egui::pos2(left, rect.top()),
            egui::pos2(right, rect.bottom()),
        );
        let bar_color = if state.enabled {
            theme
                .accent
                .gamma_multiply(if response.hovered() { 0.36 } else { 0.25 })
        } else {
            theme.input
        };
        ui.painter()
            .rect_filled(bar, CornerRadius::same(theme.input_radius), bar_color);
    }

    if let (Some((minimum, maximum)), Some(step)) = (complete_range, state.constraints.step) {
        let gap = f64::from(rect.width()) * step / (maximum - minimum);
        if gap >= 10.0 && gap.is_finite() {
            let mut x = rect.left() + gap as f32;
            while x < rect.right() - 0.5 {
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(1.0, theme.border.gamma_multiply(0.68)),
                );
                x += gap as f32;
            }
        }
    }

    let stepped_and_clamped = state.constraints.step.is_some()
        && state.constraints.clamp_min
        && state.constraints.clamp_max
        && complete_range.is_some();
    if state.tweaking && !stepped_and_clamped {
        paint_number_tweak_scales(ui, theme, rect, state, complete_range);
    }

    if let (NumberBar::Origin(_), Some((minimum, maximum))) = (state.bar, complete_range) {
        if (minimum..=maximum).contains(&state.value) {
            let fraction = ((state.value - minimum) / (maximum - minimum)) as f32;
            let x = rect.left() + rect.width() * fraction;
            let thickness = if response.hovered() || state.tweaking {
                3.0
            } else {
                1.0
            };
            ui.painter().line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                Stroke::new(
                    thickness,
                    theme
                        .accent
                        .gamma_multiply(if response.hovered() || state.tweaking {
                            1.0
                        } else {
                            0.3
                        }),
                ),
            );
        } else {
            let at_left = state.value < minimum;
            let x = if at_left { rect.left() } else { rect.right() };
            let direction = if at_left { 1.0 } else { -1.0 };
            let center = egui::pos2(x, rect.center().y);
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    center,
                    center + Vec2::new(direction * 5.0, -4.0),
                    center + Vec2::new(direction * 5.0, 4.0),
                ],
                theme
                    .accent
                    .gamma_multiply(if state.tweaking { 1.0 } else { 0.3 }),
                Stroke::NONE,
            ));
        }
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

#[allow(clippy::cast_possible_truncation)]
fn paint_number_tweak_scales(
    ui: &Ui,
    theme: &crate::TweeqTheme,
    rect: egui::Rect,
    state: NumberPaintState,
    complete_range: Option<(f64, f64)>,
) {
    let speed = state.gesture_speed.max(1e-12);
    let dash_offset = if let Some((minimum, maximum)) = complete_range {
        ((state.value - minimum) / (maximum - minimum)) * f64::from(rect.width())
    } else {
        f64::from(rect.width()) * 0.5 - state.value / speed
    };
    for offset in 0..3 {
        let gesture_precision = (-speed.log10() + f64::from(offset)).rem_euclid(3.0);
        let opacity = smoothstep(1.0, 2.0, gesture_precision).sqrt() as f32;
        if opacity <= 0.01 {
            continue;
        }
        let gap = 10_f64.powf(gesture_precision).clamp(1.0, 1_000.0);
        let radius = (2.0 - state.horizontal_weight as f32 * 0.5).max(1.0);
        let mut x = (-dash_offset).rem_euclid(gap);
        while x <= f64::from(rect.width()) {
            ui.painter().circle_filled(
                egui::pos2(rect.left() + x as f32, rect.center().y),
                radius,
                theme.accent.gamma_multiply(opacity),
            );
            x += gap;
        }
    }
}

fn release_pointer(ui: &Ui) {
    ui.ctx()
        .send_viewport_cmd(ViewportCommand::CursorGrab(CursorGrab::None));
    ui.ctx()
        .send_viewport_cmd(ViewportCommand::CursorVisible(true));
}

fn modifier_speed(ui: &Ui, fast_multiplier: f64) -> f64 {
    let fine = if ui.input(|input| input.modifiers.alt) {
        0.1
    } else {
        1.0
    };
    let fast = if ui.input(|input| input.modifiers.shift) {
        fast_multiplier.max(1.0)
    } else {
        1.0
    };
    fine * fast
}

fn arrow_increment(
    constraints: NumberConstraints,
    fast_multiplier: f64,
    fast: bool,
    fine: bool,
) -> f64 {
    if let Some(step) = constraints.step {
        return step * if fast { fast_multiplier.max(1.0) } else { 1.0 };
    }
    let mut increment =
        (if fine { 0.1 } else { 1.0 }) * if fast { fast_multiplier.max(1.0) } else { 1.0 };
    if let (Some(minimum), Some(maximum)) = (constraints.min, constraints.max)
        && maximum - minimum <= 1.0
    {
        increment *= 0.1;
    }
    increment
}

fn precision_of(value: f64) -> u8 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let value = value.abs();
    for precision in 0..=12_u8 {
        let scale = 10_f64.powi(i32::from(precision));
        let rounded = (value * scale).round() / scale;
        if (rounded - value).abs() <= 1e-10 * value.max(1.0) {
            return precision;
        }
    }
    12
}

fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let amount = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

fn format_number_fixed(value: f64, precision: u8) -> String {
    if value.is_finite() {
        format!("{value:.precision$}", precision = usize::from(precision))
    } else {
        value.to_string()
    }
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
    use super::{arrow_increment, format_number, format_number_fixed, precision_of};
    use tweeq_core::NumberConstraints;

    #[test]
    fn formats_without_redundant_zeroes() {
        assert_eq!(format_number(1.25, 4), "1.25");
        assert_eq!(format_number(-0.0, 4), "0");
        assert_eq!(format_number_fixed(1.25, 4), "1.2500");
    }

    #[test]
    fn derives_decimal_precision_from_step_or_speed() {
        assert_eq!(precision_of(1.0), 0);
        assert_eq!(precision_of(0.25), 2);
        assert_eq!(precision_of(0.001), 3);
    }

    #[test]
    fn focused_arrow_modifiers_match_vue_semantics() {
        let unstepped = NumberConstraints::default();
        assert!(close(arrow_increment(unstepped, 10.0, false, false), 1.0));
        assert!(close(arrow_increment(unstepped, 10.0, true, false), 10.0));
        assert!(close(arrow_increment(unstepped, 10.0, false, true), 0.1));

        let stepped = NumberConstraints {
            step: Some(0.25),
            ..NumberConstraints::default()
        };
        assert!(close(arrow_increment(stepped, 10.0, false, true), 0.25));
        assert!(close(arrow_increment(stepped, 10.0, true, false), 2.5));
    }

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < 1e-12
    }
}
