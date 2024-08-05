// This module contains data relating to when the user is reading a book
use termreader_core::book::{BookRef, ChapterProgress};
use termreader_sources::chapter::Chapter;

use crate::reader::{GlobalReader, GlobalReaderContents, GlobalReaderState};

/// Data related to what's being read
pub struct ReaderData {
    book: Option<BookRef>,
    chapter: Option<Chapter>,
    data: Option<GlobalReader>,
    // Set by the renderer as required
    term_height: u16,
    term_width: u16,
}

impl ReaderData {
    pub(super) fn build() -> Self {
        Self {
            book: None,
            chapter: None,
            data: None,
            term_height: 0,
            term_width: 0,
        }
    }

    pub(super) fn set_data(&mut self, book: BookRef, chapter: Option<Chapter>) {
        if chapter.is_none() {
            unimplemented!()
        };
        self.book = Some(book);
        self.data = Some(GlobalReader::from_chapter(chapter.as_ref().unwrap()));
        self.chapter = chapter;
    }

    pub fn get_reader_state_mut(&mut self) -> Option<&mut GlobalReaderState> {
        Some(&mut self.data.as_mut()?.state)
    }

    pub fn scroll_down(&mut self) {
        if let Some(ref mut reader) = self.data {
            reader.scroll_down(self.term_width, self.term_height);
        }
    }

    pub fn scroll_up(&mut self) {
        if let Some(ref mut reader) = self.data {
            reader.scroll_up(self.term_width, self.term_height);
        }
    }

    pub fn get_reader_contents(&self) -> Option<GlobalReaderContents> {
        let d = self.data.as_ref()?.contents.clone();
        Some(d)
    }

    pub fn get_book(&self) -> Option<BookRef> {
        let r = self.book.clone()?;
        Some(r)
    }

    pub fn get_chapter(&self) -> &Option<Chapter> {
        &self.chapter
    }

    pub fn set_dimensions(&mut self, dims: (u16, u16)) {
        (self.term_width, self.term_height) = dims
    }

    /// Get the progress of the current chapter as a percentage.
    /// This is calculated based on the last word displayed on the screen.
    pub fn get_ch_progress_pct(&self) -> Option<f64> {
        if let Some(reader) = &self.data {
            let prog = reader.get_progress();
            let total_words = reader.get_total_words();
            return match prog {
                termreader_core::book::ChapterProgress::Location(_) => unreachable!(),
                termreader_core::book::ChapterProgress::Word((_, end)) => {
                    Some(end as f64 / total_words as f64)
                }
                termreader_core::book::ChapterProgress::Finished => Some(1.0),
            };
        }
        None
    }

    /// Get the progress of the current chapter
    pub fn get_ch_progress(&self) -> Option<ChapterProgress> {
        Some(self.data.as_ref()?.get_progress())
    }
}
