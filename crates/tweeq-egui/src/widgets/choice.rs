use egui::{Color32, CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use tweeq_core::{EditOperation, ParamId, ParamKind};

use crate::{TweeqContext, TweeqTheme};

const INVALID: Color32 = Color32::from_rgb(232, 78, 88);

/// String-backed, segmented radio group with drag-to-select behavior.
pub struct Radio<'a> {
    id: ParamId,
    value: &'a mut String,
    options: &'a [&'a str],
    width: f32,
    enabled: bool,
    invalid: bool,
}

impl<'a> Radio<'a> {
    pub fn new(id: ParamId, value: &'a mut String, options: &'a [&'a str]) -> Self {
        Self {
            id,
            value,
            options,
            width: 240.0,
            enabled: true,
            invalid: false,
        }
    }

    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let theme = context.theme().clone();
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(self.width, theme.input_height),
            if self.enabled {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            },
        );
        response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        let before = self.value.clone();

        if self.enabled && !self.options.is_empty() {
            if response.clicked() || response.dragged() {
                if let Some(pointer) = ui.input(|input| input.pointer.interact_pos()) {
                    let index = option_at_x(rect, pointer.x, self.options.len());
                    self.options[index].clone_into(self.value);
                    response.request_focus();
                }
            }
            if response.has_focus() {
                let previous = ui.input(|input| {
                    input.key_pressed(egui::Key::ArrowLeft) || input.key_pressed(egui::Key::ArrowUp)
                });
                let next = ui.input(|input| {
                    input.key_pressed(egui::Key::ArrowRight)
                        || input.key_pressed(egui::Key::ArrowDown)
                });
                if previous {
                    step_option_wrapped(self.value, self.options, -1);
                } else if next {
                    step_option_wrapped(self.value, self.options, 1);
                }
            }
        }

        paint_radio(
            ui,
            rect,
            &response,
            self.value,
            self.options,
            self.enabled,
            self.invalid,
            &theme,
        );
        if before != *self.value {
            response.mark_changed();
            emit_string(context, self.id, self.value);
        }
        response
    }
}

/// String-backed dropdown.
pub struct Dropdown<'a> {
    id: ParamId,
    value: &'a mut String,
    options: &'a [&'a str],
    width: f32,
    enabled: bool,
    invalid: bool,
}

impl<'a> Dropdown<'a> {
    pub fn new(id: ParamId, value: &'a mut String, options: &'a [&'a str]) -> Self {
        Self {
            id,
            value,
            options,
            width: 240.0,
            enabled: true,
            invalid: false,
        }
    }

    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let before = self.value.clone();
        let theme = context.theme().clone();
        let mut inner_response = None;
        let scoped = ui.add_enabled_ui(self.enabled, |ui| {
            let inner = egui::ComboBox::from_id_salt(("tweeq-dropdown", self.id.as_u64()))
                .width(self.width)
                .selected_text(self.value.as_str())
                .show_ui(ui, |ui| {
                    for option in self.options {
                        ui.selectable_value(self.value, (*option).to_owned(), *option);
                    }
                });
            inner_response = Some(inner.response);
        });
        let mut response = inner_response.unwrap_or(scoped.response);

        if self.enabled && response.has_focus() {
            let previous = ui.input(|input| input.key_pressed(egui::Key::ArrowUp));
            let next = ui.input(|input| input.key_pressed(egui::Key::ArrowDown));
            if previous {
                step_option_wrapped(self.value, self.options, -1);
            } else if next {
                step_option_wrapped(self.value, self.options, 1);
            }
            let typed = ui.input(|input| {
                input.events.iter().find_map(|event| match event {
                    egui::Event::Text(text) if !text.trim().is_empty() => Some(text.clone()),
                    _ => None,
                })
            });
            if let Some(prefix) = typed {
                if let Some(option) = self
                    .options
                    .iter()
                    .find(|option| option.to_lowercase().starts_with(&prefix.to_lowercase()))
                {
                    self.value.clear();
                    self.value.push_str(option);
                }
            }
        }

        if self.invalid {
            ui.painter().rect_stroke(
                response.rect,
                CornerRadius::same(theme.input_radius),
                Stroke::new(1.0, INVALID),
                StrokeKind::Inside,
            );
        }
        if before != *self.value {
            response.mark_changed();
            emit_string(context, self.id, self.value);
        }
        response
    }
}

