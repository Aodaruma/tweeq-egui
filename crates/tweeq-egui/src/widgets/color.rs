#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Key, LayerId, Mesh, Order, PopupCloseBehavior, Pos2,
    Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
};
use tweeq_core::{EditOperation, EditSessionId, ParamId, ParamKind};

use crate::{TweeqContext, TweeqTheme};

const PICKER_WIDTH: f32 = 224.0;
const PAD_HEIGHT: f32 = 150.0;
const SLIDER_HEIGHT: f32 = 14.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ColorSpace {
    #[default]
    Hsv,
    Rgb,
    Hex,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ColorDragMode {
    #[default]
    SaturationValue,
    Hue,
    Saturation,
    Value,
    Alpha,
    Red,
    Green,
    Blue,
}

impl ColorDragMode {
    const fn label(self) -> &'static str {
        match self {
            Self::SaturationValue => "S / V",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Value => "Value",
            Self::Alpha => "Alpha",
            Self::Red => "Red",
            Self::Green => "Green",
            Self::Blue => "Blue",
        }
    }
}

#[derive(Clone)]
struct ColorState {
    captured: [f32; 4],
    session: Option<EditSessionId>,
    drag_mode: ColorDragMode,
    drag_origin: Pos2,
    color_space: ColorSpace,
    hex_buffer: String,
}

impl ColorState {
    fn new(value: [f32; 4]) -> Self {
        Self {
            captured: value,
            session: None,
            drag_mode: ColorDragMode::SaturationValue,
            drag_origin: Pos2::ZERO,
            color_space: ColorSpace::Hsv,
            hex_buffer: rgba_to_hex(value),
        }
    }
}

/// Tweeq color parameter with relative channel scrubbing and a dedicated picker.
///
/// Drag the compact field to edit saturation/value. Hold Shift (or H), S, V,
/// Alt (or A), R, G, or B to edit one channel. Click to open the picker.
pub struct ColorInput<'a> {
    id: ParamId,
    value: &'a mut [f32; 4],
    width: f32,
    alpha: bool,
}

impl<'a> ColorInput<'a> {
    pub fn new(id: ParamId, value: &'a mut [f32; 4]) -> Self {
        Self {
            id,
            value,
            width: 240.0,
            alpha: true,
        }
    }

    /// Sets the width of the compact color field.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(72.0);
        self
    }

    /// Shows or hides alpha editing. Hidden alpha is forced to one.
    #[must_use]
    pub fn alpha(mut self, alpha: bool) -> Self {
        self.alpha = alpha;
        self
    }

    #[allow(clippy::too_many_lines)]
    pub fn show(self, ui: &mut Ui, context: &mut TweeqContext) -> Response {
        if !self.alpha {
            self.value[3] = 1.0;
        }
        context.register_color(self.id, *self.value);
        let theme = context.theme().clone();
        let state_id = ui.make_persistent_id(("tweeq-color", self.id.as_u64()));
        let mut state = ui
            .data(|data| data.get_temp::<ColorState>(state_id))
            .unwrap_or_else(|| ColorState::new(*self.value));

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(self.width, theme.input_height),
            Sense::click_and_drag(),
        );
        response = response.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
        paint_color_field(ui, rect, &response, *self.value, &theme);

        if response.drag_started() {
            let modifiers = ui.input(|input| input.modifiers);
            context.activate_selection(
                self.id,
                ParamKind::Color,
                modifiers.shift,
                modifiers.command,
            );
            state.captured = *self.value;
            state.drag_origin = response.interact_pointer_pos().unwrap_or(rect.center());
            state.session = Some(context.start_edit(self.id, ParamKind::Color));
        }

        if response.dragged() {
            state.drag_mode = color_drag_mode(ui, self.alpha);
            let total = response.total_drag_delta().unwrap_or_default();
            *self.value = state.captured;
            apply_drag(self.value, state.drag_mode, total, self.alpha);
            state.hex_buffer = rgba_to_hex(*self.value);
            if let Some(session) = state.session {
                context.update_edit(session, EditOperation::SetColor(*self.value));
            }
            response.mark_changed();
            paint_drag_overlay(
                ui,
                state_id,
                state.drag_origin,
                *self.value,
                state.drag_mode,
                &theme,
            );
            ui.ctx().request_repaint();
        }

        if response.drag_stopped()
            && let Some(session) = state.session.take()
        {
            context.finish_edit(session, true);
        }

        let popup_before = *self.value;
        let popup = egui::Popup::from_toggle_button_response(&response)
            .id(ui.make_persistent_id(("tweeq-color-popup", self.id.as_u64())))
            .gap(4.0)
            .width(PICKER_WIDTH + 20.0)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_width(PICKER_WIDTH);
                show_picker(ui, self.value, &mut state, self.alpha, &theme)
            });
        if popup.is_some() && colors_differ(*self.value, popup_before) {
            response.mark_changed();
            state.hex_buffer = rgba_to_hex(*self.value);
            context.immediate_edit(
                self.id,
                ParamKind::Color,
                EditOperation::SetColor(*self.value),
            );
        }

        ui.data_mut(|data| data.insert_temp(state_id, state));
        response
    }
}

