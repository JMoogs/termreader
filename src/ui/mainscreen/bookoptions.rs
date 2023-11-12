use ratatui::{prelude::*, widgets::*};

use crate::{
    appstate::AppState, ui::helpers::centered_sized_rect, SELECTED_STYLE, UNSELECTED_STYLE,
};

pub fn render_local_selection(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let options = app_state.menu_options.local_options.items.clone();

    let mut max_width = 0;
    for s in options.iter() {
        if s.len() > max_width {
            max_width = s.len();
        }
    }

    max_width += 6; // + 2 for box, + 4 to make it look better

    let list: Vec<ListItem> = options
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let height = list.len() + 2;

    let display = List::new(list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    let r = centered_sized_rect(max_width as u16, height as u16, rect);
    f.render_stateful_widget(display, r, &mut app_state.menu_options.local_options.state)
}

pub fn render_global_selection(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let options = app_state.menu_options.global_options.items.clone();

    let mut max_width = 0;
    for s in options.iter() {
        if s.len() > max_width {
            max_width = s.len();
        }
    }

    max_width += 6; // + 2 for box, + 4 to make it look better

    let list: Vec<ListItem> = options
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let height = list.len() + 2;

    let display = List::new(list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    let r = centered_sized_rect(max_width as u16, height as u16, rect);
    f.render_stateful_widget(display, r, &mut app_state.menu_options.global_options.state)
}

pub fn render_type_box(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let mut text = app_state.text_buffer.clone();
    text.push('_');

    let display = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input Text:")
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: false });

    let r = centered_sized_rect(40, 6, rect);

    f.render_widget(display, r)
}

pub fn render_mv_category_box(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let options = app_state.library_data.categories.tabs.clone();

    let mut max_width = 0;
    for s in options.iter() {
        if s.len() > max_width {
            max_width = s.len();
        }
    }

    max_width += 6; // + 2 for box, + 4 to make it look better

    let list: Vec<ListItem> = options
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let height = list.len() + 2; // + 2 to add the box

    let display = List::new(list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Categories")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    let r = centered_sized_rect(max_width as u16, height as u16, rect);
    // let r = centered_rect(20, 20, rect);
    f.render_stateful_widget(display, r, &mut app_state.menu_options.category_moves.state)
}
