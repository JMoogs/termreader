use crate::state::LibScreen;
use crate::state::Screen;
use crate::AppState;
use crate::Context;
use crate::{SELECTED_STYLE, UNSELECTED_STYLE};
use ratatui::{prelude::*, widgets::*};

use super::render_selection_box;
use super::render_type_box;
use super::sources::render_book_view;

/// Renders the library tab
pub(super) fn render_lib(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    if app_state.screen == Screen::Lib(LibScreen::BookView) {
        render_book_view(f, app_state, false);
        return;
    }
    // Split into two chunks, one for the categories, and one for the book lists
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Category tabs
            Constraint::Length(3),
            // Book list
            Constraint::Min(1),
        ])
        .split(rect);

    // Render categories
    let categories: Vec<Line> = ctx
        .lib_get_categories()
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
        // .select(app_state.get_selected_category(&app_state.context))
        .style(UNSELECTED_STYLE)
        .highlight_style(SELECTED_STYLE)
        .select(app_state.lib_data.get_selected_category());

    f.render_widget(tabs, chunks[0]);

    // Render the books
    let current_category = &ctx.lib_get_categories()[app_state.lib_data.get_selected_category()];
    let mut display_data: Vec<ListItem> = ctx
        .lib_get_books()
        .get(current_category)
        .unwrap()
        .iter()
        .map(|b| ListItem::new(b.display_info()).style(UNSELECTED_STYLE))
        .collect();
    let book_len = display_data.len();

    if book_len == 0 {
        display_data.push(ListItem::new("There are no books in this category."))
    }

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
        app_state.lib_data.get_selected_book_state_mut(),
    );

    // TODO: Render a selection box on top of this page when required
    if let Screen::Lib(LibScreen::GlobalBookSelect) = app_state.screen {
        if app_state.typing {
            render_type_box(
                chunks[1],
                app_state,
                f,
                "New name (leave blank for original):".into(),
            )
        } else {
            render_selection_box(
                chunks[1],
                String::from("Options"),
                &mut app_state.lib_data.global_selected_book_opts,
                f,
            );
        }
    }
}
