use crate::{
    appstate::{BookSource, LibraryBookInfo},
    reader::buffer::BufferType,
};

use self::buffer::{BookPortion, BookProgressData, LocalBuffer, NextDisplay};

use anyhow::Result;

pub mod buffer;
pub mod widget;

pub struct ReaderData {
    pub book_info: LibraryBookInfo,
    pub portion: BookPortion,
}

impl ReaderData {
    pub fn get_progress(&self) -> Result<BookProgressData> {
        self.portion.get_progress()
    }

    pub fn set_progress(&mut self) -> Result<()> {
        self.book_info.set_progress(self.portion.get_progress()?);
        Ok(())
    }

    pub fn scroll_down(&mut self, scrolls: usize) {
        self.portion.display_next = NextDisplay::ScrollDown(scrolls);
    }

    pub fn scroll_up(&mut self, scrolls: usize) {
        self.portion.display_next = NextDisplay::ScrollUp(scrolls);
    }

    pub fn is_local_source(&self) -> bool {
        matches!(self.book_info.source_data, BookSource::Local(_))
    }

    pub fn create(mut book: LibraryBookInfo) -> Result<Self, anyhow::Error> {
        match &mut book.source_data {
            BookSource::Local(data) => {
                let line = data.progress.get_line();

                let mut buffer = LocalBuffer::empty(data.path_to_book.clone());
                buffer.surround_line(line)?;
                let portion = BookPortion::empty();

                Ok(Self {
                    book_info: book,
                    portion: BookPortion::with_buffer(BufferType::Local(buffer), (line, 0)),
                })
            }
            BookSource::Global(data) => {
                todo!();
            }
        }
    }
}
