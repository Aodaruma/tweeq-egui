use egui::{Align2, FontId, Id, InnerResponse, Response, Sense, Stroke, Ui, Vec2};
use tweeq_core::{EditOperation, ParamId, ParamKind, ViewTransform, nice_tick_step};

use crate::TweeqContext;

/// Compact tab row with a host-owned selected index.
pub struct Tabs<'a> {
    id: Id,
    selected: &'a mut usize,
    labels: &'a [&'a str],
}

impl<'a> Tabs<'a> {
    pub fn new(
        id_source: impl std::hash::Hash + std::fmt::Debug,
        selected: &'a mut usize,
        labels: &'a [&'a str],
    ) -> Self {
        Self {
            id: Id::new(id_source),
            selected,
            labels,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.push_id(self.id, |ui| {
            ui.horizontal(|ui| {
                let mut combined = None;
                for (index, label) in self.labels.iter().enumerate() {
                    let response = ui.selectable_label(*self.selected == index, *label);
                    if response.clicked() {
                        *self.selected = index;
                    }
                    combined = Some(combined.map_or(response.clone(), |previous: Response| {
                        previous.union(response)
                    }));
                }
                if ui.input(|input| input.key_pressed(egui::Key::ArrowRight)) {
                    *self.selected = (*self.selected + 1).min(self.labels.len().saturating_sub(1));
                }
                if ui.input(|input| input.key_pressed(egui::Key::ArrowLeft)) {
                    *self.selected = self.selected.saturating_sub(1);
                }
                combined.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
            })
            .inner
        })
        .inner
    }
}

/// Theme-neutral collapsing pane adapter.
pub struct CollapsingPane<'a> {
    title: &'a str,
    default_open: bool,
}

impl<'a> CollapsingPane<'a> {
    #[must_use]
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            default_open: false,
        }
    }

    #[must_use]
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> egui::CollapsingResponse<R> {
        egui::CollapsingHeader::new(self.title)
            .default_open(self.default_open)
            .show(ui, body)
    }
}

/// Movable floating pane backed by `egui::Window`.
pub struct FloatingPane<'a> {
    id: Id,
    title: &'a str,
    open: &'a mut bool,
}

impl<'a> FloatingPane<'a> {
    pub fn new(
        id_source: impl std::hash::Hash + std::fmt::Debug,
        title: &'a str,
        open: &'a mut bool,
    ) -> Self {
        Self {
            id: Id::new(id_source),
            title,
            open,
        }
    }

    pub fn show<R>(
        self,
        context: &egui::Context,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<Option<R>>> {
        egui::Window::new(self.title)
            .id(self.id)
            .open(self.open)
            .resizable(true)
            .collapsible(true)
            .show(context, body)
    }
}

/// Horizontal ruler whose model remains renderer-independent.
pub struct Ruler {
    start: f64,
    units_per_pixel: f64,
    cursor: Option<f64>,
    width: f32,
}

impl Ruler {
    #[must_use]
    pub fn new(start: f64, units_per_pixel: f64) -> Self {
        Self {
            start,
            units_per_pixel: units_per_pixel.max(f64::MIN_POSITIVE),
            cursor: None,
            width: 400.0,
        }
    }

    #[must_use]
    pub fn cursor(mut self, cursor: f64) -> Self {
        self.cursor = Some(cursor);
        self
    }

    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(80.0);
        self
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(self.width.min(ui.available_width()), 34.0),
            Sense::hover(),
        );
        let visuals = ui.visuals();
        ui.painter()
            .rect_filled(rect, 3.0, visuals.extreme_bg_color);
        let step = nice_tick_step(self.units_per_pixel, 54.0);
        let end = self.start + f64::from(rect.width()) * self.units_per_pixel;
        let mut value = (self.start / step).floor() * step;
        while value <= end + step {
            let x = rect.left() + ((value - self.start) / self.units_per_pixel) as f32;
            if x >= rect.left() - 1.0 && x <= rect.right() + 1.0 {
                ui.painter().line_segment(
                    [
                        egui::pos2(x, rect.bottom()),
                        egui::pos2(x, rect.top() + 13.0),
                    ],
                    Stroke::new(1.0, visuals.weak_text_color()),
                );
                ui.painter().text(
                    egui::pos2(x + 3.0, rect.top() + 3.0),
                    Align2::LEFT_TOP,
                    format_tick(value),
                    FontId::monospace(9.0),
                    visuals.weak_text_color(),
                );
            }
            value += step;
        }
        if let Some(cursor) = self.cursor {
            let x = rect.left() + ((cursor - self.start) / self.units_per_pixel) as f32;
            if rect.x_range().contains(x) {
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(2.0, visuals.selection.stroke.color),
                );
            }
        }
        response
    }
}

