use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::reader::buffer::BookProgressData;
use crate::reader::ReaderData;
use crate::{
    helpers::{CategoryTabs, StatefulList},
    local::LocalBookData,
    startup,
};

pub struct AppState {
    pub current_screen: CurrentScreen,
    pub current_main_tab: CategoryTabs,
    pub library_data: LibraryData,
    pub reader_data: Option<ReaderData>,
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
            reader_data: None,
        })
    }

    pub fn update_reader(&mut self, book: LibraryBookInfo) -> Result<(), anyhow::Error> {
        self.reader_data = Some(ReaderData::create(book)?);

        Ok(())
    }

    pub fn update_lib_from_reader(&mut self) -> Result<()> {
        if self.reader_data.is_none() {
            return Ok(());
        }

        self.reader_data.as_mut().unwrap().set_progress()?;

        let copy = self.reader_data.as_ref().unwrap().book_info.clone();
        let id = copy.id;

        let b = self.library_data.find_book_mut(id);

        match b {
            None => panic!(),
            Some(book) => {
                let _ = std::mem::replace(book, copy);
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct LibraryData {
    // This string contains the category name, and the stateful list contains all books under the aforementioned category.
    pub books: HashMap<String, StatefulList<LibraryBookInfo>>,
    pub default_category_name: String,
    pub categories: CategoryTabs,
}

impl LibraryData {
    pub fn find_book(&self, id: ID) -> Option<&LibraryBookInfo> {
        let lists = self.books.values();

        for list in lists {
            let search = list.items.iter().find(|i| i.id == id);
            if search.is_none() {
                continue;
            }
            let res = search.unwrap();
            return Some(res);
        }

        None
    }

    pub fn find_book_mut(&mut self, id: ID) -> Option<&mut LibraryBookInfo> {
        let lists = self.books.values_mut();

        for list in lists {
            let search = list.items.iter_mut().find(|i| i.id == id);
            if search.is_none() {
                continue;
            }
            let res = search.unwrap();
            return Some(res);
        }

        None
    }

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

        let default_category_name = value.default_category_name.clone();

        // Create all the expected categories in advance
        for category in &value.categories {
            map.insert(category.clone(), StatefulList::new());
        }

        for book in value.entries {
            match book.category.clone() {
                None => {
                    if let Some(list) = map.get_mut(&default_category_name) {
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
                        if map.contains_key(&default_category_name) {
                            map.get_mut(&default_category_name).unwrap().insert(book);
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

        LibraryData {
            books: map,
            categories,
            default_category_name,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum CurrentScreen {
    Main,
    Reader,
    ExitingReader,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ID {
    id: usize,
}

impl ID {
    pub fn generate() -> Self {
        let now = SystemTime::now();

        let unix_timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time has gone VERY backwards.");

        Self {
            id: unix_timestamp.as_millis() as usize,
        }
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

impl From<LibraryData> for LibraryJson {
    fn from(value: LibraryData) -> Self {
        let default_category_name = value.default_category_name;
        let categories = value.categories.tabs;

        // Don't add the default
        let categories = categories
            .into_iter()
            .filter(|v| v != &default_category_name)
            .collect();

        let entries: Vec<LibraryBookInfo> = value
            .books
            .into_values()
            .map(|v| {
                let s: Vec<LibraryBookInfo> = v.into();
                s
            })
            .flatten()
            .collect();
        Self {
            default_category_name,
            categories,
            entries,
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
    pub fn is_local(&self) -> bool {
        matches!(self.source_data, BookSource::Local(_))
    }

    pub fn from_local(
        path: impl Into<String>,
        category: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let data = LocalBookData::create(path)?;

        let source = BookSource::Local(data);

        Ok(Self {
            name: source.get_name(),
            source_data: source,
            category,
            id: ID::generate(),
        })
    }

    pub fn new(source_data: BookSource, category: Option<String>, id: ID) -> Self {
        Self {
            name: source_data.get_name(),
            source_data,
            category,
            id,
        }
    }

    pub fn set_progress(&mut self, progress: BookProgressData) {
        match &mut self.source_data {
            BookSource::Local(ref mut data) => {
                data.progress = progress;
            }
            BookSource::Global(data) => {
                todo!()
            }
        }
    }

    pub fn display_info(&self) -> String {
        match self.source_data.clone() {
            BookSource::Local(data) => {
                let prog = data.get_progress_display();
                // format!("{} | {}", self.name, data.format.get_progress())
                format!(
                    "{} | Lines: {}/{} ({:.2}%)",
                    self.name, prog.0, prog.1, prog.2
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

impl BookSource {
    fn get_name(&self) -> String {
        match self {
            BookSource::Local(d) => d.name.clone(),
            BookSource::Global(d) => d.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GlobalBookData {
    pub name: String,
    pub path_to_book: String,
    pub read_chapters: usize,
    pub total_chapters: usize,
    pub unread_downloaded_chapters: usize,
}