fn paint_color_field(
    ui: &Ui,
    rect: Rect,
    response: &Response,
    value: [f32; 4],
    theme: &TweeqTheme,
) {
    let background = if response.hovered() {
        theme.input_hover
    } else {
        theme.input
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(theme.input_radius),
        background,
        Stroke::NONE,
        StrokeKind::Inside,
    );
    let swatch = Rect::from_min_max(rect.min, Pos2::new(rect.left() + 54.0, rect.bottom()));
    paint_checkerboard(ui, swatch.shrink(1.0), 6.0);
    ui.painter().rect_filled(
        swatch.shrink(1.0),
        CornerRadius::same(theme.input_radius.saturating_sub(1)),
        color32(value),
    );
    let label = rgba_to_hex(value);
    ui.painter().text(
        Pos2::new(swatch.right() + 9.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::monospace(11.0),
        theme.text,
    );
}

fn show_picker(
    ui: &mut Ui,
    value: &mut [f32; 4],
    state: &mut ColorState,
    alpha: bool,
    theme: &TweeqTheme,
) -> bool {
    let before = *value;
    let mut hsva = rgba_to_hsva(*value);
    show_sv_pad(ui, &mut hsva, theme);
    ui.add_space(4.0);
    show_hue_slider(ui, &mut hsva, theme);
    if alpha {
        ui.add_space(3.0);
        show_alpha_slider(ui, &mut hsva, theme);
    } else {
        hsva[3] = 1.0;
    }
    *value = hsva_to_rgba(hsva);

    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.color_space, ColorSpace::Hsv, "HSV");
        ui.selectable_value(&mut state.color_space, ColorSpace::Rgb, "RGB");
        ui.selectable_value(&mut state.color_space, ColorSpace::Hex, "HEX");
    });

    match state.color_space {
        ColorSpace::Hsv => show_hsv_channels(ui, value, alpha),
        ColorSpace::Rgb => show_rgb_channels(ui, value, alpha),
        ColorSpace::Hex => show_hex_channel(ui, value, state),
    }

    ui.add_space(4.0);
    ui.weak("Swatches");
    let swatches = [
        [0.95, 0.22, 0.25, 1.0],
        [1.0, 0.52, 0.12, 1.0],
        [1.0, 0.82, 0.12, 1.0],
        [0.24, 0.78, 0.38, 1.0],
        [0.12, 0.78, 0.82, 1.0],
        [0.18, 0.42, 1.0, 1.0],
        [0.58, 0.32, 0.92, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        [0.48, 0.48, 0.5, 1.0],
        [0.05, 0.05, 0.06, 1.0],
    ];
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for swatch in swatches {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
            ui.painter().rect(
                rect,
                CornerRadius::same(3),
                color32(swatch),
                Stroke::new(1.0, theme.border),
                StrokeKind::Inside,
            );
            if response.clicked() {
                let retained_alpha = if alpha { value[3] } else { 1.0 };
                *value = [swatch[0], swatch[1], swatch[2], retained_alpha];
            }
        }
    });
    colors_differ(before, *value)
}

