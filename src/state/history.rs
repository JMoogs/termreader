use crate::BookInfo;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A struct to hold a user's reading history
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryData {
    /// The history entries
    pub history: VecDeque<HistoryEntry>,
    /// The currently selected history entry
    #[serde(skip)]
    pub selected: ListState,
}

impl HistoryData {
    /// Clears all history
    pub fn clear(&mut self) {
        self.history = VecDeque::new();
        self.selected.select(None);
    }
}

/// The data related to the history of a single novel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The book's info
    pub book: BookInfo,
    /// The timestamp at which the book was last read
    // I'm unsure whether this is the last time they stopped reading, or the last time they started reading, need to check
    pub timestamp: u64,
    /// The chapter the user was last reading
    pub chapter: usize,
}
