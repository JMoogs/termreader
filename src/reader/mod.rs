use crate::{
    appstate::{BookInfo, BookSource},
    global::sources::Scrape,
    reader::buffer::BufferType,
};

use self::buffer::{BookPortion, BookProgressData, LocalBuffer, NextDisplay};

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
        // match self.book_info {
        //     BookInfo::Library(info) => {
        //         // if info.is_local() {
        //         //     self.book_info.set_progress(self.get_progress()?, None)
        //         // } else {
        //         //     let ch = self.book_info.
        //         // }
        //         match info.source_data {
        //             BookSource::Local(_) => self.book_info.set_progress(self.get_progress()?, None),
        //             BookSource::Global(_) => self
        //                 .book_info
        //                 .set_progress(self.get_progress()?, Some(self.global_chapter)),
        //         }
        //     }
        //     BookInfo::Reader(info) => match info.source_data {
        //         BookSource::Local(_) => self.book_info.set_progress(self.get_progress()?, None),
        //         BookSource::Global(_) => self
        //             .book_info
        //             .set_progress(self.get_progress()?, Some(self.global_chapter)),
        //     },
        // }
        if self.is_local_source() {
            self.book_info.set_progress(self.get_progress()?, None)
        } else {
            self.book_info
                .set_progress(self.get_progress()?, Some(self.global_chapter))
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
        chapter: Option<usize>,
        source: Option<&Box<dyn Scrape>>,
    ) -> Result<Self, anyhow::Error> {
        match book.get_source_data() {
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
                let ch = chapter.unwrap();
                let line = match data.chapter_progress.get(&ch) {
                    Some(progress) => progress.get_line(),
                    None => 0,
                };

                let source = source.unwrap();
                let text = source.parse_chapter(
                    data.novel.novel_url.clone(),
                    data.novel.get_chapter_url(ch).unwrap(),
                )?;

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