fn show_sv_pad(ui: &mut Ui, hsva: &mut [f32; 4], theme: &TweeqTheme) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(PICKER_WIDTH, PAD_HEIGHT), Sense::click_and_drag());
    if let Some(pointer) = response.interact_pointer_pos() {
        hsva[1] = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        hsva[2] = (1.0 - (pointer.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
    }
    paint_sv_mesh(ui, rect, hsva[0]);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme.input_radius),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );
    let marker = Pos2::new(
        egui::lerp(rect.left()..=rect.right(), hsva[1]),
        egui::lerp(rect.bottom()..=rect.top(), hsva[2]),
    );
    let picked = color32(hsva_to_rgba(*hsva));
    ui.painter()
        .circle_stroke(marker, 5.5, Stroke::new(2.0, contrast_color(picked)));
    response
}

fn show_hue_slider(ui: &mut Ui, hsva: &mut [f32; 4], theme: &TweeqTheme) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(PICKER_WIDTH, SLIDER_HEIGHT),
        Sense::click_and_drag(),
    );
    if let Some(pointer) = response.interact_pointer_pos() {
        hsva[0] = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    }
    paint_horizontal_gradient(ui, rect, |t| color32(hsva_to_rgba([t, 1.0, 1.0, 1.0])));
    paint_slider_marker(ui, rect, hsva[0], theme);
    response
}

fn show_alpha_slider(ui: &mut Ui, hsva: &mut [f32; 4], theme: &TweeqTheme) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(PICKER_WIDTH, SLIDER_HEIGHT),
        Sense::click_and_drag(),
    );
    if let Some(pointer) = response.interact_pointer_pos() {
        hsva[3] = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    }
    paint_checkerboard(ui, rect, 4.0);
    paint_horizontal_gradient(ui, rect, |t| {
        color32(hsva_to_rgba([hsva[0], hsva[1], hsva[2], t]))
    });
    paint_slider_marker(ui, rect, hsva[3], theme);
    response
}

