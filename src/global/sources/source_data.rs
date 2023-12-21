use crate::global::sources::SortOrder;
use anyhow::Result;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

use crate::helpers::StatefulList;

use super::freewebnovel::FreeWebNovelScraper;
use super::madara::{MadaraPaths, MadaraScraper};
use super::{Novel, NovelPreview, Scrape, SourceID};

pub struct SourceData {
    pub sources: StatefulList<Source>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Source {
    Madara(MadaraScraper),
    FreeWebNovel(FreeWebNovelScraper),
}

impl Source {
    fn get_name(&self) -> String {
        match self {
            Source::Madara(s) => s.source_name.clone(),
            Source::FreeWebNovel(_) => "FreeWebNovel".into(),
        }
    }

    fn get_id(&self) -> SourceID {
        match self {
            Source::Madara(s) => s.source_id,
            Source::FreeWebNovel(s) => s.source_id,
        }
    }
}

impl Scrape for Source {
    fn get_popular(&self, sort_order: SortOrder, page: usize) -> Result<Vec<NovelPreview>> {
        match self {
            Source::Madara(s) => s.get_popular(sort_order, page),
            Source::FreeWebNovel(s) => s.get_popular(sort_order, page),
        }
    }

    fn parse_novel_and_chapters(&self, novel_path: String) -> Result<Novel> {
        match self {
            Source::Madara(s) => s.parse_novel_and_chapters(novel_path),
            Source::FreeWebNovel(s) => s.parse_novel_and_chapters(novel_path),
        }
    }

    fn parse_chapter(&self, novel_path: String, chapter_path: String) -> Result<super::Chapter> {
        match self {
            Source::Madara(s) => s.parse_chapter(novel_path, chapter_path),
            Source::FreeWebNovel(s) => s.parse_chapter(novel_path, chapter_path),
        }
    }

    fn search_novels(&self, search_term: &str) -> Result<Vec<NovelPreview>> {
        match self {
            Source::Madara(s) => s.search_novels(search_term),
            Source::FreeWebNovel(s) => s.search_novels(search_term),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SourceBookBox {
    #[default]
    Options,
    Chapters,
    Summary,
}

impl SourceData {
    pub fn build() -> Self {
        let box_novel = Source::Madara(MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "BoxNovel".into(),
            None,
            true,
        ));

        let zinn_novel = Source::Madara(MadaraScraper::new(
            SourceID::new(2),
            "https://zinnovel.com/".into(),
            "ZinnNovel".into(),
            Some(MadaraPaths::new("manga", "manga", "manga")),
            true,
        ));

        let novel_translate = Source::Madara(MadaraScraper::new(
            SourceID::new(3),
            "https://noveltranslate.com/".into(),
            "NovelTranslate".into(),
            Some(MadaraPaths::new("all-novels", "novel", "novel")),
            true,
        ));

        let sleepy_translations = Source::Madara(MadaraScraper::new(
            SourceID::new(4),
            "https://sleepytranslations.com/".into(),
            "SleepyTranslations".into(),
            Some(MadaraPaths::new("series", "series", "series")),
            true,
        ));

        let free_novel_me = Source::Madara(MadaraScraper::new(
            SourceID::new(5),
            "https://freenovel.me/".into(),
            "FreeNovelMe".into(),
            None,
            false,
        ));

        let first_kiss_novel = Source::Madara(MadaraScraper::new(
            SourceID::new(6),
            "https://1stkissnovel.org/".into(),
            "FirstKissNovel".into(),
            None,
            true,
        ));

        let most_novel = Source::Madara(MadaraScraper::new(
            SourceID::new(7),
            "https://mostnovel.com/".into(),
            "MostNovel".into(),
            Some(MadaraPaths::new("manga", "manga", "manga")),
            true,
        ));

        let novel_multiverse = Source::Madara(MadaraScraper::new(
            SourceID::new(8),
            "https://www.novelmultiverse.com/".into(),
            "NovelMultiverse".into(),
            None,
            true,
        ));

        let wuxia_world_site = Source::Madara(MadaraScraper::new(
            SourceID::new(9),
            "https://wuxiaworld.site/".into(),
            "WuxiaWorldSite".into(),
            None,
            true,
        ));

        let novel_r18 = Source::Madara(MadaraScraper::new(
            SourceID::new(10),
            "https://novelr18.com/".into(),
            "NovelR18".into(),
            Some(MadaraPaths::new("novel", "manga", "manga")),
            true,
        ));

        let free_web_novel = Source::FreeWebNovel(FreeWebNovelScraper::new(SourceID::new(11)));

        let v = vec![
            box_novel,
            zinn_novel,
            novel_translate,
            sleepy_translations,
            free_novel_me,
            first_kiss_novel,
            most_novel,
            novel_multiverse,
            wuxia_world_site,
            novel_r18,
            free_web_novel,
        ];

        let mut sources = StatefulList::with_items(v);
        sources.state.select(Some(0));

        Self { sources }
    }

    pub fn get_list(&mut self) -> &StatefulList<Source> {
        &self.sources
    }

    pub fn get_list_mut(&mut self) -> &mut StatefulList<Source> {
        &mut self.sources
    }

    pub fn get_source_names(&self) -> Vec<String> {
        self.sources.items.iter().map(|t| t.get_name()).collect()
    }

    pub fn get_state(&self) -> &ListState {
        &self.sources.state
    }

    pub fn get_state_mut(&mut self) -> &mut ListState {
        &mut self.sources.state
    }

    pub fn get_source_by_id(&self, id: SourceID) -> &Source {
        for s in self.sources.items.iter() {
            if s.get_id() == id {
                return s;
            }
        }
        // There should always be a source
        unreachable!()
    }
}
