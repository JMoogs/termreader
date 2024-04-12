use crate::Book;
use crate::ID;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct HistoryContext {
    history: VecDeque<HistoryEntry>,
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
        self.history.retain(|h| h.book.get_id() != id)
    }

    pub(super) fn add_entry(&mut self, book: Book, timestamp: u64) {
        if book.is_local() {
            self.history.push_front(HistoryEntry {
                book,
                timestamp,
                chapter: 0,
            })
        } else {
            self.history.push_front(HistoryEntry {
                chapter: book.global_get_current_ch(),
                book,
                timestamp,
            })
        }
    }

    pub(super) fn find_book_from_url(&self, url: String) -> Option<&Book> {
        for entry in self.history.iter() {
            if !entry.book.is_local() {
                let novel = entry.book.global_get_novel();
                if novel.get_full_url() == url {
                    return Some(&entry.book);
                }
            }
        }

        None
    }

    pub(super) fn find_book_from_url_mut(&mut self, url: String) -> Option<&mut Book> {
        for entry in self.history.iter_mut() {
            if !entry.book.is_local() {
                let novel = entry.book.global_get_novel();
                if novel.get_full_url() == url {
                    return Some(&mut entry.book);
                }
            }
        }

        None
    }

    pub(super) fn find_book(&self, id: ID) -> Option<&Book> {
        // for entry in self.history.iter() {
        // }
        let e = self.history.iter().find(|e| e.book.get_id() == id);
        Some(&e?.book)
    }

    pub(super) fn find_book_mut(&mut self, id: ID) -> Option<&mut Book> {
        let e = self.history.iter_mut().find(|e| e.book.get_id() == id);
        Some(&mut e?.book)
    }

    pub(super) fn get_history_len(&self) -> usize {
        self.history.len()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    book: Book,
    timestamp: u64,
    chapter: usize,
}

impl HistoryEntry {
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn get_book_name(&self) -> String {
        self.book.get_name().to_string()
    }

    pub fn get_book(&self) -> &Book {
        &self.book
    }

    pub fn get_chapter(&self) -> usize {
        self.chapter
    }
}
