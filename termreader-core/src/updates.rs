use serde::{Deserialize, Serialize};

use crate::id::ID;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct UpdatesContext {
    updates: Vec<UpdatesEntry>,
}

impl UpdatesContext {
    pub(super) fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.updates = Vec::new();
    }

    pub(super) fn remove_book(&mut self, book_id: ID) {
        self.updates.retain(|x| x.book != book_id)
    }

    pub(super) fn get_updates(&self) -> &Vec<UpdatesEntry> {
        &self.updates
    }

    pub(super) fn get_len(&self) -> usize {
        self.updates.len()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdatesEntry {
    // To avoid heavy duplication just store IDs - We only need to display updates
    // for books in the library anyways so it's ok
    book: ID,
    timestamp: u64,
    chapter: UpdatedChapters,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum UpdatedChapters {
    Range((usize, usize)),
    Single(usize),
}

impl UpdatesEntry {
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn display_new_chs(&self) -> String {
        match self.chapter {
            UpdatedChapters::Range((start, end)) => format!("Chs. {} - {}", start, end),
            UpdatedChapters::Single(ch) => format!("Ch. {}", ch),
        }
    }

    pub fn get_chapter(&self) -> UpdatedChapters {
        self.chapter
    }

    pub fn get_book_id(&self) -> ID {
        self.book
    }
}
