use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub selected_style: Style,
    pub unselected_style: Style,
    pub greyed_style: Style,
}

impl ConfigData {
    pub const DEFAULT_UNSELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::White);
    pub const DEFAULT_SELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::Green);
    pub const DEFAULT_SELECTED_STYLE_2: ratatui::style::Style = Style::new().fg(Color::Yellow);
    pub const DEFAULT_GREYED_STYLE: ratatui::style::Style = Style::new().fg(Color::DarkGray);

    pub fn build() -> Self {
        ConfigData {
            selected_style: Self::DEFAULT_SELECTED_STYLE,
            unselected_style: Self::DEFAULT_UNSELECTED_STYLE,
            greyed_style: Self::DEFAULT_GREYED_STYLE,
        }
    }
}
