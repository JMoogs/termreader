use crate::state::sources::SourceNovelPreviewSelection;
use crate::ui::render_selection_box;
use crate::ui::render_type_box;
use crate::AppState;
use crate::Context;
use crate::Screen;
use crate::SourceScreen;
use crate::{SELECTED_STYLE, UNSELECTED_STYLE};
use ratatui::{prelude::*, widgets::*};

pub(super) fn render_sources(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    if let Screen::Sources(s) = app_state.screen {
        match s {
            SourceScreen::Main | SourceScreen::Select => {
                let display_data: Vec<ListItem> = ctx
                    .source_get_info()
                    .into_iter()
                    .map(|(_, name)| ListItem::new(name).style(UNSELECTED_STYLE))
                    .collect();
                let source_len = display_data.len();

                let sources = List::new(display_data)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Sources")
                            .border_type(BorderType::Rounded),
                    )
                    .highlight_style(SELECTED_STYLE)
                    .highlight_symbol("> ");
                f.render_stateful_widget(
                    sources,
                    rect,
                    app_state.source_data.get_selected_source_state_mut(),
                );

                if let Screen::Sources(SourceScreen::Select) = app_state.screen {
                    if app_state.typing {
                        render_type_box(rect, app_state, f, "Search:".into());
                    } else {
                        // render_selection_box(rect, app_state, f);
                        render_selection_box(
                            rect,
                            String::from("Options"),
                            &mut app_state.source_data.source_options,
                            f,
                        );
                    }
                }
            }
            SourceScreen::SearchRes => render_search_results(rect, app_state, f),
            SourceScreen::BookView => render_book_view(f, app_state),
        }
    }
}

/// A function to render the results of searching a source, or viewing the popular books
fn render_search_results(rect: Rect, app_state: &mut AppState, f: &mut Frame) {
    let display: Vec<ListItem> = app_state
        .buffer
        .novel_search_res
        .items
        .clone()
        .into_iter()
        .map(|f| ListItem::new(f.get_name().to_string()).style(UNSELECTED_STYLE))
        .collect();

    let results = List::new(display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results:")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(results, rect, app_state.buffer.novel_search_res.state_mut());
}

fn render_book_view(f: &mut Frame, app_state: &mut AppState) {
    // TODO: Add support for displaying without the options for viewing a book
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

    // Title
    let title = Paragraph::new(novel.get_name())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(title, chunks_vert_1[0]);

    // Synopsis
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(UNSELECTED_STYLE);

    let block = if app_state.source_data.novel_preview_selected_field
        == SourceNovelPreviewSelection::Summary
    {
        block.style(SELECTED_STYLE)
    } else {
        block
    };

    let synopsis = Paragraph::new(novel.get_synopsis())
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app_state.buffer.novel_preview_scroll as u16, 0));

    f.render_widget(synopsis, chunks_horiz[0]);

    // Options
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Options")
        .border_type(BorderType::Rounded)
        .style(UNSELECTED_STYLE);

    let options = app_state.source_data.novel_options.clone();
    let list: Vec<ListItem> = Vec::from(options)
        .into_iter()
        .map(|i| ListItem::new(i).style(UNSELECTED_STYLE))
        .collect();

    let block = if app_state.source_data.novel_preview_selected_field
        == SourceNovelPreviewSelection::Options
    {
        block.style(SELECTED_STYLE)
    } else {
        block
    };

    let display = List::new(list)
        .block(block)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(
        display,
        chunks_vert_2[0],
        app_state.source_data.novel_options.state_mut(),
    );

    // Chapters
    let chapters = app_state
        .buffer
        .novel
        .as_ref()
        .unwrap()
        .get_chapters()
        .clone();

    let list: Vec<ListItem> = chapters
        .into_iter()
        .map(|i| {
            ListItem::new(format!("{}: {}", i.get_chapter_no(), i.get_name()))
                .style(UNSELECTED_STYLE)
        })
        .collect();
    let ch_count = list.len();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Chapters ({} total)", ch_count))
        .border_type(BorderType::Rounded);

    let block = if app_state.source_data.novel_preview_selected_field
        == SourceNovelPreviewSelection::Chapters
    {
        block.style(SELECTED_STYLE)
    } else {
        block
    };

    let display = List::new(list)
        .block(block)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol("> ");

    f.render_stateful_widget(
        display,
        chunks_vert_2[1],
        app_state.buffer.chapter_previews.state_mut(),
    );
}
