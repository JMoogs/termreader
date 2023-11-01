use crate::{
    appstate::AppState,
    reader::{buffer::NextDisplay, widget::BookText},
};
use ratatui::{prelude::*, widgets::*};

pub fn ui_reader<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    render_reader(chunks[0], app_state, f);
    render_bottom(chunks[1], app_state, f)
}

fn render_reader<B: Backend>(rect: Rect, app_state: &mut AppState, f: &mut Frame<B>) {
    let data = &mut app_state.reader_data.as_mut().unwrap();

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = block.inner(rect);

    f.render_widget(block, rect);
    f.render_stateful_widget(BookText::new(), inner, &mut data.portion);

    data.portion.display_next = NextDisplay::NoOp;
}

fn render_bottom<B: Backend>(rect: Rect, app_state: &AppState, f: &mut Frame<B>) {}
// /// helper function to create a centered rect using up certain percentage of the available rect `r`
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
