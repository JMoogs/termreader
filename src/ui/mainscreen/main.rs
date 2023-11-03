use crate::appstate::{AppState, CurrentScreen, MenuType};
use ratatui::{prelude::*, widgets::*};

use super::bookoptions::{render_local_selection, render_mv_category_box, render_rename_box};

pub fn ui_main(f: &mut Frame, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    render_tabs(chunks[0], app_state, f);

    match app_state.current_main_tab.index {
        0 => render_lib(chunks[1], app_state, f),
        1 => render_updates(chunks[1], f),
        2 => render_sources(chunks[1], f),
        3 => render_history(chunks[1], f),
        4 => render_settings(chunks[1], f),
        _ => unreachable!(),
    };
}

fn render_tabs(rect: Rect, app_state: &AppState, f: &mut Frame) {
    let unselected_style = Style::default().fg(Color::White);
    let selected_style = Style::default().fg(Color::Green);

    let titles: Vec<Line> = app_state
        .current_main_tab
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
        .select(app_state.current_main_tab.index)
        .style(unselected_style)
        .highlight_style(selected_style);

    f.render_widget(tabs, rect);
}

fn render_lib(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let unselected_style = Style::default().fg(Color::White);
    let selected_style = Style::default().fg(Color::Green);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Category tabs
            Constraint::Length(3),
            // Book list
            Constraint::Min(1),
            // Controls / other
            Constraint::Length(3),
        ])
        .split(rect);

    // The reading list categories are rendered below:
    //
    //
    let categories: Vec<Line> = app_state
        .library_data
        .categories
        .tabs
        .clone()
        .into_iter()
        .map(|t| Line::from(t).alignment(Alignment::Center))
        .collect();

    let tabs = Tabs::new(categories)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Categories")
                .border_type(BorderType::Rounded),
        )
        .select(app_state.library_data.categories.index)
        .style(unselected_style)
        .highlight_style(selected_style);

    f.render_widget(tabs, chunks[0]);

    // The reading list is rendered below:
    //
    //
    let display_data: Vec<ListItem> = app_state
        .library_data
        .get_category_list()
        .items
        .iter()
        .map(|f| {
            let s = f.display_info();
            ListItem::new(s).style(unselected_style)
        })
        .collect();

    let books = List::new(display_data)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Books")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(
        books,
        chunks[1],
        &mut app_state.library_data.get_category_list_mut().state,
    );

    // The command list is rendered below:
    //
    //
    let commands = Paragraph::new(
        "Quit: Esc/q | Scroll tabs: [/] | Scroll categories: {/} | Scroll entries: Up/Down",
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    f.render_widget(commands, chunks[2]);

    // Render a centered box on top if in certain menus.
    if let CurrentScreen::Main(MenuType::LocalSelected) = app_state.current_screen {
        render_local_selection(rect, app_state, f);
    } else if let CurrentScreen::Main(MenuType::Renaming(_)) = app_state.current_screen {
        // let rect = centered_rect(50, 15, chunks[1]);
        render_rename_box(rect, app_state, f);
    } else if let CurrentScreen::Main(MenuType::MoveCategories) = app_state.current_screen {
        render_mv_category_box(chunks[1], app_state, f)
    }
}

fn render_updates(_rect: Rect, _f: &mut Frame) {}

fn render_sources(_rect: Rect, _f: &mut Frame) {}

fn render_history(_rect: Rect, _f: &mut Frame) {}

fn render_settings(_rect: Rect, _f: &mut Frame) {}
