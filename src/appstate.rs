use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    helpers::{CategoryTabs, StatefulList},
    startup,
};

pub struct AppState {
    pub current_screen: CurrentScreen,
    pub current_main_tab: CategoryTabs,
    pub library_data: LibraryData,
}

impl AppState {
    pub fn build() -> Result<Self, anyhow::Error> {
        let lib_info = startup::load_books()?;

        Ok(Self {
            current_screen: CurrentScreen::Main,
            current_main_tab: CategoryTabs::with_tabs(vec![
                String::from("Library"),
                String::from("Updates"),
                String::from("Sources"),
                String::from("History"),
                String::from("Settings"),
            ]),
            library_data: LibraryData::from(lib_info),
        })
    }
}

pub struct LibraryData {
    // This string contains the category name, and the stateful list contains all books under the aforementioned category.
    pub books: HashMap<String, StatefulList<LibraryBookInfo>>,
    pub categories: CategoryTabs,
}

impl LibraryData {
    pub fn get_category_list(&self) -> &StatefulList<LibraryBookInfo> {
        let idx = self.categories.index;

        let name = &self.categories.tabs[idx];

        // I don't believe this should ever panic, though it needs to be tested
        return self.books.get(name).unwrap();
    }

    pub fn get_category_list_mut(&mut self) -> &mut StatefulList<LibraryBookInfo> {
        let idx = self.categories.index;

        let name = &self.categories.tabs[idx];

        // I don't believe this should ever panic, though it needs to be tested
        return self.books.get_mut(name).unwrap();
    }
}

impl From<LibraryJson> for LibraryData {
    fn from(mut value: LibraryJson) -> Self {
        let mut map: HashMap<_, StatefulList<_>> = HashMap::new();

        // Create all the expected categories in advance
        for category in &value.categories {
            map.insert(category.clone(), StatefulList::new());
        }

        for book in value.entries {
            match book.category.clone() {
                None => {
                    if let Some(list) = map.get_mut(&value.default_category_name) {
                        list.insert(book);
                    } else {
                        map.insert(
                            value.default_category_name.clone(),
                            StatefulList::with_item(book),
                        );
                    }
                }
                Some(category) => {
                    if let Some(list) = map.get_mut(&category) {
                        list.insert(book);
                    } else {
                        // If the category doesn't exist, simply put the book in the default category.
                        // TODO: On saving, revert the category to the default one
                        if map.contains_key(&value.default_category_name) {
                            map.get_mut(&value.default_category_name)
                                .unwrap()
                                .insert(book);
                        } else {
                            map.insert(
                                value.default_category_name.clone(),
                                StatefulList::with_item(book),
                            );
                        }
                    }
                }
            }
        }

        value.categories.insert(0, value.default_category_name);
        let categories = CategoryTabs::with_tabs(value.categories);
        return LibraryData {
            books: map,
            categories,
        };
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
pub struct LibraryJson {
    pub default_category_name: String,
    pub categories: Vec<String>,
    pub entries: Vec<LibraryBookInfo>,
}

impl LibraryJson {
    pub fn new(categories: Vec<String>, entries: Vec<LibraryBookInfo>) -> Self {
        Self {
            default_category_name: String::from("Default"),
            categories,
            entries,
        }
    }

    pub fn empty() -> Self {
        Self {
            default_category_name: String::from("Default"),
            categories: Vec::new(),
            entries: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct LibraryBookInfo {
    pub name: String,
    pub source_data: BookSource,
    pub category: Option<String>,
    pub id: ID,
}

impl LibraryBookInfo {
    pub fn new(name: String, source_data: BookSource, category: Option<String>, id: ID) -> Self {
        Self {
            name,
            source_data,
            category,
            id,
        }
    }

    pub fn display_info(&self) -> String {
        match self.source_data.clone() {
            BookSource::Local(data) => {
                let percent_through = 100.0 * data.current_page as f64 / data.total_pages as f64;
                format!(
                    "{} | Page: {}/{} ({:.2}%)",
                    self.name, data.current_page, data.total_pages, percent_through
                )
            }
            BookSource::Global(data) => {
                let percent_through =
                    100.0 * data.read_chapters as f64 / data.total_chapters as f64;
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
pub enum BookSource {
    Local(LocalBookData),
    Global(GlobalBookData),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct LocalBookData {
    pub path_to_book: String,
    pub current_page: usize,
    pub total_pages: usize,
    // To verify the book is unchanged
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GlobalBookData {
    pub path_to_book: String,
    pub read_chapters: usize,
    pub total_chapters: usize,
    pub unread_downloaded_chapters: usize,
}
