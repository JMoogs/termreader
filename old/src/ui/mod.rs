use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub mod controls;
pub mod helpers;
pub mod mainscreen;
pub mod reader;

fn render_loading_popup(rect: Rect, f: &mut Frame) {
    let para = Paragraph::new("Loading...")
        .style(Style::new().add_modifier(Modifier::BOLD))
        .block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    f.render_widget(para, rect)
}
