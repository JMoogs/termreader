// This module contains data related to the history tab of the TUI.

use ratatui::widgets::ListState;
use termreader_core::{history::HistoryEntry, Context};

use crate::helpers::StatefulList;

pub struct HistoryData {
    /// The currently selected history entry
    selected_entry: ListState,
    pub local_book_options: StatefulList<String>,
    pub global_book_options: StatefulList<String>,
    pub view_book_with_opts: bool,
}

impl HistoryData {
    /// Creates an instance of HistoryData
    pub fn build(ctx: &Context) -> Self {
        let selected_entry = if ctx.hist_get_len() == 0 {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };
        Self {
            selected_entry,
            local_book_options: StatefulList::from(vec![
                String::from("Continue reading"),
                String::from("Remove from history"),
            ]),
            global_book_options: StatefulList::from(vec![
                String::from("Continue reading"),
                String::from("View book"),
                String::from("Remove from history"),
            ]),
            view_book_with_opts: true,
        }
    }

    /// Returns a mutable reference to the state representing the selected history entry. This will always succeed. This function should **not** be used directly
    pub fn get_selected_entry_mut(&mut self) -> &mut ListState {
        &mut self.selected_entry
    }

    /// Returns a reference to the entry that's selected
    pub fn get_selected_book<'a>(&self, ctx: &'a Context) -> Option<&'a HistoryEntry> {
        let hist = ctx.hist_get();
        Some(&hist[self.selected_entry.selected()?])
    }

    pub fn select_next_entry(&mut self, ctx: &Context) {
        if ctx.hist_get_len() == 0 {
            self.selected_entry.select(None);
            return;
        }
        match self.selected_entry.selected() {
            Some(s) => {
                self.selected_entry
                    .select(Some((s + 1) % ctx.hist_get_len()));
            }
            None => {
                if ctx.hist_get_len() != 0 {
                    self.selected_entry.select(Some(0));
                }
            }
        }
    }

    pub fn select_prev_entry(&mut self, ctx: &Context) {
        match self.selected_entry.selected() {
            Some(s) => {
                if s == 0 {
                    if ctx.hist_get_len() != 0 {
                        self.selected_entry.select(Some(ctx.hist_get_len() - 1));
                    }
                } else {
                    self.selected_entry.select(Some(s - 1));
                }
            }
            None => {
                let len = ctx.hist_get_len();
                if len != 0 {
                    self.selected_entry.select(Some(len - 1));
                }
            }
        }
    }
}
