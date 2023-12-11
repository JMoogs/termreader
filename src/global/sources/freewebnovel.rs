use super::{
    get_html, Chapter, ChapterPreview, Novel, NovelPreview, NovelStatus, Scrape, SortOrder,
    SourceID,
};
use anyhow::Result;
// use chrono::Local;
// use html5ever::tree_builder::TreeSink;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FreeWebNovelScraper {
    pub source_id: SourceID,
}

impl FreeWebNovelScraper {
    pub fn new(source_id: SourceID) -> Self {
        Self { source_id }
    }
}

impl Scrape for FreeWebNovelScraper {
    fn get_popular(&self, _: SortOrder, page: usize) -> Result<Vec<NovelPreview>> {
        let url = format!("https://freewebnovel.com/completed-novel/{}", page);
        let html = Html::parse_document(&get_html(url)?);

        let mut novels = Vec::new();

        let novel_selector = Selector::parse(".li-row").unwrap();

        for selection in html.select(&novel_selector) {
            let name_selector = Selector::parse(".tit").unwrap();
            let novel_name = selection
                .select(&name_selector)
                .next()
                .map(|title| title.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();

            let url_selector = Selector::parse("h3 > a").unwrap();

            let novel_url = selection
                .select(&url_selector)
                .next()
                .and_then(|s| s.value().attr("href"))
                .unwrap();
            let novel_url = novel_url.replace(".html", "").get(1..).unwrap().to_string();

            let novel = NovelPreview::new(self.source_id, novel_name, novel_url);

            novels.push(novel);
        }

        Ok(novels)
    }

    fn parse_novel_and_chapters(&self, novel_path: String) -> Result<Novel> {
        let novel_url = novel_path.replace("/", "");
        let url = format!("https://freewebnovel.com/{}.html", novel_url);
        let html = Html::parse_document(&get_html(url.clone())?);

        let source = self.source_id;

        let source_name = String::from("FreeWebNovel");

        let full_url = url;

        let name = html
            .select(&Selector::parse("h1.tit").unwrap())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap()
            .trim()
            .to_string();

        let sel = Selector::parse("div.right > a").unwrap();
        let possible_authors_genres = html.select(&sel);

        let mut genres = String::new();
        let mut author = String::new();
        for selection in possible_authors_genres.into_iter() {
            let href = selection.attr("href").unwrap();
            if href.contains("/authors/") {
                author.push_str(&selection.inner_html())
            } else if href.contains("/genres/") {
                if genres.is_empty() {
                    genres.push_str(&selection.inner_html());
                } else {
                    genres.push_str(", ");
                    genres.push_str(&selection.inner_html());
                }
            }
        }

        let status = html
            .select(&Selector::parse("[title=\"Latest Release Novels\"]").unwrap())
            .next()
            .map(|content| content.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();
        let status = status.replace(|c| c == '\t' || c == '\n', ",");
        let status = match status.to_lowercase().as_str() {
            "ongoing" => NovelStatus::Ongoing,
            "completed" => NovelStatus::Completed,
            _ => NovelStatus::Unknown,
        };

        let sum_sel = Selector::parse(".inner > p").unwrap();
        let summary_line = html.select(&sum_sel);
        let mut summary = String::new();
        for line in summary_line {
            let line = line.inner_html();
            summary.push_str(&line);
            summary.push_str("\n\n");
        }

        let mut chapters = Vec::new();

        let latest_chapter_selector = Selector::parse("div.m-newest1 ul.ul-list5 a.con").unwrap();
        let latest_chap = html
            .select(&latest_chapter_selector)
            .next()
            .unwrap()
            .inner_html();
        let ch_no = Regex::new(r"\d+").unwrap();
        let latest_ch_no: usize = ch_no.find(&latest_chap).unwrap().as_str().parse().unwrap();

        for i in 1..=latest_ch_no {
            let ch = ChapterPreview {
                chapter_no: i,
                release_date: String::new(),
                name: format!("Chapter {}", i),
                url: format!("chapter-{}", i),
            };
            chapters.push(ch);
        }

        Ok(Novel {
            source,
            source_name,
            full_url,
            novel_url,
            name,
            author,
            status,
            genres,
            summary,
            chapters,
        })
    }

    fn parse_chapter(&self, novel_path: String, chapter_path: String) -> Result<Chapter> {
        let url = format!(
            "https://freewebnovel.com/{}/{}.html",
            novel_path, chapter_path
        );
        let html = Html::parse_document(&get_html(url)?);

        let chapter_name = html
            .select(&Selector::parse("h1.tit").unwrap())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap()
            .trim()
            .to_string();

        let ch_sel = Selector::parse("div.txt p").unwrap();
        let chapter_text = html.select(&ch_sel);
        let mut chapter_contents = String::new();
        for line in chapter_text {
            let line = line.inner_html().trim().to_string();
            if !line.starts_with("window.pubfuturetag") {
                chapter_contents.push_str(&line);
                chapter_contents.push_str("\n\n");
            }
        }
        let chapter_contents = chapter_contents.trim().to_string();

        return Ok(Chapter {
            source: self.source_id,
            novel_url: novel_path,
            chapter_url: chapter_path,
            chapter_name,
            chapter_contents,
        });
    }

    fn search_novels(&self, search_term: &str) -> Result<Vec<NovelPreview>> {
        let url = String::from("https://freewebnovel.com/search/");
        let search = search_term.to_string();
        let form = reqwest::blocking::multipart::Form::new().text("searchkey", search);
        let client = reqwest::blocking::Client::new();

        let response = client.post(url).multipart(form).send()?.text()?;

        let html = Html::parse_document(&response);

        let mut novels = Vec::new();

        let novel_sel = Selector::parse(".li-row > .li > .con").unwrap();

        for sel in html.select(&novel_sel) {
            let name = sel
                .select(&Selector::parse(".tit").unwrap())
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();

            let url = sel
                .select(&Selector::parse("h3 > a").unwrap())
                .next()
                .unwrap()
                .attr("href")
                .unwrap()
                .replace(".html", "")
                .get(1..)
                .unwrap()
                .to_string();

            novels.push(NovelPreview {
                source: self.source_id,
                name,
                url,
            })
        }

        return Ok(novels);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freewn_get_popular() {
        let source = FreeWebNovelScraper::new(1.into());
        let data = source.get_popular(SortOrder::Latest, 1).unwrap();
        println!("{data:?}");
    }

    #[test]
    fn freewn_get_novel() {
        let source = FreeWebNovelScraper::new(1.into());
        let data = source
            .parse_novel_and_chapters("my-three-wives-are-beautiful-vampires-novel".into())
            .unwrap();
        println!("{data:#?}");
    }

    #[test]
    fn freewn_parse_chapter() {
        let source = FreeWebNovelScraper::new(1.into());
        let data = source
            .parse_chapter(
                "my-three-wives-are-beautiful-vampires-novel".into(),
                "chapter-877".into(),
            )
            .unwrap();
        println!("{data:#?}");
    }

    #[test]
    fn freewn_search() {
        let source = FreeWebNovelScraper::new(1.into());
        let data = source.search_novels("my three").unwrap();
        println!("{data:#?}");
    }
}
