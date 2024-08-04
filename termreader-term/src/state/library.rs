// This module contains data related to the library tab of the TUI.

use ratatui::widgets::ListState;
use termreader_core::{book::Book, Context};

use crate::{helpers::StatefulList, trace_dbg};

/// Data related to the library tab
pub struct LibData {
    /// The index of the currently selected category
    pub current_category_idx: usize,
    /// The currently selected book. Resets upon swapping categories.
    selected_book: ListState,
    /// Options for a selected global book
    pub global_selected_book_opts: StatefulList<String>,
    /// Options for categories
    pub category_options: StatefulList<String>,
}

impl LibData {
    /// Creates an instance of LibData
    pub(super) fn build(ctx: &Context) -> Self {
        let cat_name = &ctx.lib_get_categories()[0];
        let selected_book = if ctx
            .lib_get_books()
            .get(cat_name)
            .expect("The first category should always exist but does not")
            .is_empty()
        {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };

        Self {
            current_category_idx: 0, // There is always at least 1 category so this is always valid
            selected_book,
            global_selected_book_opts: StatefulList::from(vec![
                String::from("Continue reading"),
                String::from("Update chapter list"),
                String::from("Move to category"),
                String::from("Rename"),
                String::from("Reset Progress"),
                String::from("Remove book from library"),
                String::from("Open in browser"),
            ]),
            category_options: StatefulList::from(vec![
                String::from("Create categories"),
                String::from("Re-order categories"),
                String::from("Rename categories"),
                String::from("Delete categories"),
            ]),
        }
    }

    /// Get the currently selected category. This function will always return a valid index
    pub fn get_selected_category(&self) -> usize {
        return self.current_category_idx;
    }

    /// Selects the next category, wrapping around as required
    pub fn select_next_category(&mut self, ctx: &Context) {
        let list_len = ctx.lib_get_categories().len();
        self.current_category_idx = (self.current_category_idx + 1) % list_len;

        self.selected_book.select(None);
        self.fix_book_selection_state(ctx);
    }

    /// Selects the previous category, wrapping around as required
    pub fn select_previous_category(&mut self, ctx: &Context) {
        let max_idx = ctx.lib_get_categories().len() - 1;
        if self.current_category_idx == 0 {
            self.current_category_idx = max_idx;
        } else {
            self.current_category_idx -= 1;
        }

        self.selected_book.select(None);
        self.fix_book_selection_state(ctx);
    }

    /// Selects the first book if none is selected, and there is a book in the current category
    pub fn fix_book_selection_state(&mut self, ctx: &Context) {
        if self.get_current_category_size(ctx) > 0 && self.selected_book.selected().is_none() {
            self.selected_book.select(Some(0))
        }
    }

    pub fn reset_selection(&mut self, ctx: &Context) {
        self.selected_book.select(None);
        self.fix_book_selection_state(ctx);
    }

    /// Returns the size of the currently selected category
    pub fn get_current_category_size(&mut self, ctx: &Context) -> usize {
        let cat_name = &ctx.lib_get_categories()[self.get_selected_category()];
        trace_dbg!(cat_name.clone());
        trace_dbg!(ctx.lib_get_books().keys());
        ctx.lib_get_books().get(cat_name).unwrap().len()
    }

    /// Returns a mutable reference to the state representing the selected book. This will always succeed
    pub fn get_selected_book_state_mut(&mut self) -> &mut ListState {
        &mut self.selected_book
    }

    /// Gets the currently selected book
    pub fn get_selected_book<'a>(&self, ctx: &'a Context) -> Option<&'a Book> {
        let idx = self.selected_book.selected()?;
        let category_name = ctx.lib_get_categories().get(self.get_selected_category())?;

        Some(&ctx.lib_get_books().get(category_name)?[idx])
    }

    /// Gets the currently selected book mutably
    pub fn get_selected_book_mut<'a>(&self, ctx: &'a mut Context) -> Option<&'a mut Book> {
        let idx = self.selected_book.selected()?;
        let category_name = ctx
            .lib_get_categories()
            .get(self.get_selected_category())?
            .to_string();

        Some(&mut ctx.lib_get_books_mut().get_mut(&category_name)?[idx])
    }

    /// Selects the next book in the currently selected category. If no book is selected, the first book is selected
    pub fn select_next_book(&mut self, ctx: &Context) {
        let size = self.get_current_category_size(ctx);
        match self.selected_book.selected() {
            Some(n) => {
                // If a book is selected it is certain there is at least 1 book in the category.
                self.selected_book.select(Some((n + 1) % size))
            }
            None => {
                if size == 0 {
                    return;
                } else {
                    self.selected_book.select(Some(0))
                }
            }
        }
    }

    /// Selects the previous book in the currently selected category. If no book is selected, the last book is selected
    pub fn select_prev_book(&mut self, ctx: &Context) {
        let size = self.get_current_category_size(ctx);
        match self.selected_book.selected() {
            Some(n) => {
                if n == 0 {
                    self.selected_book.select(Some(size - 1))
                } else {
                    self.selected_book.select(Some(n - 1))
                }
            }
            None => {
                if size == 0 {
                    return;
                } else {
                    self.selected_book.select(Some(size - 1))
                }
            }
        }
    }
}
