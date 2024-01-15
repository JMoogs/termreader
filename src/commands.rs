use anyhow::Result;

pub enum Command {
    NoOp,
    Quit,
}

pub fn parse_command(command: &str) -> Command {
    let command = command.to_lowercase();
    match command.as_str() {
        "quit" => Command::Quit,
        _ => Command::NoOp,
    }
}

pub fn run_command(command: Command) -> Result<bool> {
    match command {
        Command::NoOp => Ok(false),
        Command::Quit => Ok(true),
    }
}