/// Tweeq horizontal slot-machine picker.
pub struct Drum<'a> {
    id: ParamId,
    value: &'a mut String,
    options: &'a [&'a str],
    width: f32,
    cell_width: f32,
    prefix: &'a str,
    suffix: &'a str,
    enabled: bool,
    invalid: bool,
}

#[derive(Clone, Default)]
struct DrumMemory {
    drag_start: f32,
    wheel_accum: f32,
    type_buffer: String,
    typed_at: f64,
}

impl<'a> Drum<'a> {
    pub fn new(id: ParamId, value: &'a mut String, options: &'a [&'a str]) -> Self {
        Self {
            id,
            value,
            options,
            width: 240.0,
            cell_width: 0.0,
            prefix: "",
            suffix: "",
            enabled: true,
            invalid: false,
        }
    }

    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Overrides the automatically measured equal cell width.
    #[must_use]
    pub const fn cell_width(mut self, cell_width: f32) -> Self {
        self.cell_width = cell_width;
        self
    }

    #[must_use]
    pub const fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    #[must_use]
    pub const fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let theme = context.theme().clone();
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(self.width, theme.input_height),
            if self.enabled {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            },
        );
        response = response.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
        let before = self.value.clone();
        let active = option_index(self.value, self.options);
        let cell_width = drum_cell_width(
            ui,
            self.options,
            self.prefix,
            self.suffix,
            self.cell_width,
            self.width,
            theme.input_height,
        );
        let memory_id = response.id.with(("tweeq-drum", self.id.as_u64()));
        let mut memory = ui
            .data(|data| data.get_temp::<DrumMemory>(memory_id))
            .unwrap_or_default();
        let mut display_index = active as f32;

        if self.enabled && !self.options.is_empty() {
            if response.drag_started() {
                memory.drag_start = active as f32;
                response.request_focus();
            }
            if response.dragged() {
                let delta = response.total_drag_delta().unwrap_or_default().x;
                display_index =
                    (memory.drag_start - delta / 40.0).clamp(0.0, (self.options.len() - 1) as f32);
                set_option(self.value, self.options, display_index.round() as usize);
            } else if response.clicked()
                && let Some(pointer) = ui.input(|input| input.pointer.interact_pos())
            {
                let offset = ((pointer.x - rect.center().x) / cell_width).round() as isize;
                set_option_clamped(self.value, self.options, active as isize + offset);
                response.request_focus();
            }

            if response.hovered() {
                let wheel = ui.input(|input| {
                    if input.smooth_scroll_delta.x.abs() > input.smooth_scroll_delta.y.abs() {
                        input.smooth_scroll_delta.x
                    } else {
                        input.smooth_scroll_delta.y
                    }
                });
                memory.wheel_accum += wheel;
                while memory.wheel_accum.abs() >= 24.0 {
                    let direction = memory.wheel_accum.signum() as isize;
                    set_option_clamped(
                        self.value,
                        self.options,
                        option_index(self.value, self.options) as isize + direction,
                    );
                    memory.wheel_accum -= direction as f32 * 24.0;
                }
            }

            if response.has_focus() {
                if ui.input(|input| {
                    input.key_pressed(egui::Key::ArrowLeft) || input.key_pressed(egui::Key::ArrowUp)
                }) {
                    set_option_clamped(self.value, self.options, active as isize - 1);
                } else if ui.input(|input| {
                    input.key_pressed(egui::Key::ArrowRight)
                        || input.key_pressed(egui::Key::ArrowDown)
                }) {
                    set_option_clamped(self.value, self.options, active as isize + 1);
                }

                let typed = ui.input(|input| {
                    input.events.iter().find_map(|event| match event {
                        egui::Event::Text(text) if text.chars().all(|ch| !ch.is_control()) => {
                            Some(text.clone())
                        }
                        _ => None,
                    })
                });
                if let Some(text) = typed {
                    let now = ui.input(|input| input.time);
                    if now - memory.typed_at > 0.8 {
                        memory.type_buffer.clear();
                    }
                    memory.typed_at = now;
                    memory.type_buffer.push_str(&text.to_lowercase());
                    if let Some(index) = self.options.iter().position(|option| {
                        format!("{}{}{}", self.prefix, option, self.suffix)
                            .to_lowercase()
                            .starts_with(&memory.type_buffer)
                    }) {
                        set_option(self.value, self.options, index);
                    }
                }
            }
        }
        ui.data_mut(|data| data.insert_temp(memory_id, memory));

