use crate::{
    appstate::AppState,
    helpers::to_datetime,
    state::screen::{HistoryOptions, LibraryScreen, MiscOptions, Screen, SourceOptions},
    ui::mainscreen::bookoptions::render_type_box,
    ui::{mainscreen::sourceoptions::render_source_selection, render_loading_popup},
    SELECTED_STYLE, UNSELECTED_STYLE,
};
use ratatui::{prelude::*, widgets::*};

use super::{
    bookoptions::{render_ch_list, render_selection_box},
    sourceoptions::{render_search_results, render_source_book},
};

pub fn ui_main(f: &mut Frame, app_state: &mut AppState) {
    if app_state.channels.loading {
        render_loading_popup(f.size(), f);
    }

    if matches!(
        app_state.current_screen,
        Screen::Sources(SourceOptions::BookView)
    ) {
        render_source_book(f, app_state);
        return;
    }
    if matches!(
        app_state.current_screen,
        Screen::Misc(MiscOptions::ChapterView)
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

    match app_state.tab.selected().unwrap().as_str() {
        "Library" => render_lib(chunks[1], app_state, f),
        "Updates" => render_updates(chunks[1], f),
        "Sources" => render_sources(chunks[1], app_state, f),
        "History" => render_history(chunks[1], app_state, f),
        "Settings" => render_settings(chunks[1], f),
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
        .tab
        .to_vec()
        .into_iter()
        .map(|t| Line::from(t).alignment(Alignment::Center))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .select(app_state.tab.selected_idx().unwrap())
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
        .context
        .library
        .get_categories()
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
        .select(app_state.states.get_selected_category(&app_state.context))
        .style(UNSELECTED_STYLE)
        .highlight_style(SELECTED_STYLE);

    f.render_widget(tabs, chunks[0]);

    // The reading list is rendered below:
    //
    //
    let display_data: Vec<ListItem> = app_state
        .states
        .get_category_books(&app_state.context)
        .iter()
        .map(|s| ListItem::new(s.to_string()).style(UNSELECTED_STYLE))
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
        app_state.states.get_category_list_state(&app_state.context),
    );

    // Render a centered box on top if in certain menus.
    match app_state.current_screen {
        Screen::Library(LibraryScreen::BookSelect)
        | Screen::Library(LibraryScreen::CategorySelect)
        | Screen::Library(LibraryScreen::CategoryOptions) => {
            render_selection_box(rect, app_state, f);
        }
        Screen::Typing => {
            render_type_box(rect, app_state, f, "Input Text:".into());
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

    let items = app_state.context.sources.get_source_names();
    let display_data: Vec<ListItem> = items
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

    if let Screen::Sources(SourceOptions::SearchResults) = app_state.current_screen {
        render_search_results(chunks[0], app_state, f, "Results");
    } else {
        f.render_stateful_widget(sources, chunks[0], &mut app_state.states.selection_box);
    }

    if let Screen::Sources(SourceOptions::SourceSelect) = app_state.current_screen {
        render_source_selection(rect, app_state, f);
    } else if let Screen::Typing = app_state.current_screen {
        render_type_box(rect, app_state, f, "Search sources:".into())
    }
}

fn render_history(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(rect);

    let display_data: Vec<ListItem> = app_state
        .context
        .history
        .get_history()
        .clone()
        .into_iter()
        .map(|e| {
            if e.get_chapter() == 0 {
                format!("{} | {}", e.get_book_name(), to_datetime(e.get_timestamp()))
            } else {
                format!(
                    "{} | Chapter {} | {}",
                    e.get_book_name(),
                    e.get_chapter(),
                    to_datetime(e.get_timestamp())
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

    f.render_stateful_widget(history, chunks[0], &mut app_state.states.history_selection);

    // Menu boxes
    if matches!(
        app_state.current_screen,
        Screen::History(HistoryOptions::HistoryBookOptions)
    ) {
        render_selection_box(rect, app_state, f);
    }
}

fn render_settings(_rect: Rect, _f: &mut Frame) {}
