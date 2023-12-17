use crate::{
    appstate::{HistoryData, LibraryJson},
    shutdown::HistoryJson,
};
use anyhow::Result;
use ratatui::widgets::ListState;
use std::{collections::VecDeque, fs};

pub fn load_books() -> Result<LibraryJson> {
    let book_data = fs::read_to_string("books.json");
    if book_data.is_err() {
        return Ok(LibraryJson::empty());
    }
    let book_data = book_data.unwrap();

    let books: LibraryJson = serde_json::from_str(&book_data)?;
    Ok(books)
}

pub fn load_history() -> Result<HistoryData> {
    let history_data = fs::read_to_string("history.json");
    match history_data {
        Ok(d) => {
            let data: HistoryJson = serde_json::from_str(&d)?;
            Ok(HistoryData {
                history: data.history,
                selected: ListState::default(),
            })
        }
        Err(_) => {
            return Ok(HistoryData {
                history: VecDeque::new(),
                selected: ListState::default(),
            })
        }
    }
}
