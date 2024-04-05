use crate::sources::SourceID;
use crate::ChapterPreview;
use serde::{Deserialize, Serialize};

#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum NovelStatus {
    #[default]
    Unknown,
    Ongoing,
    Completed,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct NovelPreview {
    pub(crate) source: SourceID,
    pub(crate) name: String,
    pub(crate) url: String,
}

impl NovelPreview {
    pub fn new(source: SourceID, name: String, url: String) -> Self {
        Self { source, name, url }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct Novel {
    pub(crate) source: SourceID,
    pub(crate) source_name: String,
    pub(crate) full_url: String,
    pub(crate) novel_url: String,
    pub(crate) name: String,
    pub(crate) author: String,
    pub(crate) status: NovelStatus,
    pub(crate) genres: String,
    pub(crate) summary: String,
    pub(crate) chapters: Vec<ChapterPreview>,
}

impl Novel {
    pub fn get_source(&self) -> SourceID {
        self.source
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_synopsis(&self) -> String {
        let status = match self.status {
            NovelStatus::Ongoing => "Ongoing",
            NovelStatus::Completed => "Completed",
            NovelStatus::Unknown => "Unknown",
        };

        format!(
            "Author: {}\nStatus: {}\nGenres: {}\n\n{}",
            self.author, status, self.genres, self.summary
        )
    }

    pub fn get_chapters(&self) -> &Vec<ChapterPreview> {
        &self.chapters
    }

    pub fn get_length(&self) -> usize {
        self.chapters.len()
    }

    // TODO: Index it rather than looping through
    pub fn get_chapter_url(&self, chapter: usize) -> Option<String> {
        for ch in &self.chapters {
            if ch.chapter_no == chapter {
                return Some(ch.url.clone());
            }
        }
        None
    }

    pub fn get_url(&self) -> &str {
        &self.novel_url
    }

    pub fn get_full_url(&self) -> &str {
        &self.full_url
    }
}
