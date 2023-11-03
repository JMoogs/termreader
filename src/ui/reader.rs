use crate::{
    appstate::AppState,
    reader::{buffer::NextDisplay, widget::BookText},
};
use ratatui::{prelude::*, widgets::*};

pub fn ui_reader(f: &mut Frame, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    render_reader(chunks[0], app_state, f);
    render_bottom(chunks[1], app_state, f)
}

fn render_reader(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let data = &mut app_state.reader_data.as_mut().unwrap();

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = block.inner(rect);

    f.render_widget(block, rect);
    f.render_stateful_widget(BookText::new(), inner, &mut data.portion);

    data.portion.display_next = NextDisplay::NoOp;
}

fn render_bottom(_rect: Rect, _app_state: &AppState, _f: &mut Frame) {}
