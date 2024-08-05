#![allow(dead_code, unused_variables)]
pub mod book;
mod books_context;
pub mod history;
pub mod id;
mod library;
mod save;
mod sources;
mod updates;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{collections::VecDeque, time::UNIX_EPOCH};

use crate::books_context::BooksContext;
use crate::history::HistoryContext;
use crate::id::ID;
use crate::library::LibraryContext;
use crate::sources::SourceContext;
use crate::updates::UpdatesContext;
use book::{Book, BookRef};
use history::HistoryEntry;
use termreader_sources::sources::{Source, SourceID};
use thiserror::Error;
use updates::{UpdatedChapters, UpdatesEntry};

#[derive(Error, Debug)]
pub enum TRError {
    #[error("(de)serialization error: {0}")]
    SerializationFailure(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IOFailure(#[from] std::io::Error),
    #[error("the same identifier was used more than once")]
    Duplicate,
    #[error("an invalid choice was given: {0}")]
    InvalidChoice(String),
    #[error("a book was referred to but it does not exist in memory")]
    BookMissing,
    #[error("the operation was redundant")]
    Redundant,
    #[error("the function was used wrongly: {0}")]
    BadUse(String),
    #[error("the argument was invalid: {0}")]
    InvalidArgument(String),
}

#[derive(Clone, Debug)]
pub struct Context {
    books: BooksContext,
    library: LibraryContext,
    history: HistoryContext,
    sources: SourceContext,
    updates: UpdatesContext,
    data_path: PathBuf,
}

impl Context {
    /// Build a `Context` from the save files
    pub fn build(data_path: PathBuf) -> Result<Self, TRError> {
        let books = save::load_books(&data_path)?;
        Ok(Self {
            library: save::load_library(&data_path, &books)?,
            history: save::load_history(&data_path, &books)?,
            sources: SourceContext::build(),
            updates: save::load_updates(&data_path, &books)?,
            data_path,
            books,
        })
    }

    /// Save a `Context` to files to be loaded later
    ///
    /// The location where the files are saved is the location you call `Context::build` with
    pub fn save(self) -> Result<(), TRError> {
        save::store_history(self.history, &self.data_path)?;
        save::store_library(self.library, &self.data_path)?;
        save::store_updates(self.updates, &self.data_path)?;
        save::store_books(&self.books, &self.data_path)?;

        Ok(())
    }

    /// Clear all history data
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get the history of the user
    pub fn get_history(&self) -> &VecDeque<HistoryEntry> {
        self.history.get_history()
    }

    /// Remove the history entry for the provided book
    ///
    /// Fails silently if the history entry does not exist
    pub fn remove_history_entry(&mut self, id: ID) {
        let b = self.books.get(id);
        if let Some(book) = b {
            book.0.borrow_mut().in_history = false;
        }
        self.history.remove_entry(id)
    }

    /// Add a new history entry for the provided book
    pub fn add_history_entry(&mut self, book: ID) {
        let b = self.books.get(book);
        match b {
            Some(book) => {
                book.0.borrow_mut().in_history = true;
                self.history.add_entry(book);
            }
            None => (),
        }
    }

    /// Get the history entry count
    pub fn get_history_entry_count(&self) -> usize {
        self.history.history.len()
    }

    /// Remove a book from the user's library
    pub fn remove_from_lib(&mut self, id: ID) {
        let Some(book) = self.books.get(id) else {
            return;
        };
        self.library.remove_book(id);

        {
            let mut b = book.0.borrow_mut();
            b.in_library = false;
            b.category = None;
        }

        self.books.remove_unneeded();
    }

    /// Add a book to the user's library
    ///
    /// If a category is provided, the book will be added to that category,
    /// otherwise, it will be added to the default category.
    ///
    /// Errors if:
    /// - The book is missing from memory
    /// - The book is already in the library
    pub fn add_to_lib(&mut self, id: ID, category: Option<&str>) -> Result<(), TRError> {
        let Some(book) = self.books.get(id) else {
            return Err(TRError::BookMissing);
        };

        // Don't add the book if it's already in the library
        if book.in_library() {
            return Err(TRError::Redundant);
        }

        // Set book data
        {
            let mut b = book.0.borrow_mut();
            b.in_library = true;
            b.category = category.map(|x| x.to_string());
        }

        // Set library data
        match category {
            None => {
                let list = self
                    .library
                    .books
                    .get_mut(&self.library.default_category_name)
                    .expect("the default category should always exist");
                list.push(book);
                Ok(())
            }
            Some(c) => match self.library.books.get_mut(c) {
                Some(list) => {
                    list.push(book);
                    Ok(())
                }
                None => self.add_to_lib(id, None),
            },
        }
    }

    /// Move a book from one category to another
    ///
    /// If no category is provided, the book is moved to the default category
    pub fn move_book_category(&mut self, id: ID, category: Option<&str>) -> Result<(), TRError> {
        let Some(book) = self.books.get(id) else {
            return Err(TRError::BookMissing);
        };

        {
            let b = book.0.borrow();
            // Fail if the book isn't in the library
            if !b.in_library {
                return Err(TRError::BadUse(String::from(
                    "the book is not in the library",
                )));
            }
            // Fail if the book is already in the category
            if b.category.as_ref().map(|x| x.as_str()) == category {
                return Err(TRError::Redundant);
            }
        }

        // Use the function within `self.library` to prevent the book being cleared from
        // the map before being re-added
        self.library.remove_book(id);
        {
            let mut b = book.0.borrow_mut();
            b.category = None;
            b.in_library = false;
        }
        self.add_to_lib(id, category)
    }

    /// Move a category forwards by one in order
    ///
    /// Returns the new position of the category, or None if the index was out of bounds.
    pub fn reorder_category_forwards(&mut self, category_idx: usize) -> Option<usize> {
        self.library.reorder_category_forwards(category_idx)
    }

    /// Move a category backwards by one in order
    ///
    /// Returns the new position of the category, or None if the index was out of bounds.
    pub fn reorder_category_backwards(&mut self, category_idx: usize) -> Option<usize> {
        self.library.reorder_category_backwards(category_idx)
    }

    /// Create a new category in the library
    ///
    /// Errors if the category name is already in use
    pub fn create_library_category(&mut self, name: String) -> Result<(), TRError> {
        self.library.create_category(name)
    }

    /// Deletes a library category
    ///
    /// Errors if the default category is selected for deletion
    pub fn delete_library_category(&mut self, name: String) -> Result<(), TRError> {
        self.library.delete_category(name)
    }

    /// Rename a library category
    ///
    /// Errors if the new category name is already in use
    pub fn rename_library_category(
        &mut self,
        old_name: String,
        new_name: String,
    ) -> Result<(), TRError> {
        self.library.rename_category(old_name, new_name)
    }

    /// Get a list of all library categories
    ///
    /// The categories are given in the order in which they should be displayed
    pub fn get_library_categories(&self) -> &Vec<String> {
        &self.library.get_categories()
    }

    /// Returns a source from it's ID, returning `None` if the source ID is not valid
    pub fn get_source_by_id(&self, source_id: SourceID) -> Option<&Source> {
        self.sources.get_source_by_id(source_id)
    }

    /// Returns a vector of all source IDs and the sources' names
    pub fn get_source_info(&self) -> Vec<(SourceID, String)> {
        self.sources.get_source_info()
    }

    /// Get a book's source, returning none if the book does not have one
    pub fn get_book_source(&self, book_id: ID) -> Option<&Source> {
        let b = self.books.get(book_id);
        match b {
            None => None,
            Some(book) => {
                let source_id = book.0.borrow().global_get_source_id();
                self.get_source_by_id(source_id)
            }
        }
    }

    /// Returns the current save directory
    pub fn get_save_dir(&self) -> PathBuf {
        self.data_path.clone()
    }

    /// Clear all update data
    pub fn clear_updates(&mut self) {
        self.updates.clear()
    }

    /// Get update data
    pub fn get_updates(&self) -> &VecDeque<UpdatesEntry> {
        self.updates.get_updates()
    }

    /// Add a new update entry
    pub fn add_updates_entry(&mut self, book: ID, chapters: UpdatedChapters) {
        let Some(book) = self.books.get(book) else {
            return;
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time has gone VERY backwards")
            .as_secs();
        self.updates.updates.push_front(UpdatesEntry {
            book,
            timestamp,
            chapter: chapters,
        })
    }

    /// Get the amount of update entries
    pub fn get_updates_entry_count(&self) -> usize {
        self.updates.get_len()
    }

    /// Removes an update entry
    pub fn remove_updates_entry(&mut self, book: ID) {
        self.updates.remove_book(book);
    }

    /// Returns true if a book is stored in memory
    pub fn book_exists(&self, id: ID) -> bool {
        self.books.get(id).is_some()
    }

    /// Returns true if a book is stored in memory
    pub fn book_exists_by_url(&self, url: String) -> bool {
        self.books.find_book_by_url(url).is_some()
    }

    /// Returns a `BookRef` to a `Book` if it exists, otherwise returning `None`.
    pub fn get_book(&self, id: ID) -> Option<BookRef> {
        self.books.get(id)
    }

    /// Returns a `BookRef` to a `Book` if it exists, otherwise returning `None`.
    pub fn get_book_url(&self, url: String) -> Option<BookRef> {
        self.books.find_book_by_url(url)
    }

    /// Adds a `Book` to memory
    pub fn add_book(&mut self, book: Book) {
        self.books.add_book(book)
    }

    /// Replaces the `Book` with the new one
    pub fn replace_book(&mut self, book_id: ID, new_book: Book) {
        if let Some(b) = self.get_book(book_id) {
            b.0.replace(new_book);
        }
    }

    pub fn get_library_books(&self) -> &HashMap<String, Vec<BookRef>> {
        &self.library.books
    }
}
