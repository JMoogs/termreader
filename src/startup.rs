use crate::appstate::LibraryJson;
use std::fs;

pub fn load_books() -> Result<LibraryJson, anyhow::Error> {
    let book_data = fs::read_to_string("books.json");
    if book_data.is_err() {
        return Ok(LibraryJson::empty());
    }
    let book_data = book_data.unwrap();

    let books: LibraryJson = serde_json::from_str(&book_data)?;
    Ok(books)
}
