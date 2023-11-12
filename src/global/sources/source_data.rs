use ratatui::widgets::ListState;

use crate::helpers::StatefulList;

use super::madara::{MadaraPaths, MadaraScraper};
use super::{ChapterPreview, Novel, NovelPreview, Scrape, SourceID};

type StatefulSourceList = StatefulList<(String, Box<dyn Scrape>)>;

pub struct SourceData {
    pub sources: StatefulSourceList,
    pub novel_results: StatefulList<NovelPreview>,
    pub current_novel: Option<Novel>,
    pub current_novel_chaps: StatefulList<ChapterPreview>,
    pub current_book_ui_option: SourceBookBox,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceBookBox {
    Options,
    Chapters,
}

impl SourceData {
    pub fn build() -> Self {
        let box_novel = MadaraScraper::new(
            SourceID::new(1),
            "https://boxnovel.com/".into(),
            "BoxNovel".into(),
            None,
            true,
        );
        let box_novel: Box<dyn Scrape> = Box::new(box_novel);

        let zinn_novel = MadaraScraper::new(
            SourceID::new(2),
            "https://zinnovel.com/".into(),
            "ZinnNovel".into(),
            Some(MadaraPaths::new("manga", "manga", "manga")),
            false,
        );
        let zinn_novel: Box<dyn Scrape> = Box::new(zinn_novel);

        let novel_translate = MadaraScraper::new(
            SourceID::new(3),
            "https://noveltranslate.com/".into(),
            "NovelTranslate".into(),
            Some(MadaraPaths::new("all-novels", "novel", "novel")),
            false,
        );
        let novel_translate: Box<dyn Scrape> = Box::new(novel_translate);

        let v = vec![
            (String::from("BoxNovel"), box_novel),
            (String::from("ZinnNovel"), zinn_novel),
            (String::from("NovelTranslate"), novel_translate),
        ];

        let mut sources = StatefulList::with_items(v);
        sources.state.select(Some(0));

        Self {
            sources,
            novel_results: StatefulList::new(),
            current_novel: None,
            current_novel_chaps: StatefulList::new(),
            current_book_ui_option: SourceBookBox::Options,
        }
    }

    pub fn get_list(&mut self) -> &StatefulSourceList {
        &self.sources
    }

    pub fn get_list_mut(&mut self) -> &mut StatefulSourceList {
        &mut self.sources
    }

    pub fn get_source_names(&self) -> Vec<String> {
        self.sources.items.iter().map(|t| t.0.clone()).collect()
    }

    pub fn get_state(&self) -> &ListState {
        &self.sources.state
    }

    pub fn get_state_mut(&mut self) -> &mut ListState {
        &mut self.sources.state
    }
}
