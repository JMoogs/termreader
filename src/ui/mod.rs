pub mod main;
pub mod reader;

// use crate::appstate::AppState;
// use ratatui::{
//     prelude::*,
//     widgets::{Block, Borders, Paragraph},
// };

// pub fn ui<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Length(3),
//             Constraint::Min(1),
//             Constraint::Length(3),
//         ])
//         .split(f.size());

//     let title_block = Block::default()
//         .borders(Borders::ALL)
//         .style(Style::default());

//     let title = Paragraph::new(Text::styled("RReader", Style::default().fg(Color::Green)))
//         .block(title_block);

//     f.render_widget(title, chunks[0])
// }

// /// helper function to create a centered rect using up a certain percentage of the available rect `r`
// fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     // Cut the given rectangle into three vertical pieces
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage((100 - percent_y) / 2),
//             Constraint::Percentage(percent_y),
//             Constraint::Percentage((100 - percent_y) / 2),
//         ])
//         .split(r);

//     // Then cut the middle vertical piece into three width-wise pieces
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage((100 - percent_x) / 2),
//             Constraint::Percentage(percent_x),
//             Constraint::Percentage((100 - percent_x) / 2),
//         ])
//         .split(popup_layout[1])[1] // Return the middle chunk
// }
