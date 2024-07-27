use crate::helpers::to_datetime;
use crate::state::HistoryScreen;
use crate::state::Screen;
use crate::AppState;
use crate::Context;
use ratatui::{prelude::*, widgets::*};

use super::sources::render_book_view;
use super::sources::BookViewOption;

pub(super) fn render_history(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    let Screen::History(historyscreen) = app_state.screen else {
        // We'll never be rendering the history screen if we aren't on it
        unreachable!()
    };

    let mut render_book_v = false;
    let mut render_books = false;

    match historyscreen {
        HistoryScreen::Main => {
            render_books = true;
        }
        HistoryScreen::BookView => {
            render_book_v = true;
        } // HistoryScreen::LocalBookOptions => {
          //     render_books = true;
          //     render_selected_book = true;
          //     render_local_options = true;
          // }
          // HistoryScreen::GlobalBookOptions => {
          //     render_books = true;
          //     render_selected_book = true;
          //     render_global_options = true;
          // }
    }

    if render_book_v {
        render_book_view(f, app_state, ctx, BookViewOption::HistoryOptions)
    }

    if render_books {
        let mut display_data: Vec<ListItem> = {
            ctx.hist_get()
                .clone()
                .into_iter()
                .map(|e| {
                    let t = if e.get_chapter() == 0 {
                        format!("{} | {}", e.get_book_name(), to_datetime(e.get_timestamp()))
                    } else {
                        format!(
                            "{} | Chapter {} | {}",
                            e.get_book_name(),
                            e.get_chapter(),
                            to_datetime(e.get_timestamp())
                        )
                    };
                    ListItem::new(t).style(app_state.config.unselected_style)
                })
                .collect()
        };

        let entries_len = display_data.len();
        if entries_len == 0 {
            display_data.push(ListItem::new("You currently have no history."))
        }

        let history = List::new(display_data)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("History")
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(app_state.config.selected_style)
            .highlight_symbol("> ");

        f.render_stateful_widget(
            history,
            rect,
            app_state.history_data.get_selected_entry_mut(),
        );
    }
}