fn paint_slider_marker(ui: &Ui, rect: Rect, value: f32, theme: &TweeqTheme) {
    let x = egui::lerp(rect.left()..=rect.right(), value);
    ui.painter().line_segment(
        [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
        Stroke::new(2.0, theme.text),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(2),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );
}

fn show_hsv_channels(ui: &mut Ui, value: &mut [f32; 4], alpha: bool) {
    let mut hsva = rgba_to_hsva(*value);
    ui.horizontal(|ui| {
        channel(ui, "H", &mut hsva[0], 360.0);
        channel(ui, "S", &mut hsva[1], 100.0);
        channel(ui, "V", &mut hsva[2], 100.0);
        if alpha {
            channel(ui, "A", &mut hsva[3], 100.0);
        }
    });
    *value = hsva_to_rgba(hsva);
}

fn show_rgb_channels(ui: &mut Ui, value: &mut [f32; 4], alpha: bool) {
    ui.horizontal(|ui| {
        channel(ui, "R", &mut value[0], 255.0);
        channel(ui, "G", &mut value[1], 255.0);
        channel(ui, "B", &mut value[2], 255.0);
        if alpha {
            channel(ui, "A", &mut value[3], 100.0);
        }
    });
    for channel in value.iter_mut() {
        *channel = channel.clamp(0.0, 1.0);
    }
}

fn channel(ui: &mut Ui, label: &str, value: &mut f32, scale: f32) {
    ui.vertical(|ui| {
        ui.weak(label);
        let mut displayed = *value * scale;
        if ui
            .add_sized(
                [45.0, 20.0],
                egui::DragValue::new(&mut displayed)
                    .speed(1.0)
                    .range(0.0..=scale),
            )
            .changed()
        {
            *value = (displayed / scale).clamp(0.0, 1.0);
        }
    });
}

fn show_hex_channel(ui: &mut Ui, value: &mut [f32; 4], state: &mut ColorState) {
    let response = ui.add_sized(
        [PICKER_WIDTH, 22.0],
        egui::TextEdit::singleline(&mut state.hex_buffer)
            .font(egui::TextStyle::Monospace)
            .hint_text("#RRGGBBAA"),
    );
    if response.changed()
        && let Some(parsed) = parse_hex(&state.hex_buffer)
    {
        *value = parsed;
    }
    if !response.has_focus() {
        state.hex_buffer = rgba_to_hex(*value);
    }
}

fn color_drag_mode(ui: &Ui, alpha: bool) -> ColorDragMode {
    ui.input(|input| {
        if input.modifiers.alt || input.key_down(Key::A) {
            if alpha {
                ColorDragMode::Alpha
            } else {
                ColorDragMode::SaturationValue
            }
        } else if input.modifiers.shift || input.key_down(Key::H) || input.key_down(Key::F) {
            ColorDragMode::Hue
        } else if input.key_down(Key::S) {
            ColorDragMode::Saturation
        } else if input.key_down(Key::V) {
            ColorDragMode::Value
        } else if input.key_down(Key::R) {
            ColorDragMode::Red
        } else if input.key_down(Key::G) {
            ColorDragMode::Green
        } else if input.key_down(Key::B) {
            ColorDragMode::Blue
        } else {
            ColorDragMode::SaturationValue
        }
    })
}

fn apply_drag(value: &mut [f32; 4], mode: ColorDragMode, motion: Vec2, alpha: bool) {
    let mut hsva = rgba_to_hsva(*value);
    let fine = 0.004;
    match mode {
        ColorDragMode::SaturationValue => {
            hsva[1] = (hsva[1] + motion.x * fine).clamp(0.0, 1.0);
            hsva[2] = (hsva[2] - motion.y * fine).clamp(0.0, 1.0);
            *value = hsva_to_rgba(hsva);
        }
        ColorDragMode::Hue => {
            hsva[0] = (hsva[0] + motion.x * 0.0025).rem_euclid(1.0);
            *value = hsva_to_rgba(hsva);
        }
        ColorDragMode::Saturation => {
            hsva[1] = (hsva[1] + motion.x * fine).clamp(0.0, 1.0);
            *value = hsva_to_rgba(hsva);
        }
        ColorDragMode::Value => {
            hsva[2] = (hsva[2] + motion.x * fine).clamp(0.0, 1.0);
            *value = hsva_to_rgba(hsva);
        }
        ColorDragMode::Alpha => value[3] = (value[3] + motion.x * fine).clamp(0.0, 1.0),
        ColorDragMode::Red => value[0] = (value[0] + motion.x * fine).clamp(0.0, 1.0),
        ColorDragMode::Green => value[1] = (value[1] + motion.x * fine).clamp(0.0, 1.0),
        ColorDragMode::Blue => value[2] = (value[2] + motion.x * fine).clamp(0.0, 1.0),
    }
    if !alpha {
        value[3] = 1.0;
    }
}

fn paint_drag_overlay(
    ui: &Ui,
    id: Id,
    origin: Pos2,
    value: [f32; 4],
    mode: ColorDragMode,
    theme: &TweeqTheme,
) {
    let painter = ui
        .ctx()
        .layer_painter(LayerId::new(Order::Foreground, id.with("drag-overlay")));
    let screen = ui.ctx().content_rect();
    let mut min = Pos2::new(origin.x - 92.0, origin.y - 116.0);
    min.x = min.x.clamp(screen.left() + 8.0, screen.right() - 192.0);
    min.y = min.y.clamp(screen.top() + 8.0, screen.bottom() - 146.0);
    let overlay = Rect::from_min_size(min, Vec2::new(184.0, 138.0));
    painter.rect(
        overlay,
        CornerRadius::same(8),
        theme.surface.gamma_multiply(0.96),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );

    let hsva = rgba_to_hsva(value);
    let pad = Rect::from_min_size(overlay.min + Vec2::new(10.0, 26.0), Vec2::new(164.0, 76.0));
    paint_sv_mesh_with(&painter, pad, hsva[0]);
    let marker = Pos2::new(
        egui::lerp(pad.left()..=pad.right(), hsva[1]),
        egui::lerp(pad.bottom()..=pad.top(), hsva[2]),
    );
    painter.circle_stroke(
        marker,
        4.5,
        Stroke::new(2.0, contrast_color(color32(value))),
    );

    let hue = Rect::from_min_size(overlay.min + Vec2::new(10.0, 108.0), Vec2::new(164.0, 10.0));
    paint_horizontal_gradient_with(&painter, hue, |t| color32(hsva_to_rgba([t, 1.0, 1.0, 1.0])));
    let hue_x = egui::lerp(hue.left()..=hue.right(), hsva[0]);
    painter.line_segment(
        [Pos2::new(hue_x, hue.top()), Pos2::new(hue_x, hue.bottom())],
        Stroke::new(2.0, theme.text),
    );

    painter.text(
        Pos2::new(overlay.left() + 10.0, overlay.top() + 13.0),
        Align2::LEFT_CENTER,
        format!(
            "{}  H {:03.0}  S {:02.0}  V {:02.0}",
            mode.label(),
            hsva[0] * 360.0,
            hsva[1] * 100.0,
            hsva[2] * 100.0
        ),
        FontId::monospace(10.0),
        theme.text,
    );
    painter.circle_filled(
        Pos2::new(origin.x, origin.y),
        6.0,
        color32(value).to_opaque(),
    );
    painter.circle_stroke(
        Pos2::new(origin.x, origin.y),
        6.0,
        Stroke::new(1.5, contrast_color(color32(value))),
    );
}

fn paint_sv_mesh(ui: &Ui, rect: Rect, hue: f32) {
    paint_sv_mesh_with(ui.painter(), rect, hue);
}

fn paint_sv_mesh_with(painter: &egui::Painter, rect: Rect, hue: f32) {
    let subdivisions = 16_u32;
    let mut mesh = Mesh::default();
    for y in 0..=subdivisions {
        for x in 0..=subdivisions {
            let saturation = x as f32 / subdivisions as f32;
            let value = 1.0 - y as f32 / subdivisions as f32;
            let position = Pos2::new(
                egui::lerp(rect.left()..=rect.right(), saturation),
                egui::lerp(rect.top()..=rect.bottom(), y as f32 / subdivisions as f32),
            );
            mesh.colored_vertex(
                position,
                color32(hsva_to_rgba([hue, saturation, value, 1.0])),
            );
            if x < subdivisions && y < subdivisions {
                let row = subdivisions + 1;
                let top_left = y * row + x;
                mesh.add_triangle(top_left, top_left + 1, top_left + row);
                mesh.add_triangle(top_left + 1, top_left + row, top_left + row + 1);
            }
        }
    }
    painter.add(Shape::mesh(mesh));
}

fn paint_horizontal_gradient(ui: &Ui, rect: Rect, color_at: impl Fn(f32) -> Color32) {
    paint_horizontal_gradient_with(ui.painter(), rect, color_at);
}

fn paint_horizontal_gradient_with(
    painter: &egui::Painter,
    rect: Rect,
    color_at: impl Fn(f32) -> Color32,
) {
    let subdivisions = 24_u32;
    let mut mesh = Mesh::default();
    for index in 0..=subdivisions {
        let t = index as f32 / subdivisions as f32;
        let x = egui::lerp(rect.left()..=rect.right(), t);
        mesh.colored_vertex(Pos2::new(x, rect.top()), color_at(t));
        mesh.colored_vertex(Pos2::new(x, rect.bottom()), color_at(t));
        if index < subdivisions {
            let base = index * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 2, base + 3);
        }
    }
    painter.add(Shape::mesh(mesh));
}

