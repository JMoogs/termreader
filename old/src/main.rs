#![allow(dead_code, unused_imports)]
use crate::{
    appstate::AppState,
    helpers::StatefulList,
    state::{
        channels::RequestData,
        screen::{MiscOptions, Screen, SourceOptions},
    },
};
use appstate::NovelPreviewSelection;
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use ratatui::prelude::*;
use termreader_core::{logging::initialize_logging, Context};

pub mod appstate;
pub mod commands;
pub mod helpers;
pub mod local;
pub mod reader;
pub mod state;
pub mod ui;

use ui::{controls::handle_controls, mainscreen::main::ui_main, reader::ui_reader};

const UNSELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::White);
const SELECTED_STYLE: ratatui::style::Style = Style::new().fg(Color::Green);

fn main() -> Result<(), anyhow::Error> {
    // Set up logging
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

    let mut app_state = AppState::build()?;

    let res = run_app(&mut terminal, &mut app_state);

    // Store data on shutdown
    app_state.save();

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
    app_state: &mut AppState,
) -> Result<(), anyhow::Error> {
    loop {
        match app_state.current_screen {
            Screen::Reader => terminal.draw(|f| ui_reader(f, app_state))?,
            _ => terminal.draw(|f| ui_main(f, app_state))?,
        };

        if app_state.channels.loading {
            if let Ok(data) = app_state.channels.reciever.recv() {
                match data {
                    RequestData::SearchResults(res) => {
                        app_state.buffer.novel_previews = StatefulList::from(res?);
                        app_state.update_screen(Screen::Sources(SourceOptions::SearchResults));
                    }
                    RequestData::BookInfo((res, opt)) => {
                        let res = res?;
                        app_state.buffer.clear();
                        if opt {
                            app_state.buffer.novel_preview_selection =
                                NovelPreviewSelection::Options
                        }

                        app_state.buffer.chapter_previews =
                            StatefulList::from(res.get_chapters().clone());
                        app_state.buffer.novel = Some(res);

                        if opt {
                            app_state.update_screen(Screen::Sources(SourceOptions::BookView));
                        } else {
                            app_state.update_screen(Screen::Misc(MiscOptions::ChapterView));
                        }
                    }
                    RequestData::Updated(book) => {
                        // let b = app_state.library_data.find_book_mut(id).unwrap();
                        let b = app_state
                            .context
                            .library
                            .find_book_mut(book.get_id())
                            .unwrap();
                        std::mem::replace(b, book);
                    }
                    RequestData::Chapter((id, res, ch_no)) => {
                        let mut book = app_state.context.library.find_book_mut(id).unwrap();
                        book.global_set_ch(ch_no);
                        app_state.move_to_reader(
                            BookInfo::Library(book),
                            Some(ch_no),
                            Some(res?),
                        )?;
                    }
                }
                // `event::read()` is blocking so continue to redraw after
                app_state.channels.loading = false;
                continue;
            }
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // We only care about key presses
                // Note: This should only matter with windows as other OSes don't have key release events in crossterm
                continue;
            }
            if handle_controls(app_state, key.code)? {
                break;
            }
        }
    }

    Ok(())
}
