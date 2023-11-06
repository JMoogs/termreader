use anyhow::Result;

pub mod madara;

pub fn get_html<T: reqwest::IntoUrl>(url: T) -> Result<String> {
    let text = reqwest::blocking::get(url)?.text()?;

    if text.contains("Enable JavaScript and cookies to continue")
        || text.contains("Checking if the site connection is secure")
        || text.contains("Verify below to continue reading")
    {
        panic!("Cloudflare reached, error handling currently unimplemented.");
    } else {
        return Ok(text);
    };
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum NovelStatus {
    Ongoing,
    Completed,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct ChapterPreview {
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
    pub chatper_contents: String,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct SourceID(usize);

impl SourceID {
    fn new(id: usize) -> Self {
        Self(id)
    }
}

pub enum SortOrder {
    Latest,
    Rating,
}

pub trait Scrape {
    fn get_popular(&self, sort_order: SortOrder, page: String);
    fn parse_novel_and_chapters(&self, novel_path: String);
    fn parse_chapter(&self, novel_path: String, chapter_path: String);
    fn search_novels(&self);
}
