use crate::state::history::HistoryData;
use crate::state::library::LibraryData;
use anyhow::Result;
use std::fs;

/// Loads library data from a `books.json` file
pub fn load_books() -> Result<LibraryData> {
    let book_data = fs::read_to_string("books.json");
    if book_data.is_err() {
        return Ok(LibraryData::empty());
    } else {
        let book_data = book_data.unwrap();
        let mut books: LibraryData = serde_json::from_str(&book_data)?;
        // A category must be selected, though selection data is not stored when saving.
        books.categories.select_first();
        return Ok(books);
    }
}

/// Loads history data from a `history.json` file
pub fn load_history() -> Result<HistoryData> {
    let history_data = fs::read_to_string("history.json");
    match history_data {
        Ok(d) => {
            return Ok(serde_json::from_str(&d)?);
        }
        Err(_) => Ok(HistoryData::default()),
    }
}
