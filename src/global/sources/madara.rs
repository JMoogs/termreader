use super::{
    get_html, Chapter, ChapterPreview, Novel, NovelPreview, NovelStatus, SortOrder, SourceID,
};
use anyhow::Result;
use chrono::Local;
use html5ever::tree_builder::TreeSink;
use regex::Regex;
use reqwest::Method;
use scraper::{Html, Selector};
// https://github.com/LNReader/lnreader/blob/main/src/sources/multisrc/madara/MadaraScraper.js

struct MadaraScraper {
    source_id: SourceID,
    base_url: String,
    // page: String,
    source_name: String,
    path: Option<MadaraPaths>,
    new_chap_endpoint: bool,
}

#[derive(Clone)]
struct MadaraPaths {
    novels: String,
    novel: String,
    chapter: String,
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
    fn new(
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
                novel_name.into(),
                novel_url.into(),
            ));
        }

        Ok(novels)
    }

    fn parse_novel_and_chapters(&self, novel_path: String) -> Result<Novel> {
        let url = &self.path.clone().unwrap_or_default().novels;
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
            let req = reqwest::blocking::Request::new(Method::POST, url);
            client.execute(req)?.text()?
        } else {
            todo!()
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

            chapters.push(ChapterPreview {
                release_date,
                name: chapter_name,
                url: chapter_url,
            });
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
            chapters,
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

        let mut html = Html::parse_document(&get_html(url)?);

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

        let chapter_text = if html.select(&text_selector_1).next().is_some() {
            let ids: Vec<_> = html
                .select(&Selector::parse(".text-right div").unwrap())
                .map(|x| x.id())
                .collect();
            for id in ids {
                html.remove_from_parent(&id);
            }
            html.select(&text_selector_1).next().unwrap().html()
        } else if html.select(&text_selector_2).next().is_some() {
            let ids: Vec<_> = html
                .select(&Selector::parse(".text-left div").unwrap())
                .map(|x| x.id())
                .collect();
            for id in ids {
                html.remove_from_parent(&id);
            }
            html.select(&text_selector_2).next().unwrap().html()
        } else if html.select(&text_selector_3).next().is_some() {
            let ids: Vec<_> = html
                .select(&Selector::parse(".entry-content div").unwrap())
                .map(|x| x.id())
                .collect();
            for id in ids {
                html.remove_from_parent(&id);
            }
            html.select(&text_selector_3).next().unwrap().html()
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
            chatper_contents: chapter_text,
        })
    }

    fn search_novels(&self) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_popular() {
        let boxnovel = MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "BoxNovel".into(),
            None,
            true,
        );

        let popular = boxnovel.get_popular(SortOrder::Rating, 1);

        println!("{:?}", popular);
    }

    #[test]
    fn get_chapters() {
        let boxnovel = MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "BoxNovel".into(),
            None,
            true,
        );

        let chaps = boxnovel
            .parse_novel_and_chapters("awakening-the-weakest-talent-only-i-level-up".into());

        println!("{chaps:?}");
    }

    #[test]
    fn parse_chapter() {
        let boxnovel = MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "BoxNovel".into(),
            None,
            true,
        );

        let chap = boxnovel.parse_chapter(
            "awakening-the-weakest-talent-only-i-level-up".into(),
            "chapter-1".into(),
        );
        println!("{chap:?}");
    }
}
