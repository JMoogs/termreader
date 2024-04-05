#![allow(dead_code, unused_imports, unused_variables)]
pub mod appstate;
pub mod controls;
pub mod helpers;
pub mod logging;
pub mod reader;
pub mod state;
pub mod ui;

use crate::controls::handle_controls;
use crate::logging::initialize_logging;
use crate::state::channels::RequestData;
use crate::state::AppState;
use crate::state::Screen;
use crate::ui::ui_main;
use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use helpers::StatefulList;
use ratatui::prelude::*;
use state::sources::SourceNovelPreviewSelection;
use state::SourceScreen;
use termreader_core::Context;
use ui::reader::ui_reader;

pub const UNSELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::White);
pub const SELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::Green);

fn main() -> Result<()> {
    // Start logging
    initialize_logging()?;
    // Set up the terminal
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load data, run the app, and save data
    let mut ctx = Context::build()?;
    let mut app_state = AppState::build(&ctx);
    let res = run_app(&mut terminal, &mut ctx, &mut app_state);

    ctx.save()?;

    // Restore the terminal to its initial state
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Errors must be printed after the terminal is fixed, or they won't show (properly).
    if let Err(e) = res {
        println!("An error occured: {:?}", e);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    ctx: &mut Context,
    app_state: &mut AppState,
) -> Result<()> {
    loop {
        if app_state.quit {
            break;
        }
        match app_state.screen {
            Screen::Reader => terminal.draw(|f| ui_reader(f, ctx, app_state))?,
            _ => terminal.draw(|f| ui_main(f, ctx, app_state))?,
        };

        if app_state.channel.loading {
            if let Ok(data) = app_state.channel.reciever.recv() {
                match data {
                    RequestData::SearchResults(res) => {
                        // app_state.buffer.clear();
                        app_state.buffer.novel_search_res = StatefulList::from(res?);
                        app_state.update_screen(Screen::Sources(SourceScreen::SearchRes));
                    }
                    RequestData::BookInfo((res, opt)) => {
                        // app_state.buffer.clear();
                        let res = res?;

                        if opt {
                            app_state.source_data.novel_preview_selected_field =
                                SourceNovelPreviewSelection::Options;
                        }
                        app_state.buffer.chapter_previews =
                            StatefulList::from(res.get_chapters().clone());
                        app_state.buffer.novel = Some(res);

                        if opt {
                            app_state.update_screen(Screen::Sources(SourceScreen::BookView));
                        } else {
                            todo!()
                        }
                    }
                    RequestData::Chapter((id, res, ch)) => {
                        let book = ctx.lib_find_book_mut(id);
                        let mut book = if let Some(b) = book {
                            b.clone()
                        } else {
                            app_state.buffer.book.clone().unwrap()
                        };

                        book.global_set_ch(ch);
                        app_state.move_to_reader(book, Some(res?));
                    }
                }
                app_state.channel.loading = false;
                // `event::read()` is blocking so continue to redraw after
                continue;
            }
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // We only care about key presses
                continue;
            }
            handle_controls(ctx, app_state, key.code);
        }
    }
    Ok(())
}
