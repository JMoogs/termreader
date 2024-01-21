use crate::state::history::HistoryData;
use crate::state::library::LibraryData;
use anyhow::Result;
use std::fs;

/// Writes library data to a `books.json` file
pub fn store_books(books: &LibraryData) -> Result<()> {
    let json = serde_json::to_string(books)?;
    fs::write("books.json", json)?;
    Ok(())
}

/// Writes history data to a `history.json` file
pub fn store_history(history: &HistoryData) -> Result<()> {
    let json = serde_json::to_string(history)?;
    fs::write("history.json", json)?;
    Ok(())
}
