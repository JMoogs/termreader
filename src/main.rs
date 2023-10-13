// Temporary
#![allow(dead_code)]
#![allow(unused_variables)]

use appstate::{AppState, CurrentScreen};
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use ratatui::prelude::*;

pub mod appstate;
pub mod helpers;
pub mod shutdown;
pub mod startup;
pub mod ui;

use shutdown::generate_lib_info;
use ui::main::ui_main;

fn main() -> Result<(), anyhow::Error> {
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
    shutdown::store_books(&generate_lib_info())?;

    let mut app_state = AppState::build()?;

    let res = run_app(&mut terminal, &mut app_state);

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
            CurrentScreen::Reader => todo!(),
            CurrentScreen::ExitingReader => todo!(),
        };
        // terminal.draw(|f| ui(f, app_state))?;
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
                        app_state.library_data.categories.next();
                    }
                    event::KeyCode::Char('{') => {
                        app_state.library_data.categories.previous();
                    }
                    event::KeyCode::Up => {
                        app_state.library_data.get_category_list_mut().previous();
                    }
                    event::KeyCode::Down => {
                        app_state.library_data.get_category_list_mut().next();
                    }
                    _ => (),
                },
                CurrentScreen::Reader => {}
                CurrentScreen::ExitingReader => {}
            }
        }
    }

    Ok(())
}
