// #![allow(dead_code, unused_imports, unused_variables)]
pub mod controls;
pub mod helpers;
pub mod logging;
pub mod reader;
pub mod setup;
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
use logging::get_data_dir;
use ratatui::prelude::*;
use setup::enter_book_view;
use setup::BookViewType;
use state::channels::BookInfo;
use state::channels::BookInfoDetails;
use state::SourceScreen;
use termreader_core::book::Book;
use termreader_core::Context;
use ui::reader::ui_reader;

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
    let project_dir = get_data_dir();
    let mut ctx = Context::build(project_dir)?;
    let mut app_state = AppState::build(&ctx);
    let res = run_app(&mut terminal, &mut ctx, &mut app_state);

    app_state.config.save(&ctx.get_save_dir())?;
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
            Screen::Reader => terminal.draw(|f| ui_reader(f, app_state))?,
            _ => terminal.draw(|f| ui_main(f, ctx, app_state))?,
        };

        if app_state.channel.loading {
            if let Ok(data) = app_state.channel.reciever.recv() {
                match data {
                    RequestData::SearchResults(res) => {
                        app_state.buffer.novel_search_res = StatefulList::from(res?);
                        app_state.update_screen(Screen::Sources(SourceScreen::SearchRes));
                    }
                    RequestData::BookInfo((res, info)) => {
                        let novel = res?;
                        let book = match ctx.get_book_url(novel.get_full_url().to_string()) {
                            Some(b) => b,
                            None => {
                                let book = Book::from_novel(novel);
                                let book_id = book.get_id();
                                ctx.add_book(book);
                                ctx.get_book(book_id).expect("we just added the book")
                            }
                        };

                        match info {
                            BookInfoDetails::SourceWithOptions => {
                                enter_book_view(app_state, ctx, book, BookViewType::Source);
                            }
                            BookInfoDetails::HistoryWithOptions => {
                                enter_book_view(app_state, ctx, book, BookViewType::History);
                            }
                            _ => unreachable!(),
                        }
                    }
                    RequestData::Chapter((book_info, res, ch)) => match book_info {
                        BookInfo::NewBook(_b) => {
                            unreachable!()
                            // b.global_set_ch(ch);
                            // app_state.move_to_reader(b, Some(res?));
                        }
                        BookInfo::ID(id) => {
                            let book = ctx.get_book(id);
                            match book {
                                    Some(mut b) => {
                                        b.global_set_chapter(ch).unwrap();
                                        app_state.move_to_reader(b.clone(), Some(res?) );
                                    },
                                    None => panic!("Book existed so we returned an ID, but we were unable to find it?"),
                                }
                        }
                    },
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
