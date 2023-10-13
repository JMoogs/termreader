use crate::appstate::{BookSource, LibraryBookInfo, LibraryJson, LocalBookData, ID};
use std::fs;

pub fn store_books(books: &LibraryJson) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string(books)?;
    fs::write("books.json", json)?;
    Ok(())
}

// TEMP FOR TESTING:

pub fn generate_lib_info() -> LibraryJson {
    let mut books = Vec::new();

    let b1 = LibraryBookInfo {
        name: "Overgeared".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 1068,
            total_pages: 1914,
            hash: "test".into(),
        }),
        category: None,
        id: ID::new(0),
    };

    let b2 = LibraryBookInfo {
        name: "Shadow Slave".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 402,
            total_pages: 1194,
            hash: "test".into(),
        }),
        category: None,
        id: ID::new(0),
    };

    let b3 = LibraryBookInfo {
        name: "Nine Star Hegemon Body Art".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 5008,
            total_pages: 5744,
            hash: "test".into(),
        }),
        category: None,
        id: ID::new(0),
    };
    let b4 = LibraryBookInfo {
        name: "My Three Wives are Beautiful Vampires".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 839,
            total_pages: 860,
            hash: "test".into(),
        }),
        category: None,
        id: ID::new(0),
    };
    let b5 = LibraryBookInfo {
        name: "Magic Emperor".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 123,
            total_pages: 769,
            hash: "test".into(),
        }),
        category: Some("Favourites".into()),
        id: ID::new(0),
    };
    let b6 = LibraryBookInfo {
        name: "Supreme Magus".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 1022,
            total_pages: 2754,
            hash: "test".into(),
        }),
        category: Some("Favourites".into()),
        id: ID::new(0),
    };
    let b7 = LibraryBookInfo {
        name: "Reincarnated With The Strongest System".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 1475,
            total_pages: 1475,
            hash: "test".into(),
        }),
        category: Some("meh".into()),
        id: ID::new(0),
    };
    let b8 = LibraryBookInfo {
        name: "Dual Cultivation".into(),
        source_data: BookSource::Local(LocalBookData {
            path_to_book: "test".into(),
            current_page: 0,
            total_pages: 1060,
            hash: "test".into(),
        }),
        category: Some("meh".into()),
        id: ID::new(0),
    };

    books.push(b1);
    books.push(b2);
    books.push(b3);
    books.push(b4);
    books.push(b5);
    books.push(b6);
    books.push(b7);
    books.push(b8);

    let categories: Vec<String> = vec!["Favourites".into(), "Read later".into(), "meh".into()];

    LibraryJson::new(categories, books)
}
