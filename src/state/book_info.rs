use crate::{
    global::sources::{source_data::Source, Novel, Scrape, SourceID},
    local::LocalBookData,
    reader::buffer::{BookProgress, BookProgressData},
    state::library::LibBookInfo,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

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
        let name = novel.name.clone();
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

    pub fn get_source_id(&self) -> Option<SourceID> {
        match self.get_source_data() {
            BookSource::Local(_) => None,
            BookSource::Global(d) => Some(d.novel.source),
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
    pub read_chapters: HashSet<usize>,
    pub current_chapter: usize,
    pub total_chapters: usize,
    pub chapter_progress: HashMap<usize, BookProgressData>,
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
            name: novel.name.clone(),
            read_chapters: HashSet::new(),
            current_chapter: ch,
            total_chapters: novel.chapters.len(),
            chapter_progress: HashMap::new(),
            novel,
        }
    }

    pub fn update(&mut self, source: &Source) -> Result<Novel> {
        source.parse_novel_and_chapters(self.novel.novel_url.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChapterDownloadData {
    pub location: String,
    pub chapter_progress: HashMap<usize, BookProgressData>,
}
