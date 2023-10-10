use crate::appstate::AppState;
use ratatui::{prelude::*, widgets::*};

pub fn ui_main<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    render_tabs(chunks[0], app_state, f);

    let inner = match app_state.current_tab.index {
        0 => render_lib(chunks[1], f),
        1 => render_updates(chunks[1], f),
        2 => render_sources(chunks[1], f),
        3 => render_history(chunks[1], f),
        4 => render_settings(chunks[1], f),
        _ => unreachable!(),
    };
}

fn render_tabs<B: Backend>(rect: Rect, app_state: &AppState, f: &mut Frame<B>) {
    let unselected_style = Style::default().fg(Color::White);
    let selected_style = Style::default().fg(Color::Green);

    let titles: Vec<Line> = app_state
        .current_tab
        .tabs
        .clone()
        .into_iter()
        .map(|t| Line::from(t).alignment(Alignment::Center))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .select(app_state.current_tab.index)
        .style(unselected_style)
        .highlight_style(selected_style);

    f.render_widget(tabs, rect);
}

fn render_lib<B: Backend>(rect: Rect, f: &mut Frame<B>) {}

fn render_updates<B: Backend>(rect: Rect, f: &mut Frame<B>) {}

fn render_sources<B: Backend>(rect: Rect, f: &mut Frame<B>) {}

fn render_history<B: Backend>(rect: Rect, f: &mut Frame<B>) {}

fn render_settings<B: Backend>(rect: Rect, f: &mut Frame<B>) {}
