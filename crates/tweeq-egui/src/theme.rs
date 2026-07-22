use egui::{Color32, Context, CornerRadius, Stroke, Theme, Visuals};

/// Tweeq appearance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Bright surfaces with dark text.
    Light,
    /// Dark surfaces with bright text.
    Dark,
}

/// Semantic colors and metrics shared by Tweeq widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct TweeqTheme {
    /// Light or dark appearance.
    pub mode: ColorMode,
    /// Application background.
    pub background: Color32,
    /// Raised surface background.
    pub surface: Color32,
    /// Primary text.
    pub text: Color32,
    /// Secondary text.
    pub text_muted: Color32,
    /// Primary accent.
    pub accent: Color32,
    /// Hovered accent.
    pub accent_hover: Color32,
    /// Input background.
    pub input: Color32,
    /// Hovered input background.
    pub input_hover: Color32,
    /// Subtle border.
    pub border: Color32,
    /// Standard input height in points.
    pub input_height: f32,
    /// Standard input corner radius in points.
    pub input_radius: u8,
    /// Closely related item gap.
    pub related_gap: f32,
}

impl TweeqTheme {
    /// Default light appearance derived from the Vue reference tokens.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            mode: ColorMode::Light,
            background: Color32::from_rgb(255, 255, 255),
            surface: Color32::from_rgb(250, 250, 250),
            text: Color32::from_rgb(31, 31, 31),
            text_muted: Color32::from_rgb(100, 100, 100),
            accent: Color32::from_rgb(0, 82, 255),
            accent_hover: Color32::from_rgb(0, 64, 214),
            input: Color32::from_rgb(239, 240, 243),
            input_hover: Color32::from_rgb(233, 233, 236),
            border: Color32::from_rgb(214, 214, 220),
            input_height: 24.0,
            input_radius: 4,
            related_gap: 6.0,
        }
    }

    /// Default dark appearance derived from the Vue reference tokens.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            mode: ColorMode::Dark,
            background: Color32::from_rgb(17, 17, 17),
            surface: Color32::from_rgb(29, 29, 31),
            text: Color32::from_rgb(238, 238, 240),
            text_muted: Color32::from_rgb(166, 166, 172),
            accent: Color32::from_rgb(74, 118, 255),
            accent_hover: Color32::from_rgb(103, 140, 255),
            input: Color32::from_rgb(38, 38, 41),
            input_hover: Color32::from_rgb(49, 49, 53),
            border: Color32::from_rgb(65, 65, 70),
            input_height: 24.0,
            input_radius: 4,
            related_gap: 6.0,
        }
    }

    /// Applies the global parts of this theme to an egui context.
    ///
    /// Widgets still use their local semantic tokens, so applications may skip
    /// this method when they need to retain an existing global style.
    pub fn install(&self, context: &Context) {
        let egui_theme = match self.mode {
            ColorMode::Light => Theme::Light,
            ColorMode::Dark => Theme::Dark,
        };
        let mut style = (*context.style_of(egui_theme)).clone();
        let mut visuals = match egui_theme {
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
        };

        visuals.panel_fill = self.background;
        visuals.window_fill = self.surface;
        visuals.override_text_color = Some(self.text);
        visuals.selection.bg_fill = self.accent;
        visuals.widgets.inactive.bg_fill = self.input;
        visuals.widgets.hovered.bg_fill = self.input_hover;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text);
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(self.input_radius);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(self.input_radius);
        visuals.widgets.active.corner_radius = CornerRadius::same(self.input_radius);
        style.visuals = visuals;
        style.spacing.interact_size.y = self.input_height;
        style.spacing.item_spacing.x = self.related_gap;
        context.set_style_of(egui_theme, style);
        context.set_theme(egui_theme);
    }
}

impl Default for TweeqTheme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorMode, TweeqTheme};

    #[test]
    fn presets_have_expected_modes() {
        assert_eq!(TweeqTheme::light().mode, ColorMode::Light);
        assert_eq!(TweeqTheme::dark().mode, ColorMode::Dark);
    }
}