fn paint_checkerboard(ui: &Ui, rect: Rect, size: f32) {
    let rows = (rect.height() / size).ceil() as i32;
    let columns = (rect.width() / size).ceil() as i32;
    for row in 0..rows {
        for column in 0..columns {
            let min = Pos2::new(
                rect.left() + column as f32 * size,
                rect.top() + row as f32 * size,
            );
            let cell = Rect::from_min_size(min, Vec2::splat(size)).intersect(rect);
            let color = if (row + column) % 2 == 0 {
                Color32::from_gray(70)
            } else {
                Color32::from_gray(125)
            };
            ui.painter().rect_filled(cell, 0, color);
        }
    }
}

fn rgba_to_hsva(rgba: [f32; 4]) -> [f32; 4] {
    let red = rgba[0].clamp(0.0, 1.0);
    let green = rgba[1].clamp(0.0, 1.0);
    let blue = rgba[2].clamp(0.0, 1.0);
    let maximum = red.max(green).max(blue);
    let minimum = red.min(green).min(blue);
    let delta = maximum - minimum;
    let hue = if delta <= f32::EPSILON {
        0.0
    } else if red >= green && red >= blue {
        ((green - blue) / delta).rem_euclid(6.0) / 6.0
    } else if green >= red && green >= blue {
        ((blue - red) / delta + 2.0) / 6.0
    } else {
        ((red - green) / delta + 4.0) / 6.0
    };
    let saturation = if maximum <= f32::EPSILON {
        0.0
    } else {
        delta / maximum
    };
    [hue, saturation, maximum, rgba[3].clamp(0.0, 1.0)]
}

