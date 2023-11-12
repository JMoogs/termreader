use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod madara;
pub mod source_data;

pub fn get_html<T: reqwest::IntoUrl>(url: T) -> Result<String> {
    // let c = Client::builder().build()?.get(url).send()
    let c = reqwest::blocking::ClientBuilder::new().user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Mobile Safari/537.36");
    let client = c.build()?;
    let text = client.get(url).send()?.text()?;

    if text.contains("Enable JavaScript and cookies to continue")
        || text.contains("Checking if the site connection is secure")
        || text.contains("Verify below to continue reading")
    {
        panic!("Cloudflare reached, error handling currently unimplemented.");
    } else {
        return Ok(text);
    };
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NovelStatus {
    Ongoing,
    Completed,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Novel {
    pub source: SourceID,
    pub source_name: String,
    pub full_url: String,
    pub novel_url: String,
    pub name: String,
    pub author: String,
    pub status: NovelStatus,
    pub genres: String,
    pub summary: String,
    pub chapters: Vec<ChapterPreview>,
}

impl Novel {
    pub fn synopsis(&self) -> String {
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

    pub fn get_chapter_url(&self, chapter: usize) -> Option<String> {
        for ch in &self.chapters {
            if ch.chapter_no == chapter {
                return Some(ch.url.clone());
            }
        }
        return None;
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChapterPreview {
    pub chapter_no: usize,
    pub release_date: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Chapter {
    pub source: SourceID,
    pub novel_url: String,
    pub chapter_url: String,
    pub chapter_name: String,
    pub chapter_contents: String,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NovelPreview {
    pub source: SourceID,
    pub name: String,
    pub url: String,
}

impl NovelPreview {
    pub fn new(source: SourceID, name: String, url: String) -> Self {
        Self { source, name, url }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SourceID(usize);

impl SourceID {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

pub enum SortOrder {
    Latest,
    Rating,
}

pub trait Scrape {
    fn get_popular(&self, sort_order: SortOrder, page: usize) -> Result<Vec<NovelPreview>>;
    fn parse_novel_and_chapters(&self, novel_path: String) -> Result<Novel>;
    fn parse_chapter(&self, novel_path: String, chapter_path: String) -> Result<Chapter>;
    fn search_novels(&self, search_term: &str) -> Result<Vec<NovelPreview>>;
}
