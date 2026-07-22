use egui::{Response, Ui};
use tweeq_core::{EditOperation, ParamId, ParamKind};

use crate::TweeqContext;

/// String-backed radio group.
pub struct Radio<'a> {
    id: ParamId,
    value: &'a mut String,
    options: &'a [&'a str],
}

impl<'a> Radio<'a> {
    pub fn new(id: ParamId, value: &'a mut String, options: &'a [&'a str]) -> Self {
        Self { id, value, options }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let before = self.value.clone();
        let inner = ui.horizontal(|ui| {
            for option in self.options {
                ui.selectable_value(self.value, (*option).to_owned(), *option);
            }
        });
        if before != *self.value {
            context.immediate_edit(
                self.id,
                ParamKind::String,
                EditOperation::SetString(self.value.clone()),
            );
        }
        inner.response
    }
}

/// String-backed dropdown.
pub struct Dropdown<'a> {
    id: ParamId,
    value: &'a mut String,
    options: &'a [&'a str],
    width: f32,
}

impl<'a> Dropdown<'a> {
    pub fn new(id: ParamId, value: &'a mut String, options: &'a [&'a str]) -> Self {
        Self {
            id,
            value,
            options,
            width: 180.0,
        }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let before = self.value.clone();
        let inner = egui::ComboBox::from_id_salt(("tweeq-dropdown", self.id.as_u64()))
            .width(self.width)
            .selected_text(self.value.as_str())
            .show_ui(ui, |ui| {
                for option in self.options {
                    ui.selectable_value(self.value, (*option).to_owned(), *option);
                }
            });
        if before != *self.value {
            context.immediate_edit(
                self.id,
                ParamKind::String,
                EditOperation::SetString(self.value.clone()),
            );
        }
        inner.response
    }
}

/// Horizontal option drum with buttons, wheel, and arrow-key stepping.
pub struct Drum<'a> {
    id: ParamId,
    value: &'a mut String,
    options: &'a [&'a str],
}

impl<'a> Drum<'a> {
    pub fn new(id: ParamId, value: &'a mut String, options: &'a [&'a str]) -> Self {
        Self { id, value, options }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_string(self.id, self.value);
        let before = self.value.clone();
        let inner = ui.horizontal(|ui| {
            let previous = ui.small_button("◀");
            let label = ui.add_sized(
                [118.0, context.theme().input_height],
                egui::Button::new(self.value.as_str()),
            );
            let next = ui.small_button("▶");
            let wheel = if label.hovered() {
                ui.input(|input| input.smooth_scroll_delta.y)
            } else {
                0.0
            };
            let backwards = previous.clicked()
                || (label.has_focus() && ui.input(|input| input.key_pressed(egui::Key::ArrowLeft)))
                || wheel > 0.0;
            let forwards = next.clicked()
                || label.clicked()
                || (label.has_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::ArrowRight)))
                || wheel < 0.0;
            if backwards {
                step_option(self.value, self.options, -1);
            } else if forwards {
                step_option(self.value, self.options, 1);
            }
        });
        if before != *self.value {
            context.immediate_edit(
                self.id,
                ParamKind::String,
                EditOperation::SetString(self.value.clone()),
            );
        }
        inner.response
    }
}

fn step_option(value: &mut String, options: &[&str], direction: isize) {
    if options.is_empty() {
        return;
    }
    let index = options
        .iter()
        .position(|option| *option == value)
        .unwrap_or(0);
    let next = if direction < 0 {
        if index == 0 {
            options.len() - 1
        } else {
            index - 1
        }
    } else {
        (index + 1) % options.len()
    };
    options[next].clone_into(value);
}

#[cfg(test)]
mod tests {
    use super::step_option;

    #[test]
    fn drum_wraps_in_both_directions() {
        let mut value = "a".to_owned();
        step_option(&mut value, &["a", "b", "c"], -1);
        assert_eq!(value, "c");
        step_option(&mut value, &["a", "b", "c"], 1);
        assert_eq!(value, "a");
    }
}
