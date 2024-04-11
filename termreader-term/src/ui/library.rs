use crate::state::LibScreen;
use crate::state::Screen;
use crate::AppState;
use crate::Context;
use crate::{SELECTED_STYLE, UNSELECTED_STYLE};
use ratatui::{prelude::*, widgets::*};

use super::render_selection_box;
use super::render_selection_screen;
use super::render_type_box;
use super::sources::render_book_view;

/// Renders the library tab
pub(super) fn render_lib(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    let Screen::Lib(libscreen) = app_state.screen else {
        // We'll never be rendering the library screen if we aren't on it
        unreachable!()
    };

    let mut render_global_select = false;
    let mut render_book_v = false;
    let mut render_typing = false;
    let mut render_categories = false;
    let mut render_books = false;
    let mut render_selected_book = false;
    let mut render_category_list = false;
    let mut render_category_options = false;

    match libscreen {
        LibScreen::Main => {
            render_books = true;
            render_categories = true;
        }
        LibScreen::GlobalBookSelect => {
            if app_state.typing {
                render_books = true;
                render_selected_book = true;
                render_categories = true;
                render_typing = true;
            } else {
                render_books = true;
                render_selected_book = true;
                render_categories = true;
                render_global_select = true;
            }
        }
        LibScreen::BookView => {
            render_book_v = true;
        }
        LibScreen::CategorySelect => {
            render_categories = true;
            // Only render books in the rename screen
            if app_state
                .prev_screens
                .last()
                .expect("it should be impossible to start directly into this screen")
                == &Screen::Lib(LibScreen::GlobalBookSelect)
            {
                render_books = true;
                render_selected_book = true;
            } else {
                render_category_list = true;
            }
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
        render_book_view(f, app_state, false);
    }

    if render_typing {
        render_type_box(
            chunks[1],
            app_state,
            f,
            "New name (leave blank for original):".into(),
        )
    }

    if render_global_select {
        render_selection_box(
            chunks[1],
            String::from("Options"),
            &mut app_state.lib_data.global_selected_book_opts,
            f,
        );
    }

    if render_category_list {
        if app_state.typing {
            render_type_box(rect, app_state, f, "New Name:".into())
        } else {
            render_selection_box(
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
            .style(UNSELECTED_STYLE)
            .highlight_style(SELECTED_STYLE)
            .select(app_state.lib_data.get_selected_category());

        f.render_widget(tabs, chunks[0]);
    }

    if render_books {
        let current_category =
            &ctx.lib_get_categories()[app_state.lib_data.get_selected_category()];

        let mut display_data: Vec<ListItem> = if render_selected_book {
            let b = app_state
                .lib_data
                .get_selected_book(ctx)
                .expect("no book was selected although one was expected to be");

            vec![ListItem::new(b.display_info()).style(SELECTED_STYLE)]
        } else {
            ctx.lib_get_books()
                .get(current_category)
                .unwrap()
                .iter()
                .map(|b| ListItem::new(b.display_info()).style(UNSELECTED_STYLE))
                .collect()
        };

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

        if render_selected_book {
            f.render_stateful_widget(
                books,
                chunks[1],
                &mut ListState::default().with_selected(Some(0)),
            );
        } else {
            f.render_stateful_widget(
                books,
                chunks[1],
                app_state.lib_data.get_selected_book_state_mut(),
            );
        }
    }

    if render_category_options {
        render_selection_screen(
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
