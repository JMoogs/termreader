use crate::{
    appstate::AppState, global::sources::source_data::SourceBookBox,
    ui::helpers::centered_sized_rect, SELECTED_STYLE, UNSELECTED_STYLE,
};
use ratatui::{prelude::*, widgets::*};

pub fn render_source_selection(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let options = app_state.menu_options.source_options.items.clone();

    let mut max_width = 0;
    for s in options.iter() {
        if s.len() > max_width {
            max_width = s.len();
        }
    }
    max_width += 6;

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
    f.render_stateful_widget(display, r, &mut app_state.menu_options.source_options.state)
}

pub fn render_search_results(
    rect: Rect,
    app_state: &mut AppState,
    f: &mut Frame,
    block_text: &str,
) {
    let display: Vec<ListItem> = app_state
        .buffer
        .novel_previews
        .items
        .iter()
        .map(|f| ListItem::new(f.name.clone()).style(UNSELECTED_STYLE))
        .collect();

    let results = List::new(display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(block_text)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(results, rect, &mut app_state.buffer.novel_previews.state);
}

pub fn render_source_book(f: &mut Frame, app_state: &mut AppState) {
    // We want to display:
    // 1) The title across the top
    // 2) The synopsis - left
    // 3) Options - middle right
    // 4) The chapter list - bottom right

    let chunks_vert_1 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    let chunks_horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks_vert_1[1]);

    let chunks_vert_2 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks_horiz[1]);

    let novel = app_state.buffer.novel.as_ref().unwrap();

    // The title
    let title = Paragraph::new(novel.name.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(title, chunks_vert_1[0]);

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
        Paragraph::new(novel.synopsis())
            .block(selected_block)
            .wrap(Wrap { trim: true })
            .scroll((app_state.buffer.novel_preview_scroll as u16, 0))
    } else {
        Paragraph::new(novel.synopsis())
            .block(unselected_block)
            .wrap(Wrap { trim: true })
            .scroll((app_state.buffer.novel_preview_scroll as u16, 0))
    };

    f.render_widget(synopsis, chunks_horiz[0]);

    // Options

    let unselected_block = Block::default()
        .borders(Borders::ALL)
        .title("Options")
        .border_type(BorderType::Rounded);

    let selected_block = Block::default()
        .borders(Borders::ALL)
        .title("Options")
        .border_type(BorderType::Rounded)
        .style(SELECTED_STYLE);
    // app_state.menu_options.source_book_options
    let options = app_state.menu_options.source_book_options.items.clone();
    let list: Vec<ListItem> = options
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let display = if app_state.buffer.novel_preview_selection == SourceBookBox::Options {
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
        chunks_vert_2[0],
        &mut app_state.menu_options.source_book_options.state,
    );

    // Chapters

    let chapters = app_state.buffer.novel.clone().unwrap().chapters;

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
        chunks_vert_2[1],
        &mut app_state.buffer.chapter_previews.state,
    );
}
