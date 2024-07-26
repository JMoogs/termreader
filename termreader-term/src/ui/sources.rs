use crate::state::sources::SourceNovelPreviewSelection;
use crate::state::LibScreen;
use crate::ui::render_selection_box;
use crate::ui::render_type_box;
use crate::AppState;
use crate::Context;
use crate::Screen;
use crate::SourceScreen;
use ratatui::{prelude::*, widgets::*};

use super::render_selection_screen;

pub(super) fn render_sources(rect: Rect, ctx: &Context, app_state: &mut AppState, f: &mut Frame) {
    if let Screen::Sources(s) = app_state.screen {
        match s {
            SourceScreen::Main | SourceScreen::Select => {
                let display_data: Vec<ListItem> = ctx
                    .source_get_info()
                    .into_iter()
                    .map(|(_, name)| ListItem::new(name).style(app_state.config.unselected_style))
                    .collect();

                let sources = List::new(display_data)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Sources")
                            .border_type(BorderType::Rounded),
                    )
                    .highlight_style(app_state.config.selected_style)
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
                            &app_state.config,
                            rect,
                            String::from("Options"),
                            &mut app_state.source_data.source_options,
                            f,
                        );
                    }
                }
            }
            SourceScreen::SearchRes => render_search_results(rect, app_state, f),
            SourceScreen::BookView => render_book_view(f, app_state, BookViewOption::SourceOptions),
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
        .map(|f| ListItem::new(f.get_name().to_string()).style(app_state.config.unselected_style))
        .collect();

    let results = List::new(display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results:")
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app_state.config.selected_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(results, rect, app_state.buffer.novel_search_res.state_mut());
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum BookViewOption {
    #[default]
    None,
    LibOptions,
    SourceOptions,
}

pub fn render_book_view(f: &mut Frame, app_state: &mut AppState, option_type: BookViewOption) {
    // We want to display:
    // 1) The title across the top
    // 2) The synopsis - left
    // 3) Options - middle right (conditional on `show_options`)
    //   - In certain cases, we may want to replace the options screen with something else
    // 4) The chapter list - bottom right
    // We also need to display the bar at the bottom

    let chunks_vert_1 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // The title bar
            Constraint::Min(1),    // The rest of the contents
            Constraint::Length(3), // Left empty for the bottom bar
        ])
        .split(f.size());

    let chunks_horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // The synopsis
            Constraint::Percentage(40), // Chapters and options
        ])
        .split(chunks_vert_1[1]);

    // Only use this split if we're displaying options
    let chunks_vert_2 = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Options
            Constraint::Percentage(70), // Chapters
        ])
        .split(chunks_horiz[1]);

    let novel = app_state.buffer.novel.as_ref().unwrap();

    // Title
    let title = Paragraph::new(novel.get_alias_or_name())
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
        .style(app_state.config.unselected_style);

    let block = if app_state.source_data.novel_preview_selected_field
        == SourceNovelPreviewSelection::Summary
    {
        block.style(app_state.config.selected_style)
    } else {
        block
    };

    let synopsis = Paragraph::new(novel.get_synopsis())
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app_state.buffer.novel_preview_scroll as u16, 0));

    f.render_widget(synopsis, chunks_horiz[0]);

    // Options

    if option_type != BookViewOption::None {
        if app_state.typing {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("New Name (leave blank to reset):")
                .border_type(BorderType::Rounded)
                .style(app_state.config.selected_style); // Always the selected box if typing

            let mut text = app_state.buffer.text.clone();
            text.push('_');

            let display = Paragraph::new(text).wrap(Wrap { trim: false }).block(block);

            f.render_widget(display, chunks_vert_2[0]);
        } else if matches!(app_state.screen, Screen::Lib(LibScreen::BookViewCategory)) {
            render_selection_screen(
                &app_state.config,
                chunks_vert_2[0],
                String::from("Pick category:"),
                &mut app_state.buffer.temporary_list,
                f,
            );
        } else {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .border_type(BorderType::Rounded)
                .style(app_state.config.unselected_style);

            // let options = app_state.source_data.novel_options.clone();
            let options = match option_type {
                BookViewOption::LibOptions => app_state.lib_data.global_selected_book_opts.clone(),
                BookViewOption::SourceOptions => app_state.source_data.novel_options.clone(),
                _ => unreachable!(),
            };

            let list: Vec<ListItem> = Vec::from(options)
                .into_iter()
                .map(|i| ListItem::new(i).style(app_state.config.unselected_style))
                .collect();

            let block = if app_state.source_data.novel_preview_selected_field
                == SourceNovelPreviewSelection::Options
            {
                block.style(app_state.config.selected_style)
            } else {
                block
            };

            let display = List::new(list)
                .block(block)
                .highlight_style(app_state.config.selected_style)
                .highlight_symbol("> ");

            let state = match option_type {
                BookViewOption::LibOptions => {
                    app_state.lib_data.global_selected_book_opts.state_mut()
                }
                BookViewOption::SourceOptions => app_state.source_data.novel_options.state_mut(),
                _ => unreachable!(),
            };

            f.render_stateful_widget(display, chunks_vert_2[0], state);
        }
    }

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
                .style(app_state.config.unselected_style)
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
        block.style(app_state.config.selected_style)
    } else {
        block
    };

    let display = List::new(list)
        .block(block)
        .highlight_style(app_state.config.selected_style)
        .highlight_symbol("> ");

    if option_type != BookViewOption::None {
        f.render_stateful_widget(
            display,
            chunks_vert_2[1],
            app_state.buffer.chapter_previews.state_mut(),
        );
    } else {
        f.render_stateful_widget(
            display,
            chunks_horiz[1],
            app_state.buffer.chapter_previews.state_mut(),
        );
    }
}
