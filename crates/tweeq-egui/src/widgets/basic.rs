use egui::{
    Color32, CornerRadius, LayerId, Order, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use tweeq_core::{EditOperation, EditSessionId, ParamId, ParamKind};

use crate::{TweeqContext, TweeqTheme};

const INVALID: Color32 = Color32::from_rgb(232, 78, 88);

/// Tweeq-styled momentary button.
pub struct Button<'a> {
    label: &'a str,
    enabled: bool,
    invalid: bool,
    subtle: bool,
}

impl<'a> Button<'a> {
    #[must_use]
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
            invalid: false,
            subtle: false,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Marks the button as invalid using the same error treatment as inputs.
    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Uses Tweeq's neutral button treatment instead of an accent fill.
    #[must_use]
    pub const fn subtle(mut self, subtle: bool) -> Self {
        self.subtle = subtle;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let visuals = ui.visuals();
        let fill = if self.subtle {
            visuals.widgets.inactive.bg_fill
        } else {
            visuals.selection.bg_fill
        };
        let subtle_hover = visuals.widgets.active.bg_fill;
        let text = if self.invalid {
            INVALID
        } else if self.subtle {
            visuals.text_color()
        } else {
            contrast_text(fill)
        };
        let button = egui::Button::new(RichText::new(self.label).color(text))
            .fill(fill)
            .stroke(if self.invalid {
                Stroke::new(1.0, INVALID)
            } else {
                Stroke::NONE
            })
            .corner_radius(visuals.widgets.inactive.corner_radius)
            .min_size(Vec2::new(
                ui.spacing().interact_size.y,
                ui.spacing().interact_size.y,
            ));
        ui.scope(|ui| {
            let widgets = &mut ui.visuals_mut().widgets;
            widgets.inactive.expansion = 0.0;
            widgets.hovered.expansion = 0.0;
            widgets.active.expansion = 0.0;
            widgets.open.expansion = 0.0;
            widgets.hovered.bg_fill = if self.subtle {
                subtle_hover
            } else {
                fill.gamma_multiply(0.82)
            };
            ui.add_enabled(self.enabled, button)
        })
        .inner
    }
}

/// Boolean button that remains highlighted while active.
pub struct ToggleButton<'a> {
    id: ParamId,
    value: &'a mut bool,
    label: &'a str,
    enabled: bool,
    invalid: bool,
}

impl<'a> ToggleButton<'a> {
    pub fn new(id: ParamId, value: &'a mut bool, label: &'a str) -> Self {
        Self {
            id,
            value,
            label,
            enabled: true,
            invalid: false,
        }
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
        context.register_boolean(self.id, *self.value);
        let theme = context.theme();
        let fill = if *self.value {
            theme.accent
        } else {
            theme.input
        };
        let text = if self.invalid {
            INVALID
        } else if *self.value {
            contrast_text(fill)
        } else {
            theme.text
        };
        let button = egui::Button::new(RichText::new(self.label).color(text))
            .fill(fill)
            .stroke(if self.invalid {
                Stroke::new(1.0, INVALID)
            } else {
                Stroke::NONE
            })
            .corner_radius(theme.input_radius)
            .min_size(Vec2::new(theme.input_height, theme.input_height));
        let response = ui
            .scope(|ui| {
                ui.visuals_mut().widgets.hovered.bg_fill = if *self.value {
                    theme.accent_hover
                } else {
                    theme.input_hover
                };
                ui.add_enabled(self.enabled, button)
            })
            .inner;
        if response.clicked() {
            *self.value = !*self.value;
            emit_boolean(context, self.id, *self.value);
        }
        response
    }
}

