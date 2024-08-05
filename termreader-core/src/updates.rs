use crate::{book::BookRef, books_context::BooksContext, id::ID};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct UpdatesCtxSerialize {
    pub(super) updates: VecDeque<UpdatesEntrySerialize>,
}

impl UpdatesCtxSerialize {
    pub(super) fn from_updates_ctx(updates_ctx: UpdatesContext) -> Self {
        Self {
            updates: updates_ctx
                .updates
                .into_iter()
                .map(|x| UpdatesEntrySerialize::from_updates_entry(x))
                .collect(),
        }
    }

    pub(super) fn to_updates_ctx(self, books: &BooksContext) -> UpdatesContext {
        UpdatesContext {
            updates: self
                .updates
                .into_iter()
                .map(|x| UpdatesEntrySerialize::to_updates_entry(x, books))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct UpdatesContext {
    pub(super) updates: VecDeque<UpdatesEntry>,
}

impl UpdatesContext {
    pub(super) fn new() -> Self {
        Self {
            updates: VecDeque::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.updates = VecDeque::new();
    }

    pub(super) fn remove_book(&mut self, book_id: ID) {
        self.updates.retain(|x| x.book.get_id() != book_id)
    }

    pub(super) fn get_updates(&self) -> &VecDeque<UpdatesEntry> {
        &self.updates
    }

    pub(super) fn get_len(&self) -> usize {
        self.updates.len()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatesEntrySerialize {
    book: ID,
    timestamp: u64,
    chapter: UpdatedChapters,
}

impl UpdatesEntrySerialize {
    fn from_updates_entry(entry: UpdatesEntry) -> Self {
        Self {
            book: entry.book.get_id(),
            timestamp: entry.timestamp,
            chapter: entry.chapter,
        }
    }

    fn to_updates_entry(self, books: &BooksContext) -> UpdatesEntry {
        UpdatesEntry {
            book: books.get(self.book).unwrap(),
            timestamp: self.timestamp,
            chapter: self.chapter,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdatesEntry {
    pub(super) book: BookRef,
    pub(super) timestamp: u64,
    pub(super) chapter: UpdatedChapters,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum UpdatedChapters {
    /// An fully inclusive range
    Range((usize, usize)),
    Single(usize),
    None,
}

impl UpdatesEntry {
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn display_new_chs(&self) -> String {
        match self.chapter {
            UpdatedChapters::Range((start, end)) => format!("Chs. {} - {}", start, end),
            UpdatedChapters::Single(ch) => format!("Ch. {}", ch),
            UpdatedChapters::None => format!("(seeing text is an error and should be reported)"),
        }
    }

    pub fn get_chapter(&self) -> UpdatedChapters {
        self.chapter
    }

    pub fn get_book_ref(&self) -> BookRef {
        BookRef::clone(&self.book)
    }
}
