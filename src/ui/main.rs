use crate::appstate::{AppState, MainTabs};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn ui_main<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    render_tabs(chunks, app_state, f);
}

fn render_tabs<B: Backend>(rect: std::rc::Rc<[Rect]>, app_state: &AppState, f: &mut Frame<B>) {
    let top_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(rect[0]);

    let unselected_style = Style::default().fg(Color::White);
    let selected_style = Style::default().fg(Color::Green);

    let tab_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default());

    let main_selection = app_state.current_tab.unwrap();

    let lib = if let MainTabs::Library = main_selection {
        Paragraph::new(Text::styled("Library", selected_style))
            .block(tab_block.clone())
            .style(selected_style)
    } else {
        Paragraph::new(Text::styled("Library", unselected_style))
            .block(tab_block.clone())
            .style(unselected_style)
    };

    let updates = if let MainTabs::Updates = main_selection {
        Paragraph::new(Text::styled("Updates", selected_style))
            .block(tab_block.clone())
            .style(selected_style)
    } else {
        Paragraph::new(Text::styled("Updates", unselected_style))
            .block(tab_block.clone())
            .style(unselected_style)
    };

    let sources = if let MainTabs::Sources = main_selection {
        Paragraph::new(Text::styled("Sources", selected_style))
            .block(tab_block.clone())
            .style(selected_style)
    } else {
        Paragraph::new(Text::styled("Sources", unselected_style))
            .block(tab_block.clone())
            .style(unselected_style)
    };

    let history = if let MainTabs::History = main_selection {
        Paragraph::new(Text::styled("History", selected_style))
            .block(tab_block.clone())
            .style(selected_style)
    } else {
        Paragraph::new(Text::styled("History", unselected_style))
            .block(tab_block.clone())
            .style(unselected_style)
    };

    let settings = if let MainTabs::Settings = main_selection {
        Paragraph::new(Text::styled("Settings", selected_style))
            .block(tab_block)
            .style(selected_style)
    } else {
        Paragraph::new(Text::styled("Settings", unselected_style))
            .block(tab_block)
            .style(unselected_style)
    };

    f.render_widget(lib, top_bar[0]);
    f.render_widget(updates, top_bar[1]);
    f.render_widget(sources, top_bar[2]);
    f.render_widget(history, top_bar[3]);
    f.render_widget(settings, top_bar[4]);
}