/// Checkbox with Tweeq boolean shortcuts and horizontal swipe selection.
pub struct Checkbox<'a> {
    id: ParamId,
    value: &'a mut bool,
    label: &'a str,
    enabled: bool,
    invalid: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: ParamId, value: &'a mut bool, label: &'a str) -> Self {
        Self {
            id,
            value,
            label,
            enabled: true,
            invalid: false,
        }
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
        context.register_boolean(self.id, *self.value);
        let theme = context.theme().clone();
        let font = egui::TextStyle::Body.resolve(ui.style());
        let label_size = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), font.clone(), theme.text)
            .size();
        let width = theme.input_height
            + if self.label.is_empty() {
                0.0
            } else {
                14.0 + label_size.x
            };
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(width, theme.input_height),
            if self.enabled {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            },
        );
        response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        let interaction = boolean_interaction(ui, &response, self.id, self.value, self.enabled);
        if interaction.changed {
            response.mark_changed();
            emit_boolean(context, self.id, *self.value);
        }

        let box_rect = egui::Rect::from_min_size(rect.min, Vec2::splat(theme.input_height));
        paint_checkbox(
            ui,
            box_rect,
            &response,
            *self.value,
            self.enabled,
            self.invalid,
            &theme,
        );
        if interaction.active {
            paint_boolean_overlay(ui, self.id, box_rect, interaction.preview, &theme);
        }
        if !self.label.is_empty() {
            let color = if self.enabled {
                theme.text
            } else {
                theme.text_muted
            };
            ui.painter().text(
                egui::pos2(box_rect.max.x + 14.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.label,
                font,
                color,
            );
        }
        response
    }
}

/// Tweeq pill switch. It shares Checkbox's click, swipe, and boolean keys.
pub struct Switch<'a> {
    id: ParamId,
    value: &'a mut bool,
    label: &'a str,
    enabled: bool,
    invalid: bool,
}

impl<'a> Switch<'a> {
    pub fn new(id: ParamId, value: &'a mut bool, label: &'a str) -> Self {
        Self {
            id,
            value,
            label,
            enabled: true,
            invalid: false,
        }
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
        context.register_boolean(self.id, *self.value);
        let theme = context.theme().clone();
        let font = egui::TextStyle::Body.resolve(ui.style());
        let label_size = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), font.clone(), theme.text)
            .size();
        let track_width = theme.input_height * 2.0;
        let width = track_width
            + if self.label.is_empty() {
                0.0
            } else {
                14.0 + label_size.x
            };
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(width, theme.input_height),
            if self.enabled {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            },
        );
        response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        let interaction = boolean_interaction(ui, &response, self.id, self.value, self.enabled);
        if interaction.changed {
            response.mark_changed();
            emit_boolean(context, self.id, *self.value);
        }

        let track = egui::Rect::from_min_size(rect.min, Vec2::new(track_width, theme.input_height));
        paint_switch(
            ui,
            track,
            &response,
            *self.value,
            self.enabled,
            self.invalid,
            interaction.active,
            &theme,
        );
        if !self.label.is_empty() {
            ui.painter().text(
                egui::pos2(track.max.x + 14.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.label,
                font,
                if self.enabled {
                    theme.text
                } else {
                    theme.text_muted
                },
            );
        }
        response
    }
}

/// Single-line string parameter.
pub struct StringInput<'a> {
    id: ParamId,
    value: &'a mut String,
    hint: &'a str,
    width: f32,
    enabled: bool,
    invalid: bool,
}

#[derive(Clone, Default)]
struct StringEditMemory {
    captured: String,
    session: Option<EditSessionId>,
}

impl<'a> StringInput<'a> {
    pub fn new(id: ParamId, value: &'a mut String) -> Self {
        Self {
            id,
            value,
            hint: "",
            width: 180.0,
            enabled: true,
            invalid: false,
        }
    }

