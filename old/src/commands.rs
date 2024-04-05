use crate::{
    appstate::AppState,
    state::screen::{LibraryScreen, Screen},
    RequestData,
};
use anyhow::Result;
use std::thread;
use termreader_core::id::ID;
use termreader_sources::sources::Scrape;

pub enum Command {
    NoOp,
    Quit,
    ClearHistory,
    UpdateBook(ID),
    MarkRead((ID, usize)),
}

pub fn parse_command(command: &str, app_state: &AppState) -> Command {
    let command = command.to_lowercase();
    let (cmd, args) = {
        let mut split = command.split_whitespace();
        (split.next().unwrap(), split.collect::<Vec<&str>>())
    };

    match cmd {
        "quit" | "q" => Command::Quit,
        "clearhistory" => Command::ClearHistory,
        "update" => {
            if app_state.current_screen == Screen::Library(LibraryScreen::Default) {
                if let Some(id) = app_state.states.get_selected_book(&app_state.context) {
                    return Command::UpdateBook(id);
                }
            }
            Command::NoOp
        }
        "markread" => {
            if let Some(ch) = args.get(0) {
                if let Ok(ch_num) = ch.parse() {
                    if let Some(id) = app_state.states.get_selected_book(&app_state.context) {
                        return Command::MarkRead((id, ch_num));
                    }
                }
            }
            Command::NoOp
        }
        _ => Command::NoOp,
    }
}

pub fn run_command(command: Command, app_state: &mut AppState) -> Result<bool> {
    match command {
        Command::NoOp => Ok(false),
        Command::Quit => Ok(true),
        Command::ClearHistory => {
            app_state.context.history.clear();
            app_state.buffer.clear();
            Ok(false)
        }
        Command::UpdateBook(id) => {
            let book = app_state.context.library.find_book(id).cloned();
            if book.is_none() {
                return Ok(false);
            }
            let book = book.unwrap();
            let source = app_state
                .context
                .sources
                .get_source_by_id(book.global_get_source_id())
                .unwrap();

            let tx = app_state.channels.get_sender();
            app_state.channels.loading = true;
            thread::spawn(move || {
                book.update(source);
                let _ = tx.send(RequestData::Updated(book));
            });

            Ok(false)
        }
        Command::MarkRead((id, ch)) => {
            let book = app_state.context.library.find_book_mut(id);
            if book.is_none() {
                return Ok(false);
            }
            let book = book.unwrap();
            book.global_mark_ch_read(ch);
            Ok(false)
        }
    }
}
