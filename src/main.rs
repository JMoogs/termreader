use appstate::{AppState, CurrentScreen};
use crossterm::{
    event::{self, Event},
    execute, terminal,
};
use logging::initialize_logging;
use ratatui::prelude::*;

pub mod appstate;
pub mod global;
pub mod helpers;
pub mod local;
pub mod logging;
pub mod reader;
pub mod shutdown;
pub mod startup;
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
            CurrentScreen::Reader => terminal.draw(|f| ui_reader(f, app_state))?,
            _ => terminal.draw(|f| ui_main(f, app_state))?,
        };

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