    #[must_use]
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
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
        let state_id = ui.make_persistent_id(("tweeq-string", self.id.as_u64()));
        let mut state = ui
            .data(|data| data.get_temp::<StringEditMemory>(state_id))
            .unwrap_or_default();
        let mut response = ui
            .scope(|ui| {
                let visuals = ui.visuals_mut();
                visuals.widgets.inactive.bg_fill = if self.enabled {
                    theme.input
                } else {
                    Color32::TRANSPARENT
                };
                visuals.widgets.hovered.bg_fill = theme.input_hover;
                visuals.widgets.inactive.bg_stroke = if self.invalid {
                    Stroke::new(1.0, INVALID)
                } else if self.enabled {
                    Stroke::NONE
                } else {
                    Stroke::new(1.0, theme.border)
                };
                visuals.override_text_color = self.invalid.then_some(INVALID);
                ui.add_enabled(
                    self.enabled,
                    egui::TextEdit::singleline(self.value)
                        .hint_text(self.hint)
                        .desired_width(self.width),
                )
            })
            .inner;
        if response.gained_focus() {
            state.captured.clone_from(self.value);
            let modifiers = ui.input(|input| input.modifiers);
            context.activate_selection(
                self.id,
                ParamKind::String,
                modifiers.shift,
                modifiers.command,
            );
            state.session = Some(context.start_edit(self.id, ParamKind::String));
        }
        if self.invalid {
            ui.painter().rect_stroke(
                response.rect,
                CornerRadius::same(theme.input_radius),
                Stroke::new(1.0, INVALID),
                StrokeKind::Inside,
            );
        }
        if response.changed() {
            response.mark_changed();
            if let Some(session) = state.session {
                context.update_edit(session, EditOperation::SetString(self.value.clone()));
            } else {
                context.immediate_edit(
                    self.id,
                    ParamKind::String,
                    EditOperation::SetString(self.value.clone()),
                );
            }
        }
        if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.value.clone_from(&state.captured);
            if let Some(session) = state.session.take() {
                context.finish_edit(session, false);
            }
            response.surrender_focus();
            response.mark_changed();
        } else if (response.lost_focus()
            || (response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter))))
            && let Some(session) = state.session.take()
        {
            context.finish_edit(session, true);
            response.surrender_focus();
        }
        ui.data_mut(|data| data.insert_temp(state_id, state));
        response
    }
}

/// Visual grouping primitive for related inputs.
pub struct InputGroup;

impl InputGroup {
    pub fn show<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        egui::Frame::NONE
            .inner_margin(3.0)
            .corner_radius(4.0)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .show(ui, |ui| ui.horizontal(|ui| add_contents(ui)).inner)
    }

    pub fn label(ui: &mut Ui, text: &str) {
        ui.label(RichText::new(text).weak());
    }
}

#[derive(Clone, Copy)]
struct BooleanInteraction {
    changed: bool,
    active: bool,
    preview: bool,
}

fn boolean_interaction(
    ui: &Ui,
    response: &Response,
    id: ParamId,
    value: &mut bool,
    enabled: bool,
) -> BooleanInteraction {
    if !enabled {
        return BooleanInteraction {
            changed: false,
            active: false,
            preview: *value,
        };
    }
    let before = *value;
    if response.clicked() {
        *value = !*value;
        response.request_focus();
    }

    let captured_id = response.id.with(("tweeq-bool-captured", id.as_u64()));
    if response.drag_started() {
        ui.data_mut(|data| data.insert_temp(captured_id, *value));
    }
    if response.dragged() {
        let captured = ui
            .data(|data| data.get_temp::<bool>(captured_id))
            .unwrap_or(*value);
        let dx = response.total_drag_delta().unwrap_or_default().x;
        *value = if dx.abs() <= 3.0 { !captured } else { dx > 0.0 };
    }

    if response.has_focus() {
        let set_true = ui.input(|input| {
            [egui::Key::T, egui::Key::Y, egui::Key::Num1, egui::Key::P]
                .into_iter()
                .any(|key| input.key_pressed(key))
        });
        let set_false = ui.input(|input| {
            [egui::Key::F, egui::Key::N, egui::Key::Num0, egui::Key::M]
                .into_iter()
                .any(|key| input.key_pressed(key))
        });
        if set_true {
            *value = true;
        } else if set_false {
            *value = false;
        } else if ui.input(|input| input.key_pressed(egui::Key::Space)) {
            *value = !*value;
        }
    }
    BooleanInteraction {
        changed: before != *value,
        active: response.is_pointer_button_down_on() || response.dragged(),
        preview: *value,
    }
}

