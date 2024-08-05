use crate::TRError;
use crate::{id::ID, updates::UpdatedChapters, Context};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::Path;
use std::{collections::HashMap, rc::Rc};
use termreader_sources::{
    chapter::ChapterPreview,
    novel::Novel,
    sources::{Scrape, Source, SourceID},
};

/// A reference to a `Book`
#[derive(Serialize, Deserialize, Debug)]
pub struct BookRef(pub(crate) Rc<RefCell<Book>>);

impl Clone for BookRef {
    /// Clones the `BookRef`. This is a shallow clone
    fn clone(&self) -> Self {
        BookRef(Rc::clone(&self.0))
    }
}

impl BookRef {
    /// Returns the ID of the `Book`
    pub fn get_id(&self) -> ID {
        self.0.borrow().get_id()
    }

    /// Returns the name of the `Book`
    pub fn get_name(&self) -> String {
        self.0.borrow().get_name().to_string()
    }

    /// Returns true if the `Book` is in the library
    pub fn in_library(&self) -> bool {
        self.0.borrow().in_library
    }

    /// Get the url (just the book-specific part) of the referenced `Book`
    ///
    /// Returns `None` if the book is locally sourced
    pub fn get_url(&self) -> Option<String> {
        Some(self.0.borrow().get_url()?.to_string())
    }

    /// Get the url for a specific chapter of the referenced `Book`
    ///
    /// Returns `None` if the book is locally sourced
    pub fn get_chapter_url(&self, chapter: usize) -> Option<String> {
        Some(self.0.borrow().get_chapter_url(chapter)?.to_string())
    }

    /// Get the full url (website and book parts) of the referenced `Book`
    ///
    /// Returns `None` if the book is locally sourced
    pub fn get_full_url(&self) -> Option<String> {
        Some(self.0.borrow().get_full_url()?.to_string())
    }

    /// Renames the referenced `Book`
    pub fn rename(&mut self, new_name: String) {
        self.0.borrow_mut().rename(new_name.clone());
    }

    /// Resets all progress related to the `Book`
    pub fn reset_progress(&mut self) {
        self.0.borrow_mut().reset_progress()
    }

    /// Returns true if a book is locally sourced
    pub fn is_local(&self) -> bool {
        self.0.borrow().is_local()
    }

    /// Returns true if a book is not locally sourced
    pub fn is_global(&self) -> bool {
        self.0.borrow().is_global()
    }

    /// Returns the progress for the chapter currently being read
    pub fn get_current_ch_progress(&self) -> ChapterProgress {
        self.0.borrow().get_current_ch_progress()
    }

    pub fn get_all_chapter_progress(&self) -> HashMap<usize, ChapterProgress> {
        self.0.borrow().get_all_ch_progress()
    }

    /// Returns the current chapter
    ///
    /// Returns `None` when called on a locally sourced book
    pub fn get_current_ch(&self) -> Option<usize> {
        self.0.borrow().get_current_ch()
    }

    /// Returns the total chapter count
    ///
    /// Returns `None` when called on a locally sourced book
    pub fn get_total_ch_count(&self) -> Option<usize> {
        self.0.borrow().get_total_chs()
    }

    /// Sets the progress for a chapter of a global book
    ///
    /// Errors when called on a locally sourced book
    pub fn global_set_progress(
        &mut self,
        progress: ChapterProgress,
        chapter: usize,
    ) -> Result<(), TRError> {
        self.0.borrow_mut().global_set_progress(progress, chapter)
    }

    /// Marks a chapter as read for a global book
    ///
    /// Errors when called on a locally sourced book
    pub fn global_mark_ch_read(&mut self, chapter: usize) -> Result<(), TRError> {
        self.0.borrow_mut().global_mark_ch_read(chapter)
    }

