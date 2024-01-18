use crate::appstate::{HistoryData, LibraryJson};
use anyhow::Result;
use std::fs;

/// Takes library data in its serialisable form, and writes it to the books.json file.
pub fn store_books(books: &LibraryJson) -> Result<()> {
    let json = serde_json::to_string(books)?;
    fs::write("books.json", json)?;
    Ok(())
}

pub fn store_history(history: &HistoryData) -> Result<()> {
    let json = serde_json::to_string(history)?;
    fs::write("history.json", json)?;
    Ok(())
}