fn paint_checkbox(
    ui: &Ui,
    rect: egui::Rect,
    response: &Response,
    checked: bool,
    enabled: bool,
    invalid: bool,
    theme: &TweeqTheme,
) {
    let fill = if !enabled {
        if checked {
            theme.text_muted
        } else {
            Color32::TRANSPARENT
        }
    } else if checked {
        if response.hovered() {
            theme.accent_hover
        } else {
            theme.accent
        }
    } else if response.hovered() {
        theme.input_hover
    } else {
        theme.input
    };
    let stroke = if invalid {
        Stroke::new(1.0, INVALID)
    } else if !enabled {
        Stroke::new(1.0, theme.border)
    } else if response.has_focus() {
        Stroke::new(1.0, theme.accent)
    } else {
        Stroke::NONE
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(theme.input_radius),
        fill,
        stroke,
        StrokeKind::Inside,
    );
    if checked {
        let color = if enabled {
            contrast_text(fill)
        } else {
            theme.background
        };
        paint_check_mark(ui.painter(), rect.center(), 1.8, color);
    }
}

fn paint_boolean_overlay(
    ui: &Ui,
    id: ParamId,
    rect: egui::Rect,
    preview: bool,
    theme: &TweeqTheme,
) {
    let painter = ui.ctx().layer_painter(LayerId::new(
        Order::Foreground,
        egui::Id::new(("tweeq-boolean-overlay", id.as_u64())),
    ));
    let offset = theme.input_height * 1.05;
    let radius = theme.input_height * 0.32;
    let off = rect.center() - Vec2::new(offset, 0.0);
    let on = rect.center() + Vec2::new(offset, 0.0);

    for (center, selected) in [(off, !preview), (on, preview)] {
        painter.circle_filled(center, radius + 2.0, theme.background);
        if selected {
            painter.circle_filled(center, radius, theme.accent);
        } else {
            painter.circle_stroke(center, radius, Stroke::new(1.5, theme.border));
        }
    }
    paint_check_mark(
        &painter,
        on,
        1.35,
        if preview {
            contrast_text(theme.accent)
        } else {
            theme.border
        },
    );
}

fn paint_check_mark(painter: &egui::Painter, center: egui::Pos2, width: f32, color: Color32) {
    let stroke = Stroke::new(width, color);
    painter.line_segment(
        [center + Vec2::new(-4.2, 0.0), center + Vec2::new(-1.2, 3.0)],
        stroke,
    );
    painter.line_segment(
        [center + Vec2::new(-1.2, 3.0), center + Vec2::new(4.8, -3.5)],
        stroke,
    );
}

#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
fn paint_switch(
    ui: &Ui,
    rect: egui::Rect,
    response: &Response,
    checked: bool,
    enabled: bool,
    invalid: bool,
    tweaking: bool,
    theme: &TweeqTheme,
) {
    let fill = if !enabled {
        Color32::TRANSPARENT
    } else if checked {
        if response.hovered() {
            theme.accent_hover
        } else {
            theme.accent
        }
    } else if response.hovered() {
        theme.input_hover
    } else {
        theme.input
    };
    let outline = if invalid {
        Stroke::new(1.0, INVALID)
    } else if !enabled {
        Stroke::new(1.0, theme.border)
    } else if response.has_focus() {
        Stroke::new(1.0, theme.accent)
    } else {
        Stroke::NONE
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(255),
        fill,
        outline,
        StrokeKind::Outside,
    );
    let handle_height = rect.height() - 8.0;
    let handle_width = handle_height + if tweaking { 4.0 } else { 0.0 };
    let left = if checked {
        rect.right() - 4.0 - handle_width
    } else {
        rect.left() + 4.0
    };
    let handle = egui::Rect::from_min_size(
        egui::pos2(left, rect.center().y - handle_height * 0.5),
        Vec2::new(handle_width, handle_height),
    );
    ui.painter().rect_filled(
        handle,
        CornerRadius::same(255),
        if checked && enabled {
            theme.background
        } else {
            theme.text_muted
        },
    );
}

fn emit_boolean(context: &mut TweeqContext, id: ParamId, value: bool) {
    context.immediate_edit(id, ParamKind::Boolean, EditOperation::SetBoolean(value));
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
