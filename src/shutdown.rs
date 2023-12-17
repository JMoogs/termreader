use crate::appstate::{HistoryData, HistoryEntry, LibBookInfo, LibraryJson};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs};

pub fn store_books(books: &LibraryJson) -> Result<()> {
    let json = serde_json::to_string(books)?;
    fs::write("books.json", json)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryJson {
    history: VecDeque<HistoryEntry>,
}

impl HistoryJson {
    pub fn from_history(history: &HistoryData) -> HistoryJson {
        HistoryJson {
            history: history.history.clone(),
        }
    }
}

pub fn store_history(history: &HistoryJson) -> Result<()> {
    let json = serde_json::to_string(history)?;
    fs::write("history.json", json)?;
    Ok(())
}

// TEMP FOR TESTING:

pub fn generate_lib_info() -> LibraryJson {
    let mut books = Vec::new();

    let gatsby = LibBookInfo::from_local("sample_book.txt", None).unwrap();
    books.push(gatsby);

    let categories: Vec<String> = vec!["Favourites".into(), "Read later".into(), "meh".into()];

    LibraryJson::new(categories, books)
}
