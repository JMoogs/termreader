use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use self::helpers::centered_rect;

pub mod controls;
pub mod helpers;
pub mod mainscreen;
pub mod reader;

fn render_loading_popup(f: &mut Frame) {
    let area = centered_rect(30, 30, f.size());

    let para = Paragraph::new("Loading...")
        .style(Style::new().add_modifier(Modifier::BOLD))
        .block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    f.render_widget(para, area)
}
