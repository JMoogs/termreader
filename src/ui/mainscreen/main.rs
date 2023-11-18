use crate::{
    appstate::{AppState, CurrentScreen, LibraryOptions, SourceOptions},
    ui::mainscreen::bookoptions::{
        render_local_selection, render_mv_category_box, render_type_box,
    },
    ui::mainscreen::sourceoptions::render_source_selection,
    SELECTED_STYLE, UNSELECTED_STYLE,
};
use ratatui::{prelude::*, widgets::*};

use super::{
    bookoptions::{render_global_selection, render_lib_ch_list},
    sourceoptions::{render_search_results, render_source_book},
};

pub fn ui_main(f: &mut Frame, app_state: &mut AppState) {
    if matches!(
        app_state.current_screen,
        CurrentScreen::Sources(SourceOptions::BookView)
    ) {
        render_source_book(f, app_state);
        return;
    }
    if matches!(
        app_state.current_screen,
        CurrentScreen::Library(LibraryOptions::ChapterView)
    ) {
        render_lib_ch_list(app_state, f);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    render_tabs(chunks[0], app_state, f);

    match app_state.current_main_tab.index {
        0 => render_lib(chunks[1], app_state, f),
        1 => render_updates(chunks[1], f),
        2 => render_sources(chunks[1], app_state, f),
        3 => render_history(chunks[1], f),
        4 => render_settings(chunks[1], f),
        _ => unreachable!(),
    };
}

fn render_tabs(rect: Rect, app_state: &AppState, f: &mut Frame) {
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
        .style(UNSELECTED_STYLE)
        .highlight_style(SELECTED_STYLE);

    f.render_widget(tabs, rect);
}

fn render_lib(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
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
        .style(UNSELECTED_STYLE)
        .highlight_style(SELECTED_STYLE);

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
            ListItem::new(s).style(UNSELECTED_STYLE)
        })
        .collect();

    let books = List::new(display_data)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Books")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
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
    if let CurrentScreen::Library(LibraryOptions::LocalBookSelect) = app_state.current_screen {
        render_local_selection(rect, app_state, f);
    } else if let CurrentScreen::Library(LibraryOptions::GlobalBookSelect) =
        app_state.current_screen
    {
        render_global_selection(rect, app_state, f);
    } else if let CurrentScreen::Typing = app_state.current_screen {
        render_type_box(rect, app_state, f);
    } else if let CurrentScreen::Library(LibraryOptions::MoveCategorySelect) =
        app_state.current_screen
    {
        render_mv_category_box(chunks[1], app_state, f)
    }
}

fn render_updates(_rect: Rect, _f: &mut Frame) {}

fn render_sources(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(rect);

    let display_data: Vec<ListItem> = app_state
        .source_data
        .get_source_names()
        .into_iter()
        .map(|f| ListItem::new(f).style(UNSELECTED_STYLE))
        .collect();

    let sources = List::new(display_data)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sources")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    if let CurrentScreen::Sources(SourceOptions::SearchResults) = app_state.current_screen {
        render_search_results(chunks[0], app_state, f, "Results");
    } else {
        f.render_stateful_widget(sources, chunks[0], app_state.source_data.get_state_mut());
    }

    let commands = Paragraph::new("Quit: Esc/q | Scroll categories: {/} | Scroll entries: Up/Down")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    f.render_widget(commands, chunks[1]);

    if let CurrentScreen::Sources(SourceOptions::SourceSelect) = app_state.current_screen {
        render_source_selection(rect, app_state, f);
    } else if let CurrentScreen::Typing = app_state.current_screen {
        render_type_box(rect, app_state, f)
    }

    if let CurrentScreen::Sources(SourceOptions::SearchResults) = app_state.current_screen {
        render_search_results(chunks[0], app_state, f, "Results");
    }
}

fn render_history(_rect: Rect, _f: &mut Frame) {}

fn render_settings(_rect: Rect, _f: &mut Frame) {}
