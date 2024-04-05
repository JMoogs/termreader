use crate::helpers::to_datetime;
use crate::state::HistoryScreen;
use crate::state::Screen;
use crate::AppState;
use crate::Context;
use crate::{SELECTED_STYLE, UNSELECTED_STYLE};
use ratatui::{prelude::*, widgets::*};

use super::render_selection_box;

pub(super) fn render_history(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    let mut display_data: Vec<ListItem> = ctx
        .hist_get()
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
            ListItem::new(t).style(UNSELECTED_STYLE)
        })
        .collect();
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
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(
        history,
        rect,
        app_state.history_data.get_selected_entry_mut(),
    );

    if let Screen::History(HistoryScreen::LocalBookOptions) = app_state.screen {
        render_selection_box(
            rect,
            String::from("Options"),
            &mut app_state.history_data.local_book_options,
            f,
        );
    } else if let Screen::History(HistoryScreen::GlobalBookOptions) = app_state.screen {
        render_selection_box(
            rect,
            String::from("Options"),
            &mut app_state.history_data.global_book_options,
            f,
        );
    }
}
