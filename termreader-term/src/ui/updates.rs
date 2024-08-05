use crate::state::AppState;
use ratatui::{prelude::*, widgets::*};
use termreader_core::Context;

pub(super) fn render_updates(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    let mut display_data: Vec<ListItem> = ctx
        .get_updates()
        .clone()
        .into_iter()
        .map(|e| {
            let chs = e.display_new_chs();
            let book = e.get_book_ref();

            let st = format!("{} | {} | {}", book.get_name(), chs, e.get_timestamp());

            ListItem::new(st).style(app_state.config.unselected_style)
        })
        .collect();

    let entries_len = display_data.len();
    if entries_len == 0 {
        display_data.push(ListItem::new("There are currently no updates"))
    }

    let updates = List::new(display_data)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Updates")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app_state.config.selected_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(
        updates,
        rect,
        app_state.updates_data.get_selected_entry_mut(),
    )
}