    /// Sets the current chapter for a global book
    ///
    /// Errors when called on a locally sourced book,
    /// or when the set chapter is outside of the chapter range
    pub fn global_set_chapter(&mut self, chapter: usize) -> Result<(), TRError> {
        let mut b = self.0.borrow_mut();

        let total_chapters = b.get_total_chs().ok_or_else(|| {
            TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            ))
        })?;

        if chapter <= total_chapters {
            b.global_set_ch(chapter)
                .expect("the invariant was checked earlier");
            Ok(())
        } else {
            Err(TRError::InvalidArgument(String::from(
                "chapter outside of range",
            )))
        }
    }

    /// Returns the amount of chapters that have been read in order from the start
    ///
    /// Errors when called on a locally sourced book
    pub fn global_get_ordered_chapters(&self) -> Result<usize, TRError> {
        self.0.borrow().global_get_ordered_chapters()
    }

    /// Get the chapter that should be read for the book to be read in order
    /// the difference between this and `global_get_ordered_chapters` is that
    /// this function will return the next chapter if it exists
    ///
    /// Returns `None` if a book has no chapters, or is sourced locally
    pub fn global_get_next_ordered_chap(&mut self) -> Option<usize> {
        self.0.borrow_mut().global_get_next_ordered_chap()
    }

    /// Returns a global novels original name
    pub fn global_get_original_name(&self) -> Result<String, TRError> {
        if self.is_global() {
            Ok(self.0.borrow().global_get_novel().get_name().to_string())
        } else {
            Err(TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            )))
        }
    }

    /// Returns the synopsis of a book
    pub fn get_synopsis(&self) -> String {
        if self.is_local() {
            unimplemented!()
        } else {
            self.0.borrow().global_get_novel().get_synopsis()
        }
    }

    /// Returns the chapters a book has
    ///
    /// Errors when called on a locally sourced book
    pub fn get_chapters(&self) -> Result<Vec<ChapterPreview>, TRError> {
        match self.0.borrow().get_chapters() {
            Some(chs) => Ok(chs.clone()),
            None => Err(TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            ))),
        }
    }

    /// Get a copy of the `Book` that is referenced
    pub fn get_book(self) -> Book {
        self.0.borrow().clone()
    }

    // TODO: Remove this function as it shouldn't be implemented in core
    pub fn get_display_info(&self) -> String {
        self.0.borrow().display_info()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Book {
    id: ID,
    name: String,
    data: BookData,
    pub(crate) in_library: bool,
    pub(crate) in_history: bool,
    pub(crate) in_updates: bool,
    pub(crate) category: Option<String>,
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Book {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_chapters(&self) -> Option<&Vec<ChapterPreview>> {
        if self.is_local() {
            None
        } else {
            Some(self.global_get_novel().get_chapters())
        }
    }

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name.clone();
        if self.is_global() {
            let n = self.global_get_novel_mut();
            n.set_alias(new_name)
        }
    }

    pub fn get_id(&self) -> ID {
        self.id
    }

    pub fn from_novel(novel: Novel) -> Self {
        Self {
            id: ID::generate(),
            name: novel.get_name().to_string(),
            data: BookData::Global(GlobalData::from_novel(novel)),
            category: None,
            in_library: false,
            in_history: false,
            in_updates: false,
        }
    }

    pub fn from_local_source(path: String) -> Result<Self, TRError> {
        let p = Path::new(&path);
        let id = ID::generate();
        let name = p.file_stem().unwrap().to_str().unwrap().to_string();
        let category = None;

        let data = BookData::Local(LocalData::from_path(path)?);

        Ok(Self {
            id,
            name,
            data,
            category,
            in_library: false,
            in_history: false,
            in_updates: false,
        })
    }

    pub fn is_local(&self) -> bool {
        matches!(self.data, BookData::Local(_))
    }

    /// Returns true if a book is not locally sourced.
    pub fn is_global(&self) -> bool {
        matches!(self.data, BookData::Global(_))
    }

    pub fn get_current_ch_progress(&self) -> ChapterProgress {
        match &self.data {
            BookData::Local(d) => d.progress,
            BookData::Global(d) => {
                let progress = d.chapter_progress.get(&d.current_chapter);
                match progress {
                    Some(p) => *p,
                    None => ChapterProgress::Location((0, 0)),
                }
            }
        }
    }

    /// Get the progress for all chapters
    /// Returns a HashMap of size 1 with chapter 0 if called on a local book
    pub fn get_all_ch_progress(&self) -> HashMap<usize, ChapterProgress> {
        match &self.data {
            BookData::Local(d) => HashMap::from([(0, d.progress)]),
            BookData::Global(d) => d.chapter_progress.clone(),
        }
    }

    pub fn get_current_ch(&self) -> Option<usize> {
        match &self.data {
            BookData::Local(_) => None,
            BookData::Global(d) => Some(d.current_chapter),
        }
    }

    pub fn get_total_chs(&self) -> Option<usize> {
        match &self.data {
            BookData::Local(_) => None,
            BookData::Global(d) => Some(d.total_chapters),
        }
    }

    pub fn global_get_source_id(&self) -> SourceID {
        match &self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.source_novel.get_source(),
        }
    }

    pub fn global_get_novel(&self) -> &Novel {
        match &self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => &d.source_novel,
        }
    }

    pub fn global_get_novel_mut(&mut self) -> &mut Novel {
        match &mut self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => &mut d.source_novel,
        }
    }

    pub fn local_set_progress(&mut self, progress: ChapterProgress) {
        match &mut self.data {
            BookData::Local(p) => {
                p.progress = progress;
            }
            BookData::Global(_) => panic!("Function called on a book that is not sourced locally"),
        };
    }

    pub fn global_set_progress(
        &mut self,
        progress: ChapterProgress,
        chapter: usize,
    ) -> Result<(), TRError> {
        match &mut self.data {
            BookData::Local(_) => Err(TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            ))),
            BookData::Global(d) => {
                d.set_chapter_prog(progress, chapter);
                Ok(())
            }
        }
    }

    pub fn global_mark_ch_read(&mut self, chapter: usize) -> Result<(), TRError> {
        match &mut self.data {
            BookData::Local(_) => Err(TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            ))),
            BookData::Global(d) => {
                d.mark_chapter_complete(chapter);
                Ok(())
            }
        }
    }

    pub fn global_set_ch(&mut self, chapter: usize) -> Result<(), TRError> {
        match &mut self.data {
            BookData::Local(_) => Err(TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            ))),
            BookData::Global(d) => {
                d.set_chapter(chapter);
                Ok(())
            }
        }
    }

    /// Get the amount of chapters that have been read in order
    pub fn global_get_ordered_chapters(&self) -> Result<usize, TRError> {
        match &self.data {
            BookData::Local(_) => Err(TRError::BadUse(String::from(
                "supplied a local book where a global book should have been supplied",
            ))),
            BookData::Global(d) => Ok(d.get_ordered_chapters()),
        }
    }

    /// Get the url of a book, returning none if the book is locally sourced
    pub fn get_url(&self) -> Option<&str> {
        if self.is_global() {
            Some(self.global_get_novel().get_url())
        } else {
            None
        }
    }

    /// Get the full url of a book, returning none if the book is locally sourced
    pub fn get_full_url(&self) -> Option<&str> {
        if self.is_global() {
            Some(self.global_get_novel().get_full_url())
        } else {
            None
        }
    }

    /// Get the url of a book, returning none if the book is locally sourced,
    /// or if the chapter does not exist
    pub fn get_chapter_url(&self, chapter: usize) -> Option<&str> {
        if self.is_global() {
            self.global_get_novel().get_chapter_url(chapter)
        } else {
            None
        }
    }

    /// Get the chapter that should be read for the book to be read in order
    /// the difference between this and `global_get_ordered_chapters` is that
    /// this function will return the next chapter if it exists
    ///
    /// Returns `None` if a book has no chapters, or is sourced locally
    pub fn global_get_next_ordered_chap(&mut self) -> Option<usize> {
        match &self.data {
            BookData::Local(_) => None,
            BookData::Global(d) => {
                let last = d.get_ordered_chapters();
                if last == 0 && d.total_chapters == 0 {
                    return None;
                } else if last == 0 {
                    return Some(1);
                }

                let is_finished = d.get_chapter_prog(last) == ChapterProgress::Finished;
                // If the chapter's finished we want to start reading the next one (if it exists)
                if is_finished && d.total_chapters >= d.get_ordered_chapters() + 1 {
                    Some(d.get_ordered_chapters() + 1)
                } else {
                    Some(d.get_ordered_chapters())
                }
            }
        }
    }

    pub fn display_info(&self) -> String {
        if let BookData::Global(data) = &self.data {
            let pct =
                if (data.chapters_read_ordered as f64 / data.total_chapters as f64).is_finite() {
                    100.0 * (data.chapters_read_ordered as f64 / data.total_chapters as f64)
                } else {
                    100.0
                };
            format!(
                "{} | Chapters read: {}/{} ({:.2}%)",
                self.name.clone(),
                data.chapters_read_ordered,
                data.total_chapters,
                pct,
            )
        } else {
            return self.name.clone();
        }
    }

    pub fn reset_progress(&mut self) {
        match &mut self.data {
            BookData::Local(d) => d.progress = ChapterProgress::Word((0, 0)),
            BookData::Global(d) => {
                d.chapters_read_ordered = 0;
                d.current_chapter = 1;
                d.chapter_progress = HashMap::new();
            }
        }
    }

    pub fn get_source(&self, ctx: &Context) -> Source {
        let s_id = self.global_get_source_id();
        ctx.sources.get_source_by_id(s_id).unwrap().clone()
    }

    pub fn update(&mut self, source: &Source) -> UpdatedChapters {
        if let BookData::Global(ref mut data) = self.data {
            data.update(source)
        } else {
            UpdatedChapters::None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum BookData {
    Local(LocalData),
    Global(GlobalData),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LocalData {
    path: String,
    hash: String,
    progress: ChapterProgress,
    format: BookFormat,
}

impl LocalData {
    fn from_path(file_path: String) -> Result<Self, TRError> {
        let path = Path::new(&file_path);

        let contents = std::fs::read(path)?;
        let hash = sha256::digest(contents);

        let progress = ChapterProgress::Location((0, 0));

        let format = match path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "txt" => BookFormat::Txt,
                _ => BookFormat::Unknown,
            },
            None => BookFormat::Unknown,
        };

        Ok(Self {
            path: file_path,
            hash,
            progress,
            format,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
enum BookFormat {
    Unknown,
    Txt,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GlobalData {
    chapters_read_ordered: usize,
    current_chapter: usize,
    total_chapters: usize,
    chapter_progress: HashMap<usize, ChapterProgress>,
    source_novel: Novel,
}

impl PartialEq for GlobalData {
    fn eq(&self, other: &Self) -> bool {
        self.source_novel == other.source_novel
    }
}

impl GlobalData {
    fn from_novel(novel: Novel) -> Self {
        Self {
            chapters_read_ordered: 0,
            current_chapter: 1,
            total_chapters: novel.get_length(),
            chapter_progress: HashMap::new(),
            source_novel: novel,
        }
    }

    fn get_chapter(&self) -> usize {
        self.current_chapter
    }

    fn set_chapter(&mut self, chapter: usize) {
        if chapter <= self.total_chapters {
            self.current_chapter = chapter
        }
    }

    fn clear_all_chapter_progress(&mut self) {
        self.chapter_progress = HashMap::new();
    }

    fn clear_chapter_progress(&mut self, chapter: usize) {
        self.chapter_progress.remove(&chapter);
    }

    fn set_chapter_prog(&mut self, progress: ChapterProgress, chapter: usize) {
        self.chapter_progress.insert(chapter, progress);
        self.update_ordered_chapters();
    }

    fn get_chapter_prog(&self, chapter: usize) -> ChapterProgress {
        let ch = self.chapter_progress.get(&chapter);
        if let Some(p) = ch {
            *p
        } else {
            ChapterProgress::Location((0, 0))
        }
    }

    fn mark_chapter_complete(&mut self, chapter: usize) {
        self.chapter_progress
            .insert(chapter, ChapterProgress::Finished);
        self.update_ordered_chapters();
    }

    fn update_ordered_chapters(&mut self) {
        loop {
            let next_ch = self.chapter_progress.get(&(self.chapters_read_ordered + 1));
            if next_ch.is_none() {
                break;
            }
            if *next_ch.unwrap() != ChapterProgress::Finished {
                break;
            }
            self.chapters_read_ordered += 1;
        }
    }

    fn get_ordered_chapters(&self) -> usize {
        self.chapters_read_ordered
    }

    fn update(&mut self, source: &Source) -> UpdatedChapters {
        let u = source.parse_novel_and_chapters(self.source_novel.get_url().to_string());
        if let Ok(updated) = u {
            let current_length = self.total_chapters;
            let updated_length = updated.get_length();

            // Update the details
            self.total_chapters = updated_length;
            self.source_novel = updated;

            if updated_length - current_length == 0 {
                return UpdatedChapters::None;
            } else if updated_length - current_length == 1 {
                return UpdatedChapters::Single(updated_length);
            } else {
                return UpdatedChapters::Range((current_length + 1, updated_length));
            }
        } else {
            UpdatedChapters::None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ChapterProgress {
    /// A line number and character
    Location((usize, usize)),
    /// A starting and ending word
    Word((usize, usize)),
    Finished,
}