impl egui::Widget for Ruler {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

fn format_tick(value: f64) -> String {
    if value.abs() >= 100.0 || value.fract().abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

/// Timeline scrubber. The host owns both the range and current value.
pub struct Timeline<'a> {
    id: ParamId,
    current: &'a mut f64,
    range: std::ops::RangeInclusive<f64>,
}

impl<'a> Timeline<'a> {
    pub fn new(id: ParamId, current: &'a mut f64, range: std::ops::RangeInclusive<f64>) -> Self {
        Self { id, current, range }
    }

    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        context.register_number(self.id, *self.current);
        let theme = context.theme().clone();
        let width = ui.available_width().clamp(220.0, 560.0);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, 58.0), Sense::click_and_drag());
        ui.painter()
            .rect_filled(rect, theme.input_radius, theme.input);
        let ruler = Ruler::new(
            *self.range.start(),
            range_per_pixel(&self.range, rect.width()),
        )
        .cursor(*self.current)
        .width(rect.width());
        let _ = ui.put(rect, ruler);

        let mut state = context.take_scalar_drag_state(self.id);
        if response.drag_started() {
            state.captured = *self.current;
            state.session = Some(context.start_edit(self.id, ParamKind::Number));
        }
        if (response.dragged() || response.clicked())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let fraction = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let candidate = egui::lerp(self.range.clone(), f64::from(fraction));
            *self.current = candidate;
            if let Some(session) = state.session {
                context.update_edit(session, EditOperation::SetNumber(candidate));
            } else {
                context.immediate_edit(
                    self.id,
                    ParamKind::Number,
                    EditOperation::SetNumber(candidate),
                );
            }
        }
        if response.drag_stopped()
            && let Some(session) = state.session.take()
        {
            context.finish_edit(session, true);
        }
        context.put_scalar_drag_state(self.id, state);
        response
    }
}

fn range_per_pixel(range: &std::ops::RangeInclusive<f64>, width: f32) -> f64 {
    (*range.end() - *range.start()) / f64::from(width.max(1.0))
}

/// Pan/zoom grid proving the `ViewTransform` adapter boundary.
pub struct Viewport2D<'a> {
    offset: &'a mut [f64; 2],
    scale: &'a mut f64,
    size: Vec2,
}

impl<'a> Viewport2D<'a> {
    pub fn new(offset: &'a mut [f64; 2], scale: &'a mut f64) -> Self {
        Self {
            offset,
            scale,
            size: Vec2::new(420.0, 180.0),
        }
    }

    #[must_use]
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn show(self, ui: &mut Ui) -> Response {
        let size = Vec2::new(self.size.x.min(ui.available_width()), self.size.y);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let visuals = ui.visuals();
        ui.painter()
            .rect_filled(rect, 4.0, visuals.extreme_bg_color);

        if response.dragged() {
            let delta = ui.input(|input| input.pointer.delta());
            self.offset[0] -= f64::from(delta.x) / *self.scale;
            self.offset[1] -= f64::from(delta.y) / *self.scale;
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                *self.scale = (*self.scale * f64::from((scroll * 0.002).exp())).clamp(0.05, 50.0);
            }
        }

        let transform = ViewTransform {
            offset: *self.offset,
            scale: *self.scale,
        };
        let spacing_world = nice_tick_step(1.0 / *self.scale, 42.0);
        let world_left = self.offset[0];
        let world_top = self.offset[1];
        let world_right = world_left + f64::from(rect.width()) / *self.scale;
        let world_bottom = world_top + f64::from(rect.height()) / *self.scale;
        let mut x = (world_left / spacing_world).floor() * spacing_world;
        while x <= world_right + spacing_world {
            let screen = transform.world_to_screen([x, 0.0]);
            let px = rect.left() + screen[0] as f32;
            ui.painter().line_segment(
                [egui::pos2(px, rect.top()), egui::pos2(px, rect.bottom())],
                Stroke::new(1.0, visuals.weak_text_color().gamma_multiply(0.35)),
            );
            x += spacing_world;
        }
        let mut y = (world_top / spacing_world).floor() * spacing_world;
        while y <= world_bottom + spacing_world {
            let screen = transform.world_to_screen([0.0, y]);
            let py = rect.top() + screen[1] as f32;
            ui.painter().line_segment(
                [egui::pos2(rect.left(), py), egui::pos2(rect.right(), py)],
                Stroke::new(1.0, visuals.weak_text_color().gamma_multiply(0.35)),
            );
            y += spacing_world;
        }
        ui.painter().text(
            rect.left_top() + Vec2::splat(8.0),
            Align2::LEFT_TOP,
            format!(
                "offset {:.1}, {:.1} · zoom {:.2}×",
                self.offset[0], self.offset[1], *self.scale
            ),
            FontId::monospace(10.0),
            visuals.text_color(),
        );
        response.on_hover_text("Drag to pan · wheel/trackpad to zoom")
    }
}

/// Small host-owned command palette adapter.
pub struct CommandPalette<'a> {
    open: &'a mut bool,
    query: &'a mut String,
    commands: &'a [&'a str],
}

impl<'a> CommandPalette<'a> {
    pub fn new(open: &'a mut bool, query: &'a mut String, commands: &'a [&'a str]) -> Self {
        Self {
            open,
            query,
            commands,
        }
    }

    /// Returns the index of a selected host command.
    #[must_use]
    pub fn show(self, context: &egui::Context) -> Option<usize> {
        if !*self.open {
            return None;
        }
        let mut open = true;
        let mut selected = None;
        let mut close_after_selection = false;
        egui::Window::new("Command palette")
            .id(Id::new("tweeq-command-palette"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 80.0))
            .show(context, |ui| {
                ui.add(
                    egui::TextEdit::singleline(self.query)
                        .hint_text("Filter commands")
                        .desired_width(320.0),
                );
                let needle = self.query.to_lowercase();
                for (index, command) in self.commands.iter().enumerate() {
                    if (needle.is_empty() || command.to_lowercase().contains(&needle))
                        && ui.button(*command).clicked()
                    {
                        selected = Some(index);
                        close_after_selection = true;
                    }
                }
            });
        if close_after_selection {
            open = false;
        }
        *self.open = open;
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::format_tick;

    #[test]
    fn formats_ruler_labels_compactly() {
        assert_eq!(format_tick(120.0), "120");
        assert_eq!(format_tick(1.25), "1.25");
    }
}
