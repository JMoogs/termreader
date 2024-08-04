// This module contains data related to the sources tab of the TUI.

use ratatui::widgets::ListState;
use termreader_core::Context;
use termreader_sources::sources::SourceID;

use crate::helpers::StatefulList;

/// Data related to the sources tab
pub struct SourceData {
    /// The currently selected source
    selected_source: ListState,
    /// The options for each source
    pub source_options: StatefulList<String>,
    // The options for a novel being viewed
    pub novel_options: StatefulList<String>,
    /// The selected field in a novel preview
    pub novel_preview_selected_field: SourceNovelPreviewSelection,
}

/// An enum representing the selected field when previewing a novel on the source page
#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum SourceNovelPreviewSelection {
    Summary,
    #[default]
    Chapters,
    Options,
}

impl SourceNovelPreviewSelection {
    pub fn next_opts(&mut self) {
        *self = match self {
            SourceNovelPreviewSelection::Summary => SourceNovelPreviewSelection::Options,
            SourceNovelPreviewSelection::Chapters => SourceNovelPreviewSelection::Summary,
            SourceNovelPreviewSelection::Options => SourceNovelPreviewSelection::Chapters,
        }
    }

    pub fn prev_opts(&mut self) {
        *self = match self {
            SourceNovelPreviewSelection::Summary => SourceNovelPreviewSelection::Chapters,
            SourceNovelPreviewSelection::Chapters => SourceNovelPreviewSelection::Options,
            SourceNovelPreviewSelection::Options => SourceNovelPreviewSelection::Summary,
        }
    }

    pub fn next_no_opts(&mut self) {
        *self = match self {
            SourceNovelPreviewSelection::Summary => SourceNovelPreviewSelection::Chapters,
            SourceNovelPreviewSelection::Chapters => SourceNovelPreviewSelection::Summary,
            SourceNovelPreviewSelection::Options => unreachable!(),
        }
    }

    pub fn prev_no_opts(&mut self) {
        self.next_no_opts();
    }
}

impl SourceData {
    /// Creates an instance of SourceData
    pub fn build() -> Self {
        Self {
            selected_source: ListState::default().with_selected(Some(0)), // There should be at least one source at all times.
            source_options: StatefulList::from(vec![
                String::from("Search"),
                String::from("View Popular"),
            ]),
            novel_options: StatefulList::from(vec![
                String::from("Start from beginning"),
                String::from("Add to library"),
                String::from("Open in browser"),
            ]),
            novel_preview_selected_field: SourceNovelPreviewSelection::Chapters,
        }
    }

    pub fn reset_novel_options(&mut self) {
        self.novel_options = StatefulList::from(vec![
            String::from("Start from beginning"),
            String::from("Add to library"),
            String::from("Open in browser"),
        ]);
    }

    /// Swap between adding/removing a book to/from the library
    pub fn swap_library_options(&mut self) {
        let it = self.novel_options.items.iter_mut();
        for elem in it {
            if elem == &String::from("Add to library") {
                let _ = std::mem::replace(elem, String::from("Remove from library"));
            } else if elem == &String::from("Remove from library") {
                let _ = std::mem::replace(elem, String::from("Add to library"));
            } else {
                continue;
            }
        }
    }

    /// Returns a mutable reference to the state representing the selected source. This will always succeed. This function should **not** be used directly
    pub fn get_selected_source_state_mut(&mut self) -> &mut ListState {
        &mut self.selected_source
    }

    /// Selects the next source
    pub fn select_next(&mut self, ctx: &Context) {
        let size = ctx.source_get_info().len();
        let sel = self.selected_source.selected().unwrap();
        self.selected_source.select(Some((sel + 1) % size))
    }

    /// Selects the previous source
    pub fn select_prev(&mut self, ctx: &Context) {
        let size = ctx.source_get_info().len();
        let sel = self.selected_source.selected().unwrap();
        if sel == 0 {
            self.selected_source.select(Some(size - 1));
        } else {
            self.selected_source.select(Some(sel - 1));
        }
    }

    pub fn get_selected_source_id(&self, ctx: &Context) -> SourceID {
        let idx = self.selected_source.selected().unwrap();
        ctx.source_get_info()[idx].0
    }
}
