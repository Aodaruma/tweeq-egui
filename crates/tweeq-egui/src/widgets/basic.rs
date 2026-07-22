use egui::{Response, RichText, Ui};
use tweeq_core::{EditOperation, ParamId, ParamKind};

use crate::TweeqContext;

/// Tweeq-styled momentary button.
pub struct Button<'a> {
    label: &'a str,
    enabled: bool,
}

impl<'a> Button<'a> {
    #[must_use]
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add_enabled(self.enabled, egui::Button::new(self.label))
    }
}

/// Boolean button that remains highlighted while active.
pub struct ToggleButton<'a> {
    id: ParamId,
    value: &'a mut bool,
    label: &'a str,
}

impl<'a> ToggleButton<'a> {
    pub fn new(id: ParamId, value: &'a mut bool, label: &'a str) -> Self {
        Self { id, value, label }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_boolean(self.id, *self.value);
        let response = ui.selectable_label(*self.value, self.label);
        if response.clicked() {
            *self.value = !*self.value;
            context.immediate_edit(
                self.id,
                ParamKind::Boolean,
                EditOperation::SetBoolean(*self.value),
            );
        }
        response
    }
}

/// Checkbox with Tweeq boolean shortcuts while focused.
pub struct Checkbox<'a> {
    id: ParamId,
    value: &'a mut bool,
    label: &'a str,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: ParamId, value: &'a mut bool, label: &'a str) -> Self {
        Self { id, value, label }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_boolean(self.id, *self.value);
        let before = *self.value;
        let response = ui.checkbox(self.value, self.label);
        if response.has_focus() {
            if ui.input(|input| {
                [egui::Key::T, egui::Key::Y, egui::Key::Num1, egui::Key::P]
                    .into_iter()
                    .any(|key| input.key_pressed(key))
            }) {
                *self.value = true;
            }
            if ui.input(|input| {
                [egui::Key::F, egui::Key::N, egui::Key::Num0, egui::Key::M]
                    .into_iter()
                    .any(|key| input.key_pressed(key))
            }) {
                *self.value = false;
            }
        }
        if before != *self.value {
            context.immediate_edit(
                self.id,
                ParamKind::Boolean,
                EditOperation::SetBoolean(*self.value),
            );
        }
        response
    }
}

/// Compact switch painted using egui's boolean control behavior.
pub struct Switch<'a> {
    inner: Checkbox<'a>,
}

impl<'a> Switch<'a> {
    pub fn new(id: ParamId, value: &'a mut bool, label: &'a str) -> Self {
        Self {
            inner: Checkbox::new(id, value, label),
        }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        self.inner.show(ui, context)
    }
}

/// Single-line string parameter.
pub struct StringInput<'a> {
    id: ParamId,
    value: &'a mut String,
    hint: &'a str,
    width: f32,
}

impl<'a> StringInput<'a> {
    pub fn new(id: ParamId, value: &'a mut String) -> Self {
        Self {
            id,
            value,
            hint: "",
            width: 180.0,
        }
    }

    #[must_use]
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let response = ui.add_sized(
            [self.width, context.theme().input_height],
            egui::TextEdit::singleline(self.value).hint_text(self.hint),
        );
        if response.changed() {
            context.immediate_edit(
                self.id,
                ParamKind::String,
                EditOperation::SetString(self.value.clone()),
            );
        }
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
