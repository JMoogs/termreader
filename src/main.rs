// Temporary
#![allow(dead_code)]
#![allow(unused_variables)]

use appstate::{AppState, CurrentScreen};
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use logging::initialize_logging;
use ratatui::prelude::*;

pub mod appstate;
pub mod helpers;
pub mod local;
pub mod logging;
pub mod reader;
pub mod shutdown;
pub mod startup;
pub mod ui;

use ui::{main::ui_main, reader::ui_reader};

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

    // TESTING:
    // shutdown::store_books(&shutdown::generate_lib_info())?;

    let mut app_state = AppState::build()?;

    let res = run_app(&mut terminal, &mut app_state);

    shutdown::store_books(&app_state.library_data.clone().into())?;

    // Restore terminal on ending:
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

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
            CurrentScreen::Main => terminal.draw(|f| ui_main(f, app_state))?,
            CurrentScreen::Reader => terminal.draw(|f| ui_reader(f, app_state))?,
            CurrentScreen::ExitingReader => todo!(),
        };

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // We only care about key presses
                // Note: This should only matter with windows as other OSes don't have key release events in crossterm
                continue;
            }
            match app_state.current_screen {
                CurrentScreen::Main => match key.code {
                    event::KeyCode::Esc | event::KeyCode::Char('q') => break,
                    event::KeyCode::Char(']') => {
                        app_state.current_main_tab.next();
                    }
                    event::KeyCode::Char('[') => {
                        app_state.current_main_tab.previous();
                    }
                    event::KeyCode::Char('}') => {
                        if app_state.current_main_tab.in_library() {
                            app_state.library_data.categories.next();
                        }
                    }
                    event::KeyCode::Char('{') => {
                        if app_state.current_main_tab.in_library() {
                            app_state.library_data.categories.previous();
                        }
                    }
                    event::KeyCode::Up => {
                        if app_state.current_main_tab.in_library() {
                            app_state.library_data.get_category_list_mut().previous();
                        }
                    }
                    event::KeyCode::Down => {
                        if app_state.current_main_tab.in_library() {
                            app_state.library_data.get_category_list_mut().next();
                        }
                    }
                    event::KeyCode::Enter | event::KeyCode::Char(' ') => {
                        if app_state.current_main_tab.in_library() {
                            let idx = app_state
                                .library_data
                                .get_category_list()
                                .state
                                .selected()
                                .unwrap();
                            let book =
                                app_state.library_data.get_category_list().items[idx].clone();

                            app_state.current_screen = CurrentScreen::Reader;
                            app_state.update_reader(book)?;
                        }
                    }
                    _ => (),
                },
                CurrentScreen::Reader => match key.code {
                    event::KeyCode::Esc | event::KeyCode::Char('q') => {
                        app_state.update_lib_from_reader()?;
                        // shutdown::store_books(&app_state.library_data.clone().into())?;
                        app_state.current_screen = CurrentScreen::Main;
                    }
                    event::KeyCode::Down => {
                        app_state.reader_data.as_mut().unwrap().scroll_down(1);
                    }
                    event::KeyCode::Up => {
                        app_state.reader_data.as_mut().unwrap().scroll_up(1);
                    }
                    event::KeyCode::Right => {
                        trace_dbg!(app_state
                            .reader_data
                            .as_mut()
                            .unwrap()
                            .portion
                            .display_line_idxs
                            .clone());
                    }
                    _ => (),
                },
                CurrentScreen::ExitingReader => {}
            }
        }
    }

    Ok(())
}
