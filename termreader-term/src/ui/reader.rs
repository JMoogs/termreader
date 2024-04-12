use crate::AppState;
use chrono::Local;

use ratatui::{prelude::*, widgets::*};

pub fn ui_reader(f: &mut Frame, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    render_reader(chunks[0], app_state, f);
    render_bottom(chunks[1], &app_state, f)
}

fn render_reader(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = block.inner(rect);
    app_state
        .reader_data
        .set_dimensions((inner.width, inner.height));

    f.render_widget(block, rect);
    f.render_stateful_widget(
        app_state.reader_data.get_reader_contents().unwrap(),
        inner,
        app_state.reader_data.get_reader_state_mut().unwrap(),
    );
}

fn render_bottom(rect: Rect, app_state: &AppState, f: &mut Frame) {
    let book = app_state.reader_data.get_book().as_ref().cloned().unwrap();

    let title = book.get_name();

    let time = Local::now().time();
    let time = time.format("%H:%M").to_string();

    let display = if book.is_local() {
        format!("{} | {}", title, time)
    } else {
        let ch = book.global_get_current_ch();
        let ch_name = app_state
            .reader_data
            .get_chapter()
            .as_ref()
            .unwrap()
            .get_name();
        let ch_name = if ch_name.is_empty() {
            "Unnamed chapter"
        } else {
            ch_name
        };
        let ch_percent = app_state
            .reader_data
            .get_ch_progress_pct()
            .expect("There should always be a book at this stage");
        let ch_percent = ch_percent * 100.0;
        format!(
            "{}, Ch {}: {} ({:.2}%) | {}",
            title, ch, ch_name, ch_percent, time
        )
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
