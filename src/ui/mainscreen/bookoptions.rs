use ratatui::{prelude::*, widgets::*};

use crate::{
    appstate::AppState, global::sources::source_data::SourceBookBox,
    ui::helpers::centered_sized_rect, SELECTED_STYLE, UNSELECTED_STYLE,
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
    let mut text = app_state.buffer.text.clone();
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

pub fn render_ch_list(app_state: &mut AppState, f: &mut Frame) {
    // Same as source book, but without the options, i.e. a title,
    // the synopsis on the left, and the chapters on the right.
    let chunks_vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    let chunks_horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks_vert[1]);

    let novel = app_state.buffer.novel.as_ref().unwrap();

    let synopsis = novel.synopsis();

    // The title
    let title = Paragraph::new(novel.name.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(title, chunks_vert[0]);

    let unselected_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(UNSELECTED_STYLE);
    let selected_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(SELECTED_STYLE);
    // Synopsis
    let synopsis = if app_state.buffer.novel_preview_selection == SourceBookBox::Summary {
        Paragraph::new(synopsis)
            .block(selected_block)
            .wrap(Wrap { trim: true })
            .scroll((app_state.buffer.novel_preview_scroll as u16, 0))
    } else {
        app_state.buffer.novel_preview_selection = SourceBookBox::Chapters;
        Paragraph::new(synopsis)
            .block(unselected_block)
            .wrap(Wrap { trim: true })
            .scroll((app_state.buffer.novel_preview_scroll as u16, 0))
    };

    f.render_widget(synopsis, chunks_horiz[0]);

    let chapters = app_state.buffer.chapter_previews.items.clone();

    let list: Vec<ListItem> = chapters
        .into_iter()
        .map(|i| ListItem::new(format!("{}: {}", i.chapter_no, i.name)).style(UNSELECTED_STYLE))
        .collect();

    let ch_count = list.len();

    let unselected_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Chapters ({} total)", ch_count))
        .border_type(BorderType::Rounded);

    let selected_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Chapters ({} total)", ch_count))
        .border_type(BorderType::Rounded)
        .style(SELECTED_STYLE);

    let display = if app_state.buffer.novel_preview_selection == SourceBookBox::Chapters {
        List::new(list)
            .block(selected_block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
    } else {
        List::new(list)
            .block(unselected_block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
    };

    f.render_stateful_widget(
        display,
        chunks_horiz[1],
        &mut app_state.buffer.chapter_previews.state,
    )
}

pub fn render_global_selection_history(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let options = app_state.menu_options.global_history_options.items.clone();

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
    f.render_stateful_widget(
        display,
        r,
        &mut app_state.menu_options.global_history_options.state,
    )
}

pub fn render_local_selection_history(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let options = app_state.menu_options.local_history_options.items.clone();

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
    f.render_stateful_widget(
        display,
        r,
        &mut app_state.menu_options.local_history_options.state,
    )
}
