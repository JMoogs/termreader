use crate::{
    appstate::{
        AppState, CurrentScreen, HistoryOptions, LibraryOptions, MiscOptions, SourceOptions,
    },
    helpers::to_datetime,
    ui::mainscreen::bookoptions::{
        render_category_box, render_category_options, render_local_selection, render_type_box,
    },
    ui::{mainscreen::sourceoptions::render_source_selection, render_loading_popup},
    SELECTED_STYLE, UNSELECTED_STYLE,
};
use ratatui::{prelude::*, widgets::*};

use super::{
    bookoptions::{
        render_ch_list, render_global_selection, render_global_selection_history,
        render_local_selection_history,
    },
    sourceoptions::{render_search_results, render_source_book},
};

pub fn ui_main(f: &mut Frame, app_state: &mut AppState) {
    if app_state.channels.loading {
        render_loading_popup(f);
        return;
    }

    if matches!(
        app_state.current_screen,
        CurrentScreen::Sources(SourceOptions::BookView)
    ) {
        render_source_book(f, app_state);
        return;
    }
    if matches!(
        app_state.current_screen,
        CurrentScreen::Misc(MiscOptions::ChapterView)
    ) {
        render_ch_list(app_state, f);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    render_tabs(chunks[0], app_state, f);

    match app_state.current_main_tab.selected_idx() {
        0 => render_lib(chunks[1], app_state, f),
        1 => render_updates(chunks[1], f),
        2 => render_sources(chunks[1], app_state, f),
        3 => render_history(chunks[1], app_state, f),
        4 => render_settings(chunks[1], f),
        _ => unreachable!(),
    };

    // The command list is rendered below:
    //
    //
    let text = if !app_state.command_bar {
        String::from(
            "Quit: Esc/q | Scroll tabs: [/] | Scroll categories: {/} | Scroll entries: Up/Down",
        )
    } else {
        format!(":{}_", app_state.buffer.text)
    };

    let text = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(text, chunks[2]);
}

fn render_tabs(rect: Rect, app_state: &AppState, f: &mut Frame) {
    let titles: Vec<Line> = app_state
        .current_main_tab
        .tabs()
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
        .select(app_state.current_main_tab.selected_idx())
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
        ])
        .split(rect);

    // The reading list categories are rendered below:
    //
    //
    let categories: Vec<Line> = app_state
        .library_data
        .categories
        .items
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
        .select(app_state.library_data.categories.selected_idx().unwrap())
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
        app_state.library_data.get_category_list_mut().state_mut(),
    );

    // Render a centered box on top if in certain menus.
    match app_state.current_screen {
        CurrentScreen::Library(LibraryOptions::LocalBookSelect) => {
            render_local_selection(rect, app_state, f);
        }
        CurrentScreen::Library(LibraryOptions::GlobalBookSelect) => {
            render_global_selection(rect, app_state, f);
        }
        CurrentScreen::Typing => {
            render_type_box(rect, app_state, f);
        }
        CurrentScreen::Library(LibraryOptions::CategorySelect) => {
            render_category_box(chunks[1], app_state, f)
        }
        CurrentScreen::Library(LibraryOptions::CategoryOptions) => {
            render_category_options(rect, app_state, f)
        }
        _ => (),
    }
}

fn render_updates(_rect: Rect, _f: &mut Frame) {}

fn render_sources(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
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
        f.render_stateful_widget(
            sources,
            chunks[0],
            app_state.source_data.sources.state_mut(),
        );
    }

    if let CurrentScreen::Sources(SourceOptions::SourceSelect) = app_state.current_screen {
        render_source_selection(rect, app_state, f);
    } else if let CurrentScreen::Typing = app_state.current_screen {
        render_type_box(rect, app_state, f)
    }

    if let CurrentScreen::Sources(SourceOptions::SearchResults) = app_state.current_screen {
        render_search_results(chunks[0], app_state, f, "Results");
    }
}

fn render_history(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(rect);

    let display_data: Vec<ListItem> = app_state
        .history_data
        .history
        .clone()
        .into_iter()
        .map(|s| {
            if s.chapter == 0 {
                format!(
                    "{} | {}",
                    s.book.get_source_data().get_name(),
                    to_datetime(s.timestamp)
                )
            } else {
                format!(
                    "{} | Chapter {} | {}",
                    s.book.get_source_data().get_name(),
                    s.chapter,
                    to_datetime(s.timestamp)
                )
            }
        })
        .map(|f| ListItem::new(f).style(UNSELECTED_STYLE))
        .collect();

    let history = List::new(display_data)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("History")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(history, chunks[0], &mut app_state.history_data.selected);

    // Menu boxes
    if let CurrentScreen::History(HistoryOptions::HistoryLocalBookOptions) =
        app_state.current_screen
    {
        render_local_selection_history(rect, app_state, f);
    } else if let CurrentScreen::History(HistoryOptions::HistoryGlobalBookOptions) =
        app_state.current_screen
    {
        render_global_selection_history(rect, app_state, f);
    }
}

fn render_settings(_rect: Rect, _f: &mut Frame) {}
