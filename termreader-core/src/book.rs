use crate::id::ID;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use termreader_sources::{
    novel::Novel,
    sources::{Scrape, Source, SourceID},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Book {
    id: ID,
    name: String,
    data: BookData,
    category: Option<String>,
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

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
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
        }
    }

    pub fn from_local_source(path: String) -> Result<Self> {
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
        })
    }

    pub fn is_local(&self) -> bool {
        matches!(self.data, BookData::Local(_))
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

    pub fn global_get_current_ch(&self) -> usize {
        match &self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.current_chapter,
        }
    }

    pub fn global_get_total_chs(&self) -> usize {
        match &self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.total_chapters,
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

    pub fn local_set_progress(&mut self, progress: ChapterProgress) {
        match &mut self.data {
            BookData::Local(p) => {
                p.progress = progress;
            }
            BookData::Global(_) => panic!("Function called on a book that is not sourced locally"),
        };
    }

    pub fn global_set_progress(&mut self, progress: ChapterProgress, chapter: usize) {
        match &mut self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.set_chapter_prog(progress, chapter),
        };
    }

    pub fn global_mark_ch_read(&mut self, chapter: usize) {
        match &mut self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.mark_chapter_complete(chapter),
        };
    }

    pub fn global_set_ch(&mut self, chapter: usize) {
        match &mut self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.set_chapter(chapter),
        };
    }

    /// Get the amount of chapters that have been read in order
    pub fn global_get_ordered_chapters(&self) -> usize {
        match &self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => d.get_ordered_chapters(),
        }
    }

    /// Get the chapter that should be read for the book to be read in order
    /// the difference between this and `global_get_ordered_chapters` is that
    /// this function will return the next chapter if it exists
    pub fn global_get_next_ordered_chap(&mut self) -> usize {
        match &self.data {
            BookData::Local(_) => panic!("Function called on a book that is not sourced globally"),
            BookData::Global(d) => {
                let last = d.get_ordered_chapters();
                if last == 0 && d.total_chapters == 0 {
                    panic!("No chapters! Unsure what to do")
                } else if last == 0 {
                    return 1;
                }

                let is_finished = d.get_chapter_prog(last) == ChapterProgress::Finished;
                // If the chapter's finished we want to start reading the next one (if it exists)
                if is_finished && d.total_chapters >= d.get_ordered_chapters() + 1 {
                    d.get_ordered_chapters() + 1
                } else {
                    d.get_ordered_chapters()
                }
            }
        }
    }

    pub fn display_info(&self) -> String {
        if let BookData::Global(data) = &self.data {
            format!(
                "{} | Chapters read: {}/{} ({:.2}%)",
                self.name.clone(),
                data.chapters_read_ordered,
                data.total_chapters,
                data.chapters_read_ordered as f64 / data.total_chapters as f64
            )
        } else {
            return self.name.clone();
        }
    }

    pub fn update(&mut self, source: &Source) {
        if let BookData::Global(ref mut data) = self.data {
            data.update(source)
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum BookData {
    Local(LocalData),
    Global(GlobalData),
}

#[derive(Serialize, Deserialize, Clone)]
struct LocalData {
    path: String,
    hash: String,
    progress: ChapterProgress,
    format: BookFormat,
}

impl LocalData {
    fn from_path(file_path: String) -> Result<Self> {
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

#[derive(Serialize, Deserialize, Clone, Copy)]
enum BookFormat {
    Unknown,
    Txt,
}

#[derive(Serialize, Deserialize, Clone)]
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

    fn update(&mut self, source: &Source) {
        let u = source.parse_novel_and_chapters(self.source_novel.get_url().to_string());
        if let Ok(updated) = u {
            self.total_chapters = updated.get_length();
            self.source_novel = updated;
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
