use egui::{Response, Ui};
use tweeq_core::{EditOperation, ParamId, ParamKind};

use crate::TweeqContext;

/// RGBA color parameter using egui's renderer-independent color picker.
pub struct ColorInput<'a> {
    id: ParamId,
    value: &'a mut [f32; 4],
}

impl<'a> ColorInput<'a> {
    pub fn new(id: ParamId, value: &'a mut [f32; 4]) -> Self {
        Self { id, value }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_color(self.id, *self.value);
        let response = ui.color_edit_button_rgba_unmultiplied(self.value);
        if response.changed() {
            context.immediate_edit(
                self.id,
                ParamKind::Color,
                EditOperation::SetColor(*self.value),
            );
        }
        response
    }
}