        if !response.dragged() {
            display_index = option_index(self.value, self.options) as f32;
        }
        paint_drum(
            ui,
            rect,
            &response,
            self.options,
            self.prefix,
            self.suffix,
            option_index(self.value, self.options),
            display_index,
            cell_width,
            self.enabled,
            self.invalid,
            &theme,
        );
        if before != *self.value {
            response.mark_changed();
            emit_string(context, self.id, self.value);
        }
        response
    }
}

fn paint_radio(
    ui: &Ui,
    rect: egui::Rect,
    response: &Response,
    value: &str,
    options: &[&str],
    enabled: bool,
    invalid: bool,
    theme: &TweeqTheme,
) {
    let stroke = focus_or_invalid_stroke(response, enabled, invalid, theme);
    ui.painter().rect(
        rect,
        CornerRadius::same(theme.input_radius),
        if enabled {
            theme.input
        } else {
            Color32::TRANSPARENT
        },
        stroke,
        StrokeKind::Inside,
    );
    if options.is_empty() {
        return;
    }
    let width = rect.width() / options.len() as f32;
    let pointer = ui.input(|input| input.pointer.hover_pos());
    for (index, option) in options.iter().enumerate() {
        let segment = egui::Rect::from_min_max(
            egui::pos2(rect.left() + width * index as f32, rect.top()),
            egui::pos2(rect.left() + width * (index + 1) as f32, rect.bottom()),
        );
        let active = *option == value;
        let hovered = response.hovered() && pointer.is_some_and(|p| segment.contains(p));
        if active || hovered {
            let fill = if active {
                if hovered || response.dragged() {
                    theme.accent_hover
                } else {
                    theme.accent
                }
            } else {
                theme.input_hover
            };
            ui.painter().rect_filled(
                segment.shrink(0.5),
                CornerRadius::same(theme.input_radius),
                fill,
            );
        }
        ui.painter().text(
            segment.center(),
            egui::Align2::CENTER_CENTER,
            option,
            egui::TextStyle::Body.resolve(ui.style()),
            if !enabled {
                theme.text_muted
            } else if active {
                contrast_text(theme.accent)
            } else {
                theme.text
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_drum(
    ui: &Ui,
    rect: egui::Rect,
    response: &Response,
    options: &[&str],
    prefix: &str,
    suffix: &str,
    active: usize,
    display_index: f32,
    cell_width: f32,
    enabled: bool,
    invalid: bool,
    theme: &TweeqTheme,
) {
    ui.painter().rect(
        rect,
        CornerRadius::same(theme.input_radius),
        if !enabled {
            Color32::TRANSPARENT
        } else if response.hovered() {
            theme.input_hover
        } else {
            theme.input
        },
        focus_or_invalid_stroke(response, enabled, invalid, theme),
        StrokeKind::Inside,
    );
    let painter = ui.painter().with_clip_rect(rect);
    let center = rect.center().x;
    painter.line_segment(
        [
            egui::pos2(center, rect.top()),
            egui::pos2(center, rect.bottom()),
        ],
        Stroke::new(1.0, theme.accent),
    );
    for (index, option) in options.iter().enumerate() {
        let x = center + (index as f32 - display_index) * cell_width;
        if x + cell_width * 0.5 < rect.left() || x - cell_width * 0.5 > rect.right() {
            continue;
        }
        let distance = ((x - center).abs() / (rect.width() * 0.5)).clamp(0.0, 1.0);
        let alpha = ((1.0 - distance) * 255.0) as u8;
        let base = if index == active {
            theme.text
        } else {
            theme.text_muted
        };
        let color = if enabled {
            Color32::from_rgba_premultiplied(base.r(), base.g(), base.b(), alpha)
        } else {
            theme.text_muted
        };
        painter.text(
            egui::pos2(x, rect.center().y - 1.0),
            egui::Align2::CENTER_CENTER,
            format!("{prefix}{option}{suffix}"),
            egui::TextStyle::Body.resolve(ui.style()),
            color,
        );
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom() - 4.0),
                egui::pos2(x, rect.bottom() - 2.0),
            ],
            Stroke::new(1.0, color.gamma_multiply(0.5)),
        );
    }
}

fn drum_cell_width(
    ui: &Ui,
    options: &[&str],
    prefix: &str,
    suffix: &str,
    override_width: f32,
    viewport_width: f32,
    input_height: f32,
) -> f32 {
    if override_width > 0.0 {
        return override_width.max(1.0);
    }
    let font = egui::TextStyle::Body.resolve(ui.style());
    let label = options
        .iter()
        .map(|option| {
            ui.painter()
                .layout_no_wrap(
                    format!("{prefix}{option}{suffix}"),
                    font.clone(),
                    Color32::WHITE,
                )
                .size()
                .x
        })
        .fold(input_height, f32::max)
        + 18.0;
    let mut cells = (viewport_width / label).floor() as usize;
    if cells % 2 == 1 {
        cells = cells.saturating_sub(1);
    }
    cells = cells.max(2);
    (viewport_width / cells as f32).min(label + 32.0)
}

fn focus_or_invalid_stroke(
    response: &Response,
    enabled: bool,
    invalid: bool,
    theme: &TweeqTheme,
) -> Stroke {
    if invalid {
        Stroke::new(1.0, INVALID)
    } else if !enabled {
        Stroke::new(1.0, theme.border)
    } else if response.has_focus() {
        Stroke::new(1.0, theme.accent)
    } else {
        Stroke::NONE
    }
}

fn option_at_x(rect: egui::Rect, x: f32, len: usize) -> usize {
    (((x - rect.left()) / rect.width()).clamp(0.0, 0.999_999) * len as f32) as usize
}

fn option_index(value: &str, options: &[&str]) -> usize {
    options
        .iter()
        .position(|option| *option == value)
        .unwrap_or(0)
}

fn set_option(value: &mut String, options: &[&str], index: usize) {
    if let Some(option) = options.get(index) {
        value.clear();
        value.push_str(option);
    }
}

fn set_option_clamped(value: &mut String, options: &[&str], index: isize) {
    if options.is_empty() {
        return;
    }
    set_option(
        value,
        options,
        index.clamp(0, options.len() as isize - 1) as usize,
    );
}

fn step_option_wrapped(value: &mut String, options: &[&str], direction: isize) {
    if options.is_empty() {
        return;
    }
    let index = option_index(value, options);
    let next = if direction < 0 {
        if index == 0 {
            options.len() - 1
        } else {
            index - 1
        }
    } else {
        (index + 1) % options.len()
    };
    set_option(value, options, next);
}

fn emit_string(context: &mut TweeqContext, id: ParamId, value: &str) {
    context.immediate_edit(
        id,
        ParamKind::String,
        EditOperation::SetString(value.to_owned()),
    );
}

fn contrast_text(background: Color32) -> Color32 {
    let luminance = 0.299 * f32::from(background.r())
        + 0.587 * f32::from(background.g())
        + 0.114 * f32::from(background.b());
    if luminance > 150.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::{option_at_x, set_option_clamped, step_option_wrapped};

    #[test]
    fn radio_hit_test_covers_segments() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 24.0));
        assert_eq!(option_at_x(rect, 10.0, 3), 0);
        assert_eq!(option_at_x(rect, 150.0, 3), 1);
        assert_eq!(option_at_x(rect, 299.0, 3), 2);
    }

    #[test]
    fn drum_clamps_at_both_ends() {
        let mut value = "a".to_owned();
        set_option_clamped(&mut value, &["a", "b", "c"], -1);
        assert_eq!(value, "a");
        set_option_clamped(&mut value, &["a", "b", "c"], 99);
        assert_eq!(value, "c");
    }

    #[test]
    fn keyboard_option_stepping_wraps() {
        let mut value = "a".to_owned();
        step_option_wrapped(&mut value, &["a", "b", "c"], -1);
        assert_eq!(value, "c");
        step_option_wrapped(&mut value, &["a", "b", "c"], 1);
        assert_eq!(value, "a");
    }
}
