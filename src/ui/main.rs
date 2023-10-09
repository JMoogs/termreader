use crate::appstate::AppState;
use ratatui::{prelude::*, widgets::*};

pub fn ui_main<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    render_tabs(chunks[0], app_state, f);
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

    f.render_widget(tabs, rect)
}
