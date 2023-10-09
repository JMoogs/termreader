use appstate::{AppState, CurrentScreen};
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use ratatui::prelude::*;

pub mod appstate;
pub mod ui;

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

    let mut app_state = AppState::build();
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
                continue;
            }
            match app_state.current_screen {
                CurrentScreen::Main => match key.code {
                    event::KeyCode::Esc | event::KeyCode::Char('q') => break,
                    event::KeyCode::Char(']') => {
                        app_state.current_tab.next();
                    }
                    event::KeyCode::Char('[') => {
                        app_state.current_tab.previous();
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
