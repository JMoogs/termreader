use crate::{
    appstate::AppState,
    global::sources::Scrape,
    state::{
        book_info::{BookSource, ID},
        screen::{CurrentScreen, LibraryOptions},
    },
    RequestData,
};
use anyhow::Result;
use std::thread;

pub enum Command {
    NoOp,
    Quit,
    ClearHistory,
    UpdateBook(ID),
}

pub fn parse_command(command: &str, app_state: &AppState) -> Command {
    let command = command.to_lowercase();
    match command.as_str() {
        "quit" | "q" => Command::Quit,
        "clearhistory" => Command::ClearHistory,
        "update" => {
            if app_state.current_screen == CurrentScreen::Library(LibraryOptions::Default) {
                if let Some(b) = app_state.library_data.get_category_list().selected() {
                    return Command::UpdateBook(b.id);
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
            app_state.history_data.clear();
            Ok(false)
        }
        Command::UpdateBook(id) => {
            let book = app_state.library_data.find_book_mut(id);
            if book.is_none() {
                return Ok(false);
            }
            let book = book.unwrap();
            if let BookSource::Global(data) = &book.source_data {
                let source_id = data.novel.source;
                let source = app_state.source_data.get_source_by_id(source_id).clone();
                let tx = app_state.channels.get_sender();
                let path = data.novel.novel_url.clone();

                app_state.channels.loading = true;
                thread::spawn(move || {
                    let novel = source.parse_novel_and_chapters(path);
                    let _ = tx.send(RequestData::UpdateInfo((id, novel)));
                });
            }
            Ok(false)
        }
    }
}
