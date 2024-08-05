use crate::book::BookRef;
use crate::books_context::BooksContext;
use crate::ID;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct HistCtxSerialize {
    pub(super) history: VecDeque<HistEntrySerialize>,
}

impl HistCtxSerialize {
    pub(super) fn from_hist_ctx(hist_ctx: HistoryContext) -> Self {
        Self {
            history: hist_ctx
                .history
                .into_iter()
                .map(|x| HistEntrySerialize::from_hist_entry(x))
                .collect(),
        }
    }

    pub(super) fn to_hist_ctx(self, books: &BooksContext) -> HistoryContext {
        HistoryContext {
            history: self
                .history
                .into_iter()
                .map(|x| HistEntrySerialize::to_hist_entry(x, books))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct HistoryContext {
    pub(super) history: VecDeque<HistoryEntry>,
}

impl HistoryContext {
    pub(super) fn new() -> Self {
        Self {
            history: VecDeque::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.history = VecDeque::new();
    }

    pub(super) fn get_history(&self) -> &VecDeque<HistoryEntry> {
        &self.history
    }

    pub(super) fn remove_entry(&mut self, id: ID) {
        self.history.retain(|h| h.book.0.borrow().get_id() != id)
    }

    pub(super) fn add_entry(&mut self, book: BookRef) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time has gone VERY backwards")
            .as_secs();
        if book.0.borrow().is_local() {
            self.history.push_front(HistoryEntry {
                book,
                timestamp,
                chapter: 0,
            })
        } else {
            let ch = book
                .0
                .borrow()
                .get_current_ch()
                .expect("we've checked that the book isn't local");
            self.history.push_front(HistoryEntry {
                chapter: ch,
                book,
                timestamp,
            })
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct HistEntrySerialize {
    book: ID,
    timestamp: u64,
    chapter: usize,
}

impl HistEntrySerialize {
    fn from_hist_entry(entry: HistoryEntry) -> Self {
        Self {
            book: entry.book.get_id(),
            timestamp: entry.timestamp,
            chapter: entry.chapter,
        }
    }

    fn to_hist_entry(self, books: &BooksContext) -> HistoryEntry {
        HistoryEntry {
            book: books.get(self.book).unwrap(),
            timestamp: self.timestamp,
            chapter: self.chapter,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    book: BookRef,
    timestamp: u64,
    chapter: usize,
}

impl HistoryEntry {
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn get_book_name(&self) -> String {
        self.book.0.borrow().get_name().to_string()
    }

    pub fn get_chapter(&self) -> usize {
        self.chapter
    }

    pub fn get_book_id(&self) -> ID {
        self.book.0.borrow().get_id()
    }

    pub fn get_book_ref(&self) -> BookRef {
        BookRef::clone(&self.book)
    }
}
