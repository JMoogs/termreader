use ratatui::style::{Color, Style};

#[derive(Debug, Clone)]
pub struct ConfigData {
    pub selected_style: Style,
    pub unselected_style: Style,
}

impl ConfigData {
    pub const DEFAULT_UNSELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::White);
    pub const DEFAULT_SELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::Green);
    pub const DEFAULT_SELECTED_STYLE_2: ratatui::style::Style = Style::new().fg(Color::Yellow);

    pub fn build() -> Self {
        ConfigData {
            selected_style: Self::DEFAULT_SELECTED_STYLE,
            unselected_style: Self::DEFAULT_UNSELECTED_STYLE,
        }
    }
}