fn colors_differ(left: [f32; 4], right: [f32; 4]) -> bool {
    left.into_iter()
        .zip(right)
        .any(|(left, right)| left.to_bits() != right.to_bits())
}

fn hsva_to_rgba(hsva: [f32; 4]) -> [f32; 4] {
    let hue = hsva[0].rem_euclid(1.0) * 6.0;
    let saturation = hsva[1].clamp(0.0, 1.0);
    let value = hsva[2].clamp(0.0, 1.0);
    let chroma = value * saturation;
    let x = chroma * (1.0 - (hue.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match hue as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let match_value = value - chroma;
    [
        red + match_value,
        green + match_value,
        blue + match_value,
        hsva[3].clamp(0.0, 1.0),
    ]
}

fn color32(value: [f32; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (value[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (value[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (value[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (value[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn contrast_color(color: Color32) -> Color32 {
    let luminance = 0.2126 * f32::from(color.r())
        + 0.7152 * f32::from(color.g())
        + 0.0722 * f32::from(color.b());
    if luminance < 140.0 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

fn rgba_to_hex(value: [f32; 4]) -> String {
    let color = color32(value);
    if color.a() == 255 {
        format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
    } else {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            color.r(),
            color.g(),
            color.b(),
            color.a()
        )
    }
}

fn parse_hex(text: &str) -> Option<[f32; 4]> {
    let text = text.trim().trim_start_matches('#');
    if text.len() != 6 && text.len() != 8 {
        return None;
    }
    let red = u8::from_str_radix(&text[0..2], 16).ok()?;
    let green = u8::from_str_radix(&text[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&text[4..6], 16).ok()?;
    let alpha = if text.len() == 8 {
        u8::from_str_radix(&text[6..8], 16).ok()?
    } else {
        255
    };
    Some([
        f32::from(red) / 255.0,
        f32::from(green) / 255.0,
        f32::from(blue) / 255.0,
        f32::from(alpha) / 255.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::{hsva_to_rgba, parse_hex, rgba_to_hex, rgba_to_hsva};

    #[test]
    fn hsv_round_trip_preserves_rgba() {
        let source = [0.16, 0.42, 1.0, 0.75];
        let round_trip = hsva_to_rgba(rgba_to_hsva(source));
        for (actual, expected) in round_trip.into_iter().zip(source) {
            assert!((actual - expected).abs() < 0.000_1);
        }
    }

    #[test]
    fn hex_supports_rgb_and_rgba() {
        assert_eq!(rgba_to_hex([1.0, 0.0, 0.5, 1.0]), "#FF0080");
        let parsed = parse_hex("#33669980").expect("valid RGBA hex");
        assert!((parsed[3] - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!(parse_hex("#xyz").is_none());
    }
}
