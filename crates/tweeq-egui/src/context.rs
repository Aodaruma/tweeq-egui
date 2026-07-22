use crate::TweeqTheme;

/// App-owned Tweeq state shared by widgets.
#[derive(Debug, Clone)]
pub struct TweeqContext {
    theme: TweeqTheme,
}

impl TweeqContext {
    /// Creates a context using the given theme.
    #[must_use]
    pub const fn new(theme: TweeqTheme) -> Self {
        Self { theme }
    }

    /// Returns the active theme.
    #[must_use]
    pub const fn theme(&self) -> &TweeqTheme {
        &self.theme
    }

    /// Replaces the active theme.
    pub fn set_theme(&mut self, theme: TweeqTheme) {
        self.theme = theme;
    }
}

impl Default for TweeqContext {
    fn default() -> Self {
        Self::new(TweeqTheme::dark())
    }
}
