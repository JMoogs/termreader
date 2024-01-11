use super::{
    get_html, Chapter, ChapterPreview, Novel, NovelPreview, NovelStatus, Scrape, SortOrder,
    SourceID,
};
use anyhow::Result;
use chrono::Local;
use html5ever::tree_builder::TreeSink;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
// https://github.com/LNReader/lnreader/blob/main/src/sources/multisrc/madara/MadaraScraper.js

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MadaraScraper {
    pub source_id: SourceID,
    pub base_url: String,
    pub source_name: String,
    pub path: Option<MadaraPaths>,
    pub new_chap_endpoint: bool,
}

impl Scrape for MadaraScraper {
    fn get_popular(&self, sort_order: SortOrder, page: usize) -> Result<Vec<NovelPreview>> {
        self.get_popular(sort_order, page)
    }

    fn parse_novel_and_chapters(&self, novel_path: String) -> Result<Novel> {
        self.parse_novel_and_chapters(novel_path)
    }

    fn parse_chapter(&self, novel_path: String, chapter_path: String) -> Result<Chapter> {
        self.parse_chapter(novel_path, chapter_path)
    }

    fn search_novels(&self, search_term: &str) -> Result<Vec<NovelPreview>> {
        self.search_novels(search_term)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MadaraPaths {
    novels: String,
    novel: String,
    chapter: String,
}

impl MadaraPaths {
    pub fn new(
        novels: impl Into<String>,
        novel: impl Into<String>,
        chapter: impl Into<String>,
    ) -> Self {
        Self {
            novels: novels.into(),
            novel: novel.into(),
            chapter: chapter.into(),
        }
    }
}

impl Default for MadaraPaths {
    fn default() -> Self {
        Self {
            novels: String::from("novel"),
            novel: String::from("novel"),
            chapter: String::from("novel"),
        }
    }
}

impl MadaraScraper {
    pub fn new(
        source_id: SourceID,
        base_url: String,
        source_name: String,
        path: Option<MadaraPaths>,
        new_chap_endpoint: bool,
    ) -> Self {
        Self {
            source_id,
            base_url,
            source_name,
            path,
            new_chap_endpoint,
        }
    }

    fn get_popular(&self, order: SortOrder, page: usize) -> Result<Vec<NovelPreview>> {
        let sort_order = match order {
            SortOrder::Latest => "?m_orderby=latest",
            SortOrder::Rating => "?m_orderby=rating",
        };

        let url_path = &self.path.clone().unwrap_or_default().novels;

        let url = format!("{}{}/page/{}{}", self.base_url, url_path, page, sort_order);

        let mut html = Html::parse_document(&get_html(url)?);

        let selector = Selector::parse(".manga-title-badges").unwrap();
        let ids: Vec<_> = html.select(&selector).map(|x| x.id()).collect();
        for id in ids {
            html.remove_from_parent(&id);
        }

        let mut novels = Vec::new();

        let novel_selector = Selector::parse(".page-item-detail").unwrap();

        for selection in html.select(&novel_selector) {
            let name_selector = Selector::parse(".post-title").unwrap();
            let novel_name = selection
                .select(&name_selector)
                .next()
                .map(|title| title.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();

            let a_selector = Selector::parse("a").unwrap();
            let url = selection
                .select(&name_selector)
                .next()
                .and_then(|t| t.select(&a_selector).next())
                .and_then(|a| a.value().attr("href"));

            let novel_url = if let Some(n_url) = url {
                n_url.split('/').collect::<Vec<&str>>()[4]
            } else {
                panic!("Scraper outdated");
            };

            novels.push(NovelPreview::new(
                self.source_id,
                novel_name,
                novel_url.into(),
            ));
        }

        Ok(novels)
    }

    fn parse_novel_and_chapters(&self, novel_path: String) -> Result<Novel> {
        let url = &self.path.clone().unwrap_or_default().novel;
        let url = format!("{}{}/{}/", self.base_url, url, novel_path);

        let mut html = Html::parse_document(&get_html(&url)?);

        let selector = Selector::parse(".manga-title-badges").unwrap();
        let ids: Vec<_> = html.select(&selector).map(|x| x.id()).collect();
        for id in ids {
            html.remove_from_parent(&id);
        }

        let name_selector = Selector::parse(".post-title").unwrap();
        let novel_name = html
            .select(&name_selector)
            .next()
            .map(|title| title.text().collect::<String>())
            .unwrap()
            .trim()
            .to_string();

        let detail_sel = Selector::parse(".post-content_item, .post-content").unwrap();

        let mut genres = String::new();
        let mut author = String::new();
        let mut status = NovelStatus::Unknown;

        for selection in html.select(&detail_sel) {
            let detail_name = selection
                .select(&Selector::parse("h5").unwrap())
                .next()
                .map(|h5| h5.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();

            let detail = selection
                .select(&Selector::parse(".summary-content").unwrap())
                .next()
                .map(|content| content.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();

            match detail_name.as_str() {
                "Genre(s)" => genres = detail.replace(|c| c == '\t' || c == '\n', ","),
                "Author(s)" => author = detail,
                "Status" => {
                    status = if detail.contains("OnGoing") {
                        NovelStatus::Ongoing
                    } else {
                        NovelStatus::Completed
                    }
                }
                _ => (),
            }
        }

        let selector = Selector::parse("div.summary__content .code-block,script").unwrap();
        let ids: Vec<_> = html.select(&selector).map(|x| x.id()).collect();
        for id in ids {
            html.remove_from_parent(&id);
        }

        let summary = html
            .select(&Selector::parse("div.summary__content").unwrap())
            .next()
            .map(|sum| sum.text().collect::<String>())
            .unwrap()
            .trim()
            .to_string();

        let mut chapters = Vec::new();

        let client = reqwest::blocking::Client::new();
        let response = if self.new_chap_endpoint {
            let url = reqwest::Url::parse(&format!("{}{}", url, "ajax/chapters/")).unwrap();
            client.post(url).send()?.text()?
        } else {
            let selector_1 = Selector::parse(".rating-post-id").unwrap();
            let selector_2 = Selector::parse("#manga-chapters-holder").unwrap();

            let mut novel_id = html
                .select(&selector_1)
                .next()
                .and_then(|element| element.value().attr("value"))
                .unwrap_or_default();

            if novel_id.is_empty() {
                novel_id = html
                    .select(&selector_2)
                    .next()
                    .and_then(|element| element.value().attr("data-id"))
                    .unwrap_or_default()
            }

            let url =
                reqwest::Url::parse(&format!("{}{}", self.base_url, "wp-admin/admin-ajax.php"))
                    .unwrap();

            let form = reqwest::blocking::multipart::Form::new()
                .text("action", "manga_get_chapters")
                .text("manga", novel_id.to_string());

            client.post(url).multipart(form).send()?.text()?
        };

        let html = Html::parse_document(&response);

        let selector = Selector::parse(".wp-manga-chapter").unwrap();
        for selection in html.select(&selector) {
            let a_selector = Selector::parse("a").unwrap();
            let chapter_name = selection
                .select(&a_selector)
                .next()
                .map(|name| name.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();
            let chapter_name = if chapter_name.is_empty() {
                String::from("[No Name Provided]")
            } else {
                chapter_name
            };

            let mut release_date = html
                .select(&Selector::parse("span.chapter-release-date").unwrap())
                .next()
                .map(|d| d.text().collect::<String>())
                .unwrap_or_default()
                .trim()
                .to_string();

            let release_date = if release_date.contains("ago") {
                let now = Local::now();

                let re = Regex::new(r"\d+").unwrap();
                if let Some(capture) = re.find(&release_date) {
                    let time_ago: i64 = capture.as_str().parse().unwrap();

                    if release_date.contains("hours ago") || release_date.contains("hour ago") {
                        release_date = (now - chrono::Duration::hours(time_ago))
                            .format("%Y-%m-%d")
                            .to_string();
                    }

                    if release_date.contains("days ago") || release_date.contains("day ago") {
                        release_date = (now - chrono::Duration::days(time_ago))
                            .format("%Y-%m-%d")
                            .to_string();
                    }

                    if release_date.contains("months ago") || release_date.contains("month ago") {
                        release_date = (now - chrono::Duration::days(30 * time_ago))
                            .format("%Y-%m-%d")
                            .to_string();
                    }
                }

                release_date
            } else {
                release_date
            };

            let chapter_url = selection
                .select(&Selector::parse("a").unwrap())
                .next()
                .and_then(|a| a.value().attr("href"));

            let chapter_url = if let Some(u) = chapter_url {
                let parts = u.split('/').collect::<Vec<&str>>();
                match parts.get(6) {
                    Some(p) => format!("{}/{}", parts[5], p),
                    None => parts[5].to_string(),
                }
            } else {
                panic!("Scraper outdated")
            };

            chapters.push((release_date, chapter_name, chapter_url));
        }

        let chapters = chapters.into_iter().rev();
        let mut c = Vec::new();
        for (i, chapter) in chapters.enumerate() {
            let preview = ChapterPreview {
                chapter_no: i + 1,
                release_date: chapter.0,
                name: chapter.1,
                url: chapter.2,
            };
            c.push(preview);
        }

        Ok(Novel {
            source: self.source_id,
            source_name: self.source_name.clone(),
            full_url: url,
            novel_url: novel_path,
            name: novel_name,
            author,
            status,
            genres,
            summary,
            chapters: c,
        })
    }

    fn parse_chapter(&self, novel_path: String, chapter_path: String) -> Result<Chapter> {
        let url = format!(
            "{}{}/{}/{}",
            self.base_url,
            self.path.clone().unwrap_or_default().chapter,
            novel_path,
            chapter_path
        );

        let html = Html::parse_document(&get_html(url)?);

        let mut chapter_name = html
            .select(&Selector::parse(".text-center").unwrap())
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        if chapter_name.is_empty() {
            chapter_name = html
                .select(&Selector::parse("#chapter-heading").unwrap())
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap_or_default()
                .trim()
                .to_string();
        }

        let text_selector_right = Selector::parse(".text-right").unwrap();
        let text_selector_left = Selector::parse(".text-left").unwrap();
        let text_selector_content = Selector::parse(".entry-content").unwrap();

        let chapter_text = if html.select(&text_selector_left).next().is_some() {
            let mut text = String::new();
            for line in html.select(&Selector::parse(".text-left p").unwrap()) {
                text.push('\n');
                text.push('\n');
                text.push_str(line.text().collect::<String>().trim());
            }
            text.trim().to_string()
        } else if html.select(&text_selector_right).next().is_some() {
            let mut text = String::new();
            for line in html.select(&Selector::parse(".text-right p").unwrap()) {
                text.push('\n');
                text.push_str(line.text().collect::<String>().trim());
            }
            text.trim().to_string()
        } else if html.select(&text_selector_content).next().is_some() {
            let mut text = String::new();
            for line in html.select(&Selector::parse(".entry-content p").unwrap()) {
                text.push('\n');
                text.push_str(line.text().collect::<String>().trim());
            }
            text.trim().to_string()
        } else {
            String::from(
                "No text was found. Check the source - if it has text, the scraper is broken.",
            )
        };

        Ok(Chapter {
            source: self.source_id,
            novel_url: novel_path,
            chapter_url: chapter_path,
            chapter_name,
            chapter_contents: chapter_text,
        })
    }

    fn search_novels(&self, search_term: &str) -> Result<Vec<NovelPreview>> {
        let url = format!("{}?s={}&post_type=wp-manga", self.base_url, search_term);
        let html = Html::parse_document(&get_html(url)?);

        let mut novels = Vec::new();

        let selector = Selector::parse(".c-tabs-item__content").unwrap();

        for selection in html.select(&selector) {
            let name_selector = Selector::parse(".post-title").unwrap();
            let name = selection
                .select(&name_selector)
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap()
                .trim()
                .to_string();
            let novel_url = selection
                .select(&name_selector)
                .next()
                .and_then(|t| t.select(&Selector::parse("a").unwrap()).next())
                .and_then(|a| a.value().attr("href"));
            let novel_url = if let Some(n_url) = novel_url {
                n_url.split('/').collect::<Vec<&str>>()[4].to_string()
            } else {
                panic!("Scraper outdated");
            };

            novels.push(NovelPreview {
                source: self.source_id,
                name,
                url: novel_url,
            })
        }

        Ok(novels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_popular() {
        let source = MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "Source".into(),
            None,
            true,
        );

        let popular = source.get_popular(SortOrder::Rating, 1);

        println!("{:?}", popular);
    }

    #[test]
    fn get_chapters() {
        let source = MadaraScraper::new(
            SourceID::new(1),
            "https://zinnovel.com/".into(),
            "Source".into(),
            Some(MadaraPaths::new("manga", "manga", "manga")),
            true,
        );

        let chaps = source.parse_novel_and_chapters("beastmaster-of-the-ages".into());

        println!("{chaps:?}");
    }

    #[test]
    fn parse_chapter() {
        let source = MadaraScraper::new(
            SourceID::new(1),
            "https://zinnovel.com/".into(),
            "Source".into(),
            Some(MadaraPaths::new("manga", "manga", "manga")),
            false,
        );

        let chap = source.parse_chapter(
            "beastmaster-of-the-ages".into(),
            "chapter-2496-beast-massacre".into(),
        );
        println!("{chap:?}");
    }

    #[test]
    fn search_novel() {
        let source = MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "Source".into(),
            None,
            true,
        );

        let s = source.search_novels("awakening");
        println!("{s:?}");
    }
}
