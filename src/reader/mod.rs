use crate::{
    appstate::{BookInfo, BookSource},
    global::sources::Chapter,
    reader::buffer::BufferType,
};

use self::buffer::{BookPortion, BookProgress, BookProgressData, LocalBuffer, NextDisplay};

use anyhow::Result;

pub mod buffer;
pub mod widget;

pub struct ReaderData {
    pub book_info: BookInfo,
    pub global_chapter: usize,
    pub portion: BookPortion,
}

impl ReaderData {
    pub fn get_progress(&self) -> Result<BookProgressData> {
        self.portion.get_progress()
    }

    pub fn set_progress(&mut self) -> Result<()> {
        if self.is_local_source() {
            self.book_info.set_progress(self.get_progress()?, None)
        } else {
            let prog = self.get_progress()?;
            if prog.progress == BookProgress::FINISHED {
                match self.book_info.get_source_data_mut() {
                    BookSource::Local(_) => unreachable!(),
                    BookSource::Global(d) => {
                        d.read_chapters.insert(self.global_chapter);
                    }
                }
            }
            self.book_info.set_progress(prog, Some(self.global_chapter))
        }
        Ok(())
    }

    pub fn scroll_down(&mut self, scrolls: usize) {
        self.portion.display_next = NextDisplay::ScrollDown(scrolls);
    }

    pub fn scroll_up(&mut self, scrolls: usize) {
        self.portion.display_next = NextDisplay::ScrollUp(scrolls);
    }

    pub fn is_local_source(&self) -> bool {
        self.book_info.is_local()
    }

    pub fn create(
        mut book: BookInfo,
        chapter_no: Option<usize>,
        text: Option<Chapter>,
    ) -> Result<Self, anyhow::Error> {
        match book.get_source_data_mut() {
            BookSource::Local(data) => {
                let line = data.progress.get_line();

                let mut buffer = LocalBuffer::empty(data.path_to_book.clone());
                buffer.surround_line(line)?;

                Ok(Self {
                    book_info: book,
                    portion: BookPortion::with_buffer(BufferType::Local(buffer), (line, 0)),
                    global_chapter: 0,
                })
            }
            BookSource::Global(data) => {
                let ch = chapter_no.unwrap();
                let line = match data.chapter_progress.get(&ch) {
                    Some(progress) => progress.get_line(),
                    None => 0,
                };

                let text = text.unwrap();
                let lines = text
                    .chapter_contents
                    .lines()
                    .map(|l| l.to_string())
                    .collect();

                let b = BufferType::Global(lines);

                Ok(Self {
                    book_info: book,
                    portion: BookPortion::with_buffer(b, (line, 0)),
                    global_chapter: ch,
                })
            }
        }
    }
}
