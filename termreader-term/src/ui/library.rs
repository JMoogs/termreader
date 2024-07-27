use crate::state::LibScreen;
use crate::state::Screen;
use crate::AppState;
use crate::Context;
use ratatui::{prelude::*, widgets::*};

use super::render_selection_box;
use super::render_selection_screen;
use super::render_type_box;
use super::sources::render_book_view;
use super::sources::BookViewOption;

/// Renders the library tab
pub(super) fn render_lib(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    let Screen::Lib(libscreen) = app_state.screen else {
        // We'll never be rendering the library screen if we aren't on it
        unreachable!()
    };

    let mut render_book_v = false;
    let mut render_categories = false;
    let mut render_books = false;
    let mut render_category_list = false;
    let mut render_category_options = false;

    match libscreen {
        LibScreen::Main => {
            render_books = true;
            render_categories = true;
        }
        LibScreen::BookView | LibScreen::BookViewCategory => {
            render_book_v = true;
        }
        LibScreen::CategorySelect => {
            render_categories = true;
            render_category_list = true;
        }
        LibScreen::CategoryOptions => {
            render_categories = true;
            render_category_options = true;
        }
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

    if render_book_v {
        render_book_view(f, app_state, ctx, BookViewOption::LibOptions);
    }

    if render_category_list {
        if app_state.typing {
            render_type_box(rect, app_state, f, "New Name:".into())
        } else {
            render_selection_box(
                &app_state.config,
                chunks[1],
                String::from("Pick category:"),
                &mut app_state.buffer.temporary_list,
                f,
            );
        }
    }

    if render_categories {
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
            .style(app_state.config.unselected_style)
            .highlight_style(app_state.config.selected_style)
            .select(app_state.lib_data.get_selected_category());

        f.render_widget(tabs, chunks[0]);
    }

    if render_books {
        let current_category =
            &ctx.lib_get_categories()[app_state.lib_data.get_selected_category()];

        let mut display_data: Vec<ListItem> = ctx
            .lib_get_books()
            .get(current_category)
            .unwrap()
            .iter()
            .map(|b| ListItem::new(b.display_info()).style(app_state.config.unselected_style))
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
            .highlight_style(app_state.config.selected_style)
            .highlight_symbol("> ");

        f.render_stateful_widget(
            books,
            chunks[1],
            app_state.lib_data.get_selected_book_state_mut(),
        );
    }

    if render_category_options {
        render_selection_screen(
            &app_state.config,
            chunks[1],
            "Options".into(),
            &mut app_state.lib_data.category_options,
            f,
        );

        if app_state.typing {
            render_type_box(chunks[1], app_state, f, "Enter name:".into())
        }
    }
}
