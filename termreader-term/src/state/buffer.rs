use crate::appstate::SourceBookBox;
use crate::helpers::StatefulList;
use termreader_sources::chapter::ChapterPreview;
use termreader_sources::novel::{Novel, NovelPreview};

/// A buffer for storing any temporary data
#[derive(Debug, Clone, Default)]
pub struct AppBuffer {
    /// For text boxes and other typed interfaces
    pub text: String,
    pub novel_previews: StatefulList<NovelPreview>,
    pub chapter_previews: StatefulList<ChapterPreview>,
    pub novel: Option<Novel>,
    pub novel_preview_scroll: usize,
    pub novel_preview_selection: SourceBookBox,
}

impl AppBuffer {
    /// Entirely clears the buffer
    pub fn clear(&mut self) {
        let _ = std::mem::take(self);
    }

    /// Clears all entries related to a novel from the buffer
    pub fn clear_novel(&mut self) {
        self.chapter_previews = StatefulList::new();
        self.novel = None;
        self.novel_preview_scroll = 0;
        self.novel_preview_selection = SourceBookBox::Options;
    }

    /// Clears the buffer, and selects the correct options for viewing a chapter list
    pub fn ch_list_setup(&mut self) {
        self.clear();
        self.novel_preview_selection = SourceBookBox::Chapters;
    }
}
