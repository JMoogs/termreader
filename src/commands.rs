use anyhow::Result;

use crate::appstate::AppState;

pub enum Command {
    NoOp,
    Quit,
    ClearHistory,
}

pub fn parse_command(command: &str) -> Command {
    let command = command.to_lowercase();
    match command.as_str() {
        "quit" | "q" => Command::Quit,
        "clearhistory" => Command::ClearHistory,
        _ => Command::NoOp,
    }
}

pub fn run_command(command: Command, app_state: &mut AppState) -> Result<bool> {
    match command {
        Command::NoOp => Ok(false),
        Command::Quit => Ok(true),
        Command::ClearHistory => {
            app_state.history_data.clear();
            Ok(false)
        }
    }
}
