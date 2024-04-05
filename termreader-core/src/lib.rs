#![allow(dead_code)]
pub mod book;
pub mod history;
pub mod id;
mod library;
mod save;
mod sources;

use std::collections::HashMap;
use std::collections::VecDeque;
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
}

impl Context {
    pub fn build() -> Result<Self> {
        Ok(Self {
            library: save::load_library()?,
            history: save::load_history()?,
            sources: SourceContext::build(),
        })
    }

    pub fn save(self) -> Result<()> {
        save::store_history(&self.history)?;
        save::store_library(&self.library)?;

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

    pub fn lib_add_book(&mut self, book: Book, category: Option<String>) {
        self.library.add_book(book, category);
    }

    pub fn lib_move_book_category(&mut self, id: ID, category: Option<String>) {
        self.library.move_category(id, category);
    }

    pub fn lib_create_category(&mut self, name: String) {
        self.library.create_category(name);
    }

    pub fn lib_delete_category(&mut self, name: String) {
        self.library.delete_category(name);
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
}
