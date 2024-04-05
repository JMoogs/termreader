use crate::{
    appstate::AppState,
    reader::{buffer::NextDisplay, widget::BookText},
};
use chrono::Local;
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

fn render_bottom(rect: Rect, app_state: &AppState, f: &mut Frame) {
    let book = &app_state.reader_data.as_ref().unwrap().book_info;

    let title = book.get_source_data().get_name();

    let progress_pct = app_state
        .reader_data
        .as_ref()
        .unwrap()
        .get_progress()
        .unwrap()
        .get_pct();

    let time = Local::now().time();
    let time = time.format("%H:%M").to_string();

    let display = if book.is_local() {
        format!("{} | {:.2}% | {}", title, progress_pct, time)
    } else {
        let ch = app_state
            .reader_data
            .as_ref()
            .unwrap()
            .book_info
            .get_source_data()
            .get_chapter();
        format!("{}, Ch {} | {:.2}% | {}", title, ch, progress_pct, time)
    };

    let display = if !app_state.command_bar {
        display
    } else {
        format!(":{}_", app_state.buffer.text)
    };

    let text = Paragraph::new(display).block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    f.render_widget(text, rect)
}
