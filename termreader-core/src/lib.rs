#![allow(dead_code)]
pub mod book;
pub mod history;
pub mod id;
mod library;
mod save;
mod sources;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use termreader_sources::sources::{Source, SourceID};

use crate::book::Book;
use crate::history::HistoryContext;
use crate::id::ID;
use crate::library::LibraryContext;
use crate::sources::SourceContext;
use anyhow::Result;
use history::HistoryEntry;

#[derive(Clone)]
pub struct Context {
    library: LibraryContext,
    history: HistoryContext,
    sources: SourceContext,
    data_path: PathBuf,
}

impl Context {
    pub fn build(data_path: PathBuf) -> Result<Self> {
        Ok(Self {
            library: save::load_library(&data_path)?,
            history: save::load_history(&data_path)?,
            sources: SourceContext::build(),
            data_path,
        })
    }

    pub fn save(self) -> Result<()> {
        save::store_history(&self.history, &self.data_path)?;
        save::store_library(&self.library, &self.data_path)?;

        Ok(())
    }

    pub fn hist_clear(&mut self) {
        self.history.clear();
    }

    pub fn hist_get(&self) -> &VecDeque<HistoryEntry> {
        self.history.get_history()
    }

    pub fn hist_remove_entry(&mut self, id: ID) {
        self.history.remove_entry(id);
    }

    pub fn hist_add_entry(&mut self, book: Book, timestamp: u64) {
        self.history.add_entry(book, timestamp);
    }

    pub fn hist_get_len(&self) -> usize {
        self.history.get_history_len()
    }

    pub fn lib_find_book(&self, id: ID) -> Option<&Book> {
        self.library.find_book(id)
    }

    pub fn lib_find_book_mut(&mut self, id: ID) -> Option<&mut Book> {
        self.library.find_book_mut(id)
    }

    pub fn lib_remove_book(&mut self, id: ID) {
        self.library.remove_book(id);
    }

    /// Add a book to a given category, or the default category if a category isn't given
    ///
    /// This function checks if the book is already in the user's library before adding it
    pub fn lib_add_book(&mut self, book: Book, category: Option<&str>) {
        self.library.add_book(book, category);
    }

    /// Move a book from one category to another. If no category is provided,
    /// the book is moved to the default category
    pub fn lib_move_book_category(&mut self, id: ID, category: Option<&str>) {
        self.library.move_category(id, category);
    }

    /// Move a category forwards by one in order
    ///
    /// Returns the new position of the category, or None if the index was out of bounds.
    pub fn lib_reorder_category_up(&mut self, category_idx: usize) -> Option<usize> {
        self.library.reorder_category_up(category_idx)
    }

    /// Move a category backwards by one in order
    ///
    /// Returns the new position of the category, or None if the index was out of bounds.
    pub fn lib_reorder_category_down(&mut self, category_idx: usize) -> Option<usize> {
        self.library.reorder_category_down(category_idx)
    }

    pub fn lib_create_category(&mut self, name: String) -> Result<(), ()> {
        self.library.create_category(name)
    }

    /// Deletes a category, failing if the category being deleted is the default category
    pub fn lib_delete_category(&mut self, name: String) -> Result<(), ()> {
        self.library.delete_category(name)
    }

    pub fn lib_rename_category(&mut self, old_name: String, new_name: String) {
        self.library.rename_category(old_name, new_name)
    }

    pub fn lib_get_categories(&self) -> &Vec<String> {
        &self.library.get_categories()
    }

    pub fn lib_get_books(&self) -> &HashMap<String, Vec<Book>> {
        self.library.get_books()
    }

    pub fn lib_get_books_mut(&mut self) -> &mut HashMap<String, Vec<Book>> {
        self.library.get_books_mut()
    }

    pub fn source_get_by_id(&self, id: SourceID) -> Option<&Source> {
        self.sources.get_source_by_id(id)
    }

    pub fn source_get_info(&self) -> Vec<(SourceID, String)> {
        self.sources.get_source_info()
    }

    /// Finds whether a book already exists for a given URL, returning it if it does
    pub fn find_book_by_url(&self, url: String) -> Option<&Book> {
        self.history
            .find_book_from_url(url.clone())
            .or_else(|| self.library.find_book_from_url(url))
    }

    pub fn find_book_by_url_mut(&mut self, url: String) -> Option<&mut Book> {
        self.history
            .find_book_from_url_mut(url.clone())
            .or_else(|| self.library.find_book_from_url_mut(url))
    }

    pub fn find_book(&self, id: ID) -> Option<&Book> {
        Some(
            self.library
                .find_book(id)
                .or_else(|| self.history.find_book(id))?,
        )
    }

    pub fn find_book_mut(&mut self, id: ID) -> Option<&mut Book> {
        let b = self.library.find_book_mut(id);
        if b.is_some() {
            return b;
        }

        Some(self.history.find_book_mut(id)?)
    }
}
