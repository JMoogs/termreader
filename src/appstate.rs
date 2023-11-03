use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::reader::buffer::{BookProgress, BookProgressData};
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
    pub reader_menu_options: ReaderMenuOptions,
    /// A buffer to store any text that may be typed by the user.
    pub text_buffer: String,
}

pub struct ReaderMenuOptions {
    pub local_options: StatefulList<String>,
    pub global_options: StatefulList<String>,
    pub category_moves: StatefulList<String>,
}

impl ReaderMenuOptions {
    fn new(categories: Vec<String>) -> Self {
        let local_options = StatefulList::with_items(vec![
            String::from("Continue reading"),
            String::from("Move to category..."),
            String::from("Rename"),
            String::from("Start from beginning"),
            String::from("Remove book from library"),
        ]);

        let global_options = StatefulList::with_items(vec![
            String::from("Continue reading"),
            String::from("View chapter list"),
            String::from("Move to category..."),
            String::from("Rename"),
            String::from("Start from beginning"),
            String::from("Remove book from library"),
        ]);

        let category_moves = StatefulList::with_items(categories);

        Self {
            local_options,
            global_options,
            category_moves,
        }
    }
}

impl AppState {
    pub fn build() -> Result<Self, anyhow::Error> {
        let lib_info = startup::load_books()?;
        let library_data = LibraryData::from(lib_info);

        let cats = library_data.categories.tabs.clone();

        Ok(Self {
            current_screen: CurrentScreen::Main(MenuType::Default),
            current_main_tab: CategoryTabs::with_tabs(vec![
                String::from("Library"),
                String::from("Updates"),
                String::from("Sources"),
                String::from("History"),
                String::from("Settings"),
            ]),
            library_data,
            reader_data: None,
            reader_menu_options: ReaderMenuOptions::new(cats),
            text_buffer: String::new(),
        })
    }

    pub fn update_reader(&mut self, book: BookInfo) -> Result<(), anyhow::Error> {
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
    pub books: HashMap<String, StatefulList<BookInfo>>,
    pub default_category_name: String,
    pub categories: CategoryTabs,
}

impl LibraryData {
    // Problem: does not unselect if the book is selected.
    pub fn move_category(&mut self, id: ID, category_name: Option<String>) {
        let book = self.find_book(id).unwrap().clone();
        self.remove_book(id);
        self.add_book(book, category_name);
    }

    pub fn create_category(&mut self, name: String) {
        self.books.insert(name, StatefulList::new());
    }

    pub fn add_book(&mut self, mut book: BookInfo, category: Option<String>) {
        let (list, cat_exists) = match category {
            Some(ref cat) => match self.books.get_mut(cat) {
                Some(l) => (l, true),
                None => (
                    self.books.get_mut(&self.default_category_name).unwrap(),
                    false,
                ),
            },
            None => (
                self.books.get_mut(&self.default_category_name).unwrap(),
                false,
            ),
        };

        if cat_exists {
            book.category = category;
        } else {
            book.category = None;
        }
        list.items.push(book);

        if list.items.len() == 1 {
            list.state.select(Some(0));
        }
    }

    pub fn remove_book(&mut self, id: ID) {
        let lists = self.books.values_mut();

        for list in lists {
            let search = list.items.iter().position(|i| i.id == id);
            if search.is_none() {
                continue;
            }

            let pos = search.unwrap();

            let sel = list.state.selected().unwrap();

            list.items.remove(pos);

            if sel == pos {
                if list.items.len() > 0 {
                    list.state.select(Some(0));
                } else {
                    list.state.select(None);
                }
            }
        }
    }

    pub fn rename_book(&mut self, id: ID, new_name: String) {
        let book = self.find_book_mut(id);

        if book.is_none() {
            return;
        }

        let book = book.unwrap();
        book.name = new_name.clone();

        match &mut book.source_data {
            BookSource::Local(d) => d.name = new_name,
            BookSource::Global(d) => d.name = new_name,
        }
    }

