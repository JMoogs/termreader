use serde::{Deserialize, Serialize};

pub struct AppState {
    pub(crate) current_screen: CurrentScreen,
    pub(crate) current_tab: MainScreenTabs,
}

impl AppState {
    pub fn build() -> Self {
        Self {
            current_screen: CurrentScreen::Main,
            current_tab: MainScreenTabs::build(),
        }
    }
}

pub(crate) struct MainScreenTabs {
    pub(crate) tabs: Vec<String>,
    pub(crate) index: usize,
}

impl MainScreenTabs {
    fn build() -> Self {
        let v = vec![
            String::from("Library"),
            String::from("Updates"),
            String::from("Sources"),
            String::from("History"),
            String::from("Settings"),
        ];

        Self { tabs: v, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.tabs.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum CurrentScreen {
    Main,
    Reader,
    ExitingReader,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ID {
    id: usize,
}

impl ID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct LibraryInfo {
    name: String,
    source_data: BookSource,
}

impl LibraryInfo {
    pub fn display_info(&self) -> String {
        match self.source_data.clone() {
            BookSource::Local(data) => {
                let percent_through = data.current_page as f64 / data.total_pages as f64;
                format!(
                    "{} | Page: {}/{} ({:.2}%)",
                    self.name, data.current_page, data.total_pages, percent_through
                )
            }
            BookSource::Global(data) => {
                let percent_through = data.read_chapters as f64 / data.total_chapters as f64;
                if data.unread_downloaded_chapters > 0 {
                    format!(
                        "{} | Chapter: {}/{} ({:.2}%) | Downloaded: {}",
                        self.name,
                        data.read_chapters,
                        data.total_chapters,
                        percent_through,
                        data.unread_downloaded_chapters
                    )
                } else {
                    format!(
                        "{} | Chapter: {}/{} ({:.2}%)",
                        self.name, data.read_chapters, data.total_chapters, percent_through
                    )
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
enum BookSource {
    Local(LocalBookData),
    Global(GlobalBookData),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
struct LocalBookData {
    path_to_book: String,
    current_page: usize,
    total_pages: usize,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
struct GlobalBookData {
    path_to_book: String,
    read_chapters: usize,
    total_chapters: usize,
    unread_downloaded_chapters: usize,
}
