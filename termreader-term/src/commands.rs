use crate::{
    appstate::AppState,
    state::{
        book_info::{BookSource, ID},
        screen::{CurrentScreen, LibraryOptions},
    },
    RequestData,
};
use anyhow::Result;
use std::thread;
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
            if app_state.current_screen == CurrentScreen::Library(LibraryOptions::Default) {
                if let Some(b) = app_state.library_data.get_category_list().selected() {
                    return Command::UpdateBook(b.id);
                }
            }
            Command::NoOp
        }
        "markread" => {
            if let Some(ch) = args.get(0) {
                if let Ok(ch_num) = ch.parse() {
                    if let Some(book) = app_state.library_data.get_category_list().selected() {
                        return Command::MarkRead((book.id, ch_num));
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
            app_state.history_data.clear();
            Ok(false)
        }
        Command::UpdateBook(id) => {
            let book = app_state.library_data.find_book_mut(id).cloned();
            if book.is_none() {
                return Ok(false);
            }
            let book = book.unwrap();
            if let BookSource::Global(data) = &book.source_data {
                let source_id = data.novel.get_source();
                let source = app_state.get_source_by_id(source_id).cloned().unwrap();
                let tx = app_state.channels.get_sender();
                let path = data.novel.get_url().to_string();

                app_state.channels.loading = true;
                thread::spawn(move || {
                    let novel = source.parse_novel_and_chapters(path);
                    let _ = tx.send(RequestData::UpdateInfo((id, novel)));
                });
            }
            Ok(false)
        }
        Command::MarkRead((id, ch)) => {
            let book = app_state.library_data.find_book_mut(id);
            if book.is_none() {
                return Ok(false);
            }
            let book = book.unwrap();

            if let BookSource::Global(d) = &mut book.source_data {
                d.mark_ch_complete(ch);
                if d.get_current_chapter() == ch && !d.total_chapters == ch {
                    d.set_current_chapter(d.get_current_chapter() + 1);
                }
            }
            Ok(false)
        }
    }
}
