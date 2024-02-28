use crate::{
    local::LocalBookData,
    reader::buffer::{BookProgress, BookProgressData},
    state::library::LibBookInfo,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::SystemTime};
use termreader_sources::novel::Novel;
use termreader_sources::sources::{Scrape, Source, SourceID};

/// An ID used to uniquely identify a book.
/// Determined using the current timestamp, resulting in very little risk of collisions.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ID {
    id: u128,
}

impl ID {
    /// Generates an ID using system time.
    pub fn generate() -> Self {
        let now = SystemTime::now();

        let unix_timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time has gone VERY backwards.");

        Self {
            id: unix_timestamp.as_nanos(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookInfo {
    Library(LibBookInfo),
    Reader(ReaderBookInfo),
}

impl BookInfo {
    /// Creates an instance of `BookInfo` from a `Novel`
    pub fn from_novel_temp(novel: Novel, ch: usize) -> Result<Self, anyhow::Error> {
        let name = novel.get_name().to_string();
        let source = BookSource::Global(GlobalBookData::create(novel, ch));

        Ok(BookInfo::Reader(ReaderBookInfo {
            name,
            source_data: source,
        }))
    }

    pub fn is_local(&self) -> bool {
        match self {
            BookInfo::Library(d) => matches!(d.source_data, BookSource::Local(_)),
            BookInfo::Reader(d) => matches!(d.source_data, BookSource::Local(_)),
        }
    }

    pub fn get_progress(&self) -> BookProgress {
        let source_data = self.get_source_data();
        match source_data {
            BookSource::Local(ref data) => data.progress.progress,
            BookSource::Global(d) => {
                let p = d.chapter_progress.get(&d.current_chapter);
                match p {
                    Some(place) => place.progress,
                    None => BookProgress::Location((0, 0)),
                }
            }
        }
    }

    pub fn get_source_data_mut(&mut self) -> &mut BookSource {
        match self {
            BookInfo::Library(d) => &mut d.source_data,
            BookInfo::Reader(d) => &mut d.source_data,
        }
    }

    pub fn get_source_data(&self) -> &BookSource {
        match self {
            BookInfo::Library(d) => &d.source_data,
            BookInfo::Reader(d) => &d.source_data,
        }
    }

    pub fn get_source_id(&self) -> Option<&SourceID> {
        match self.get_source_data() {
            BookSource::Local(_) => None,
            BookSource::Global(d) => Some(d.novel.get_source()),
        }
    }

    pub fn get_novel(&self) -> Option<&Novel> {
        match self.get_source_data() {
            BookSource::Local(_) => None,
            BookSource::Global(d) => Some(&d.novel),
        }
    }

    pub fn update_progress(&mut self, progress: BookProgress, chapter: Option<usize>) {
        match self {
            BookInfo::Library(d) => match d.source_data {
                BookSource::Local(ref mut data) => data.progress.progress = progress,
                BookSource::Global(ref mut data) => {
                    let c = chapter.unwrap();
                    let entry = data.chapter_progress.get_mut(&c).unwrap();
                    entry.progress = progress;
                }
            },
            BookInfo::Reader(d) => match d.source_data {
                BookSource::Local(ref mut data) => {
                    data.progress.progress = progress;
                }
                BookSource::Global(ref mut data) => {
                    let c = chapter.unwrap();
                    let entry = data.chapter_progress.get_mut(&c).unwrap();
                    entry.progress = progress;
                }
            },
        }
    }

    pub fn set_progress(&mut self, progress: BookProgressData, chapter: Option<usize>) {
        match self {
            BookInfo::Library(d) => match d.source_data {
                BookSource::Local(ref mut data) => data.progress = progress,
                BookSource::Global(ref mut data) => {
                    data.chapter_progress.insert(chapter.unwrap(), progress);
                }
            },
            BookInfo::Reader(d) => match d.source_data {
                BookSource::Local(ref mut data) => {
                    data.progress = progress;
                }
                BookSource::Global(ref mut data) => {
                    data.chapter_progress.insert(chapter.unwrap(), progress);
                }
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReaderBookInfo {
    pub name: String,
    pub source_data: BookSource,
}

/// The type of source from which a book is provided. The variants each contain related info to the source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookSource {
    /// A locally sourced book.
    Local(LocalBookData),
    /// A book that is not sourced on the user's device.
    Global(GlobalBookData),
}

impl BookSource {
    /// Get the name of a book from its source.
    pub fn get_name(&self) -> String {
        match self {
            BookSource::Local(d) => d.name.clone(),
            BookSource::Global(d) => d.name.clone(),
        }
    }

    pub fn get_chapter(&self) -> usize {
        match self {
            BookSource::Local(_) => 0,
            BookSource::Global(d) => d.current_chapter,
        }
    }

    pub fn get_next_chapter(&self) -> Option<usize> {
        match self {
            BookSource::Local(_) => None,
            BookSource::Global(d) => {
                let next = d.current_chapter + 1;
                if next <= d.total_chapters {
                    Some(next)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_prev_chapter(&self) -> Option<usize> {
        match self {
            BookSource::Local(_) => None,
            BookSource::Global(d) => {
                if d.current_chapter <= 1 {
                    None
                } else {
                    Some(d.current_chapter - 1)
                }
            }
        }
    }

    pub fn set_chapter(&mut self, chapter: usize) {
        match self {
            BookSource::Local(_) => (),
            BookSource::Global(d) => d.current_chapter = chapter,
        }
    }

    /// Clears any progress on any chapters
    pub fn clear_chapter_data(&mut self) {
        match self {
            BookSource::Local(_) => (),
            BookSource::Global(d) => d.chapter_progress = HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalBookData {
    pub name: String,
    chapters_read_ordered: usize,
    current_chapter: usize,
    pub total_chapters: usize,
    chapter_progress: HashMap<usize, BookProgressData>,
    pub novel: Novel,
}

impl PartialEq for GlobalBookData {
    fn eq(&self, other: &Self) -> bool {
        self.novel == other.novel
    }
}

impl GlobalBookData {
    pub fn create(novel: Novel, ch: usize) -> Self {
        Self {
            name: novel.get_name().to_string(),
            chapters_read_ordered: 0,
            current_chapter: ch,
            total_chapters: novel.get_length(),
            chapter_progress: HashMap::new(),
            novel,
        }
    }

    pub fn update(&mut self, source: &Source) -> Result<Novel> {
        source.parse_novel_and_chapters(self.novel.get_url().to_string())
    }

    pub fn set_current_chapter(&mut self, chapter: usize) {
        self.current_chapter = chapter;
    }

    pub fn set_current_chapter_prog(&mut self, progress: BookProgressData) {
        self.chapter_progress.insert(self.current_chapter, progress);
    }

    pub fn get_current_chapter(&self) -> usize {
        self.current_chapter
    }

    pub fn mark_ch_complete(&mut self, chapter: usize) {
        if chapter <= self.total_chapters {
            self.chapter_progress
                .insert(chapter, BookProgressData::FINISHED);
            self.update_ordered_chapters()
        }
    }

    fn update_ordered_chapters(&mut self) {
        loop {
            let next_ch = self.chapter_progress.get(&(self.chapters_read_ordered + 1));
            if next_ch.is_none() {
                break;
            }
            if next_ch.unwrap().progress != BookProgress::FINISHED {
                break;
            }
            self.chapters_read_ordered += 1;
        }
    }

    pub fn get_chapters_ordered(&self) -> usize {
        self.chapters_read_ordered
    }

    pub fn get_chapter_progress(&self, chapter: usize) -> Option<BookProgressData> {
        self.chapter_progress.get(&chapter).cloned()
    }

    pub fn set_chapter_progress(&mut self, chapter: usize, progress: BookProgressData) {
        if chapter <= self.total_chapters {
            self.chapter_progress.insert(chapter, progress);
        }
    }

    pub fn clear_ch_progress(&mut self, chapter: usize) {
        self.chapter_progress.remove(&chapter);
    }

    pub fn clear_all_ch_progress(&mut self) {
        self.chapter_progress = HashMap::new();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChapterDownloadData {
    pub location: String,
    pub chapter_progress: HashMap<usize, BookProgressData>,
}
