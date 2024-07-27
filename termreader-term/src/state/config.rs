use anyhow::Result;
use std::{fs, path::PathBuf};

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub selected_style: Style,
    pub selected_style_2: Style,
    pub unselected_style: Style,
    pub greyed_style: Style,
    #[serde(skip)]
    pub prompt_style: Option<Style>,
}

impl Default for ConfigData {
    fn default() -> Self {
        ConfigData {
            selected_style: Self::DEFAULT_SELECTED_STYLE,
            selected_style_2: Self::DEFAULT_SELECTED_STYLE_2,
            unselected_style: Self::DEFAULT_UNSELECTED_STYLE,
            greyed_style: Self::DEFAULT_GREYED_STYLE,
            prompt_style: None,
        }
    }
}

impl ConfigData {
    pub const DEFAULT_UNSELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::White);
    pub const DEFAULT_SELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::Green);
    pub const DEFAULT_SELECTED_STYLE_2: ratatui::style::Style = Style::new().fg(Color::Yellow);
    pub const DEFAULT_GREYED_STYLE: ratatui::style::Style = Style::new().fg(Color::DarkGray);

    pub fn save(self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string(&self)?;
        std::fs::write(path.join("config.json"), json)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Result<ConfigData> {
        if let Ok(data) = fs::read_to_string(path.join("config.json")) {
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(ConfigData::default())
        }
    }

    pub fn get_prompt_style(&self) -> Style {
        self.prompt_style.unwrap_or_else(|| self.selected_style)
    }

    pub fn reset_styles(&mut self) {
        self.prompt_style = None;
    }
}
