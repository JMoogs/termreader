// This module holds the buffer struct. This struct contains all temporary data required for the running of the TUI

use crate::{helpers::StatefulList, ui::sources::BookViewOption};
use termreader_core::book::Book;
use termreader_sources::{
    chapter::ChapterPreview,
    novel::{Novel, NovelPreview},
};

pub struct Buffer {
    /// Text that the user types, for search boxes and similar
    pub text: String,
    /// The results of searching for a novel or viewing popular novels
    pub novel_search_res: StatefulList<NovelPreview>,
    /// The chapter prewiews of a novel
    pub chapter_previews: StatefulList<ChapterPreview>,
    /// The novel/book currently being viewed
    pub novel: Option<Novel>,
    /// How far scrolled the novel description is in a preview
    pub novel_preview_scroll: usize,
    /// A book being read directly from the sources page
    pub book: Option<Book>,
    /// A temporary stateful list
    pub temporary_list: StatefulList<String>,
    /// A temporary book view option
    pub book_view_option: BookViewOption,
    /// Set true when we're reordering two categories
    pub reorder_lock: bool,
}

impl Buffer {
    /// Creates a new buffer
    pub fn build() -> Self {
        Self {
            text: String::new(),
            novel_search_res: StatefulList::new(),
            chapter_previews: StatefulList::new(),
            novel: None,
            novel_preview_scroll: 0,
            book: None,
            temporary_list: StatefulList::new(),
            book_view_option: BookViewOption::None,
            reorder_lock: false,
        }
    }

    /// Clears the entire buffer
    pub fn clear(&mut self) {
        let _ = std::mem::replace(self, Self::build());
    }
}
