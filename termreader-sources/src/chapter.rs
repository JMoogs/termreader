use crate::sources::SourceID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct ChapterPreview {
    pub(crate) chapter_no: usize,
    pub(crate) release_date: String,
    pub(crate) name: String,
    pub(crate) url: String,
}

impl ChapterPreview {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn get_chapter_no(&self) -> usize {
        self.chapter_no
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Chapter {
    pub(crate) source: SourceID,
    pub(crate) novel_url: String,
    pub(crate) chapter_url: String,
    pub(crate) chapter_name: String,
    pub(crate) chapter_contents: String,
}

impl Chapter {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.chapter_name
    }

    #[inline]
    pub fn get_contents(&self) -> &str {
        &self.chapter_contents
    }
}
