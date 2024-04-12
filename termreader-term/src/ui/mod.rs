pub mod helpers;
pub mod history;
pub mod library;
pub mod reader;
pub mod sources;

use crate::helpers::StatefulList;
use crate::state::LibScreen;
use crate::state::Screen;
use crate::state::SourceScreen;
use crate::ui::helpers::centered_sized_rect;
use crate::ui::history::render_history;
use crate::ui::library::render_lib;
use crate::ui::sources::render_sources;
use crate::AppState;
use crate::Context;
use crate::{SELECTED_STYLE, UNSELECTED_STYLE};
use ratatui::{prelude::*, widgets::*};

/// Manages rendering for the UI when not in reader mode
pub fn ui_main(f: &mut Frame, ctx: &mut Context, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Render the tabs
    if !(app_state.screen == Screen::Sources(SourceScreen::BookView)
        || app_state.screen == Screen::Lib(LibScreen::BookView))
    {
        render_tabs(chunks[0], app_state, f);
    }

    // Render the body of the content, depending on the selected tab
    match app_state.menu_tabs.selected().unwrap().as_str() {
        "Library" => render_lib(chunks[1], ctx, app_state, f),
        "Sources" => render_sources(chunks[1], ctx, app_state, f),
        "History" => render_history(chunks[1], ctx, app_state, f),
        // TODO: Render other tabs
        // _ => unreachable!(),
        _ => (),
    }

    // Render command bar / controls
    if !(app_state.screen == Screen::Sources(SourceScreen::BookView)
        || app_state.screen == Screen::Lib(LibScreen::BookView))
    {
        let text = if app_state.command_bar {
            format!(":{}_", app_state.buffer.text)
        } else {
            String::from(
                "Quit: Esc/q | Scroll tabs: [/] | Scroll categories: {/} | Scroll entries: Up/Down",
            )
        };

        let text = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        f.render_widget(text, chunks[2]);
    }
}

/// Renders the different tabs
fn render_tabs(rect: Rect, app_state: &AppState, f: &mut Frame) {
    let titles: Vec<Line> = Vec::from(app_state.menu_tabs.clone())
        .into_iter()
        .map(|t| Line::from(t).alignment(Alignment::Center))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(UNSELECTED_STYLE)
        .highlight_style(SELECTED_STYLE)
        .select(app_state.menu_tabs.selected_idx().unwrap());

    f.render_widget(tabs, rect);
}

fn render_selection_box(
    rect: Rect,
    box_name: String,
    selection: &mut StatefulList<String>,
    f: &mut Frame,
) {
    let options = selection.items.clone();

    let mut max_width = 0;
    for s in options.iter() {
        if s.len() > max_width {
            max_width = s.len();
        }
    }

    // Ensure that the box name isn't too long for the box
    max_width = max_width.max(box_name.len());

    max_width += 5; // + 2 for borders, + 2 for selection +1 to make it look better

    let list: Vec<ListItem> = options
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let height = list.len() + 2;

    let display = List::new(list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(box_name)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    let r = centered_sized_rect(max_width as u16, height as u16, rect);
    f.render_stateful_widget(display, r, selection.state_mut())
}

fn render_type_box(rect: Rect, app_state: &mut AppState, f: &mut Frame, title: String) {
    let mut text = app_state.buffer.text.clone();
    text.push('_');

    let display = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: false });

    let r = centered_sized_rect(40, 6, rect);

    f.render_widget(display, r)
}

fn render_selection_screen(
    rect: Rect,
    box_name: String,
    selection: &mut StatefulList<String>,
    f: &mut Frame,
) {
    let options = selection.items.clone();

    let list: Vec<ListItem> = options
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let display = List::new(list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(box_name)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(display, rect, selection.state_mut())
}