    pub fn find_book(&self, id: ID) -> Option<&BookInfo> {
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

    pub fn find_book_mut(&mut self, id: ID) -> Option<&mut BookInfo> {
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

    pub fn get_category_list(&self) -> &StatefulList<BookInfo> {
        let idx = self.categories.index;

        let name = &self.categories.tabs[idx];

        match self.books.get(name) {
            Some(books) => return books,
            None => panic!("This should never happen"),
        }
    }

    pub fn get_category_list_mut(&mut self) -> &mut StatefulList<BookInfo> {
        let idx = self.categories.index;

        let name = &self.categories.tabs[idx];

        match self.books.get_mut(name) {
            Some(books) => return books,
            None => panic!("This should never happen"),
        }
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

        // Ensure the default category has an entry, so that other functions always work.
        if !map.contains_key(&default_category_name) {
            map.insert(value.default_category_name.clone(), StatefulList::new());
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
    Main(MenuType),
    Reader,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum MenuType {
    Default,
    LocalSelected,
    GlobalSelected,
    Renaming(u8),
    MoveCategories,
}

/// An ID used to uniquely identify a book.
/// Determined using the current timestamp, resulting in very little risk of collisions.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ID {
    id: u128,
}

impl ID {
    /// Generates an ID using system time.
    pub fn generate() -> Self {
        let now = SystemTime::now();

        let unix_timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time has gone VERY backwards.");

        Self {
            id: unix_timestamp.as_nanos(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryJson {
    pub default_category_name: String,
    pub categories: Vec<String>,
    pub entries: Vec<BookInfo>,
}

impl LibraryJson {
    pub fn new(categories: Vec<String>, entries: Vec<BookInfo>) -> Self {
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

        let entries: Vec<BookInfo> = value
            .books
            .into_values()
            .map(|v| {
                let s: Vec<BookInfo> = v.into();
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

/// Contains info about a book.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookInfo {
    /// The name of the book.
    pub name: String,
    /// Contains information that is unique to the type of source.
    pub source_data: BookSource,
    /// The category name of which the book is in. None implies that it is in the default category.
    pub category: Option<String>,
    /// The unique ID of the book.
    pub id: ID,
}

impl BookInfo {
    /// Determine whether a book has a local source. Returns true when the source is local.
    pub fn is_local(&self) -> bool {
        matches!(self.source_data, BookSource::Local(_))
    }

    /// Creates an instance of `BookInfo` from a path to a local source file.
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

    /// Creates an instance of `BookInfo`.
    pub fn new(source_data: BookSource, category: Option<String>) -> Self {
        Self {
            name: source_data.get_name(),
            source_data,
            category,
            id: ID::generate(),
        }
    }

    pub fn update_progress(&mut self, progress: BookProgress) {
        match &mut self.source_data {
            BookSource::Local(ref mut data) => {
                data.progress.progress = progress;
            }
            BookSource::Global(_) => {
                todo!()
            }
        }
    }

    pub fn set_progress(&mut self, progress: BookProgressData) {
        match &mut self.source_data {
            BookSource::Local(ref mut data) => {
                data.progress = progress;
            }
            BookSource::Global(_) => {
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
                format!(
                    "{} | Chapter: {}/{} ({:.2}%)",
                    self.name, data.read_chapters, data.total_chapters, percent_through,
                )
            }
        }
    }
}

/// The type of source from which a book is provided. The variants each contain related info to the source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookSource {
    /// A locally sourced book.
    Local(LocalBookData),
    /// A book that is not sourced on the user's device.
    Global(GlobalBookData),
}

impl BookSource {
    /// Get the name of a book from its source.
    fn get_name(&self) -> String {
        match self {
            BookSource::Local(d) => d.name.clone(),
            BookSource::Global(d) => d.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalBookData {
    pub name: String,
    pub path_to_book: String,
    pub read_chapters: usize,
    pub total_chapters: usize,
    pub chapter_progress: HashMap<usize, BookProgressData>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChapterDownloadData {
    pub location: String,
    pub chapter_progress: HashMap<usize, BookProgressData>,
}
