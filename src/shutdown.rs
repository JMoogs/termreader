use crate::appstate::{BookInfo, LibraryJson};
use std::fs;

pub fn store_books(books: &LibraryJson) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string(books)?;
    fs::write("books.json", json)?;
    Ok(())
}

// TEMP FOR TESTING:

pub fn generate_lib_info() -> LibraryJson {
    let mut books = Vec::new();

    let gatsby = BookInfo::from_local("sample_book.txt", None).unwrap();
    books.push(gatsby);

    let categories: Vec<String> = vec!["Favourites".into(), "Read later".into(), "meh".into()];

    LibraryJson::new(categories, books)
}
