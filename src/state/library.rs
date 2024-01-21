use crate::appstate::{BookSource, GlobalBookData, ID};
use crate::global::sources::Novel;
use crate::helpers::StatefulList;
use crate::local::LocalBookData;
use crate::reader::buffer::{BookProgress, BookProgressData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data related to the user's library of novels
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryData {
    /// The string contains the category name, and the stateful list contains all books under the aforementioned category
    pub books: HashMap<String, StatefulList<LibBookInfo>>,
    /// The name of the default category
    pub default_category_name: String,
    /// A list of all the categories. This list manages which is selected
    pub categories: StatefulList<String>,
}

impl LibraryData {
    /// Creates an empty library
    pub fn empty() -> Self {
        let default_name = String::from("Default");
        let mut map = HashMap::new();
        map.insert(default_name.clone(), StatefulList::new());

        Self {
            books: map,
            default_category_name: default_name.clone(),
            categories: StatefulList::with_item(default_name),
        }
    }

    /// Moves a book to a category.
    /// If `None` is passed in as the category name, or the category name isn't recognized, the book is moved to the default category
    pub fn move_category(&mut self, id: ID, category_name: Option<String>) {
        let book = self.find_book(id).unwrap().clone();
        self.remove_book(id);
        self.add_book(book, category_name);
    }

    /// Creates a new category
    pub fn create_category(&mut self, name: String) {
        // Don't allow two categories with the same name.
        if self.books.contains_key(&name) {
            return;
        }
        self.books.insert(name.clone(), StatefulList::new());
        self.categories.items.push(name)
    }

    /// Renames a category
    pub fn rename_category(&mut self, old_name: String, new_name: String) {
        if let Some(v) = self.books.remove(&old_name) {
            if self.default_category_name == old_name {
                self.default_category_name = new_name.clone();
            }

            let pos = self
                .categories
                .items
                .iter()
                .position(|r| r == &old_name)
                .unwrap();

            self.categories.items.remove(pos);
            self.categories.items.insert(pos, new_name.clone());

            self.books.insert(new_name, v);
        }
    }

    /// Deletes a category. The default category cannot be deleted, meaning a category will always exist
    pub fn delete_category(&mut self, name: String) {
        // Don't allow the user to delete the default category: it can only be renamed.
        if self.default_category_name == name {
            return;
        }

        if let Some(mut v) = self.books.remove(&name) {
            let pos = self
                .categories
                .items
                .iter()
                .position(|r| r == &name)
                .unwrap();
            // If the category being deleted is removed, set the selection to the first category, that will always exist.
            if pos == self.categories.selected_idx().unwrap() {
                self.categories.select_first()
            }
            self.categories.items.remove(pos);

            let default_list = self.books.get_mut(&self.default_category_name).unwrap();

            default_list.items.append(&mut v.items)
        }
    }

    /// Adds a book to a category, adding the default of the category isn't recognized.
    pub fn add_book(&mut self, mut book: LibBookInfo, category: Option<String>) {
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
            list.select_first();
        }
    }

    /// Removes a book from the library
    pub fn remove_book(&mut self, id: ID) {
        let lists = self.books.values_mut();

        for list in lists {
            let search = list.items.iter().position(|i| i.id == id);
            if search.is_none() {
                continue;
            }

            let pos = search.unwrap();

            let sel = list.selected_idx().unwrap();

            list.items.remove(pos);

            if sel == pos {
                if !list.items.is_empty() {
                    list.select_first();
                } else {
                    list.unselect();
                }
            }
        }
    }

    /// Renames a book
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

    /// Given an ID, returns a reference to the book
    pub fn find_book(&self, id: ID) -> Option<&LibBookInfo> {
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

    /// Given an ID, returns a mutable reference to the book
    pub fn find_book_mut(&mut self, id: ID) -> Option<&mut LibBookInfo> {
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

    /// Returns a reference to the list of books in the currently selected category
    pub fn get_category_list(&self) -> &StatefulList<LibBookInfo> {
        let idx = self.categories.selected_idx().unwrap();

        let name = &self.categories.items[idx];

        match self.books.get(name) {
            Some(books) => books,
            None => panic!("This should never happen"),
        }
    }

    /// Returns a mutable reference to the list of books in the currently selected category
    pub fn get_category_list_mut(&mut self) -> &mut StatefulList<LibBookInfo> {
        let idx = self.categories.selected_idx().unwrap();

        let name = &self.categories.items[idx];

        match self.books.get_mut(name) {
            Some(books) => books,
            None => panic!("This should never happen"),
        }
    }
}

/// Contains info about a book that is in a user's library
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibBookInfo {
    /// The name of the book
    pub name: String,
    /// Information about the source
    pub source_data: BookSource,
    /// The category name of which the book is in. None implies that it is in the default category
    pub category: Option<String>,
    /// The unique ID of the book
    pub id: ID,
}

impl PartialEq for LibBookInfo {
    // Only IDs need to be compared - we only want to check if it's the same book, not if all the properties are the same
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl LibBookInfo {
    /// Returns true when the source is local
    pub fn is_local(&self) -> bool {
        matches!(self.source_data, BookSource::Local(_))
    }

    /// Creates an instance of `LibBookInfo` from a path to a local source file.
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

    /// Creates an instance from a novel
    pub fn from_global(novel: Novel, category: Option<String>) -> Result<Self, anyhow::Error> {
        let data = GlobalBookData::create(novel, 1);
        let source = BookSource::Global(data);
        Ok(Self {
            name: source.get_name(),
            source_data: source,
            category,
            id: ID::generate(),
        })
    }

    /// Creates an instance of `LibBookInfo`
    pub fn new(source_data: BookSource, category: Option<String>) -> Self {
        Self {
            name: source_data.get_name(),
            source_data,
            category,
            id: ID::generate(),
        }
    }

    /// Updates the progress of a book
    pub fn update_progress(&mut self, progress: BookProgress) {
        match &mut self.source_data {
            BookSource::Local(ref mut data) => {
                data.progress.progress = progress;
            }
            BookSource::Global(d) => {
                let p = d.chapter_progress.get_mut(&d.current_chapter).unwrap();
                p.progress = progress;
            }
        }
    }

    /// Sets the progress of a book
    pub fn set_progress(&mut self, progress: BookProgressData) {
        match &mut self.source_data {
            BookSource::Local(ref mut data) => {
                data.progress = progress;
            }
            BookSource::Global(d) => {
                d.chapter_progress.insert(d.current_chapter, progress);
            }
        }
    }

    /// Gets the progress of a book
    pub fn get_progress(&self) -> BookProgress {
        match &self.source_data {
            BookSource::Local(ref data) => data.progress.progress,
            BookSource::Global(d) => {
                let p = d.chapter_progress.get(&d.current_chapter);
                match p {
                    Some(place) => place.progress,
                    None => BookProgress::Location((0, 0)),
                }
            }
        }
    }

    /// Displays info about a book
    pub fn display_info(&self) -> String {
        match self.source_data.clone() {
            BookSource::Local(data) => {
                let prog = data.get_progress_display();
                format!(
                    "{} | Lines: {}/{} ({:.2}%)",
                    self.name, prog.0, prog.1, prog.2
                )
            }
            BookSource::Global(data) => {
                let percent_through =
                    100.0 * data.read_chapters.len() as f64 / data.total_chapters as f64;
                format!(
                    "{} | Chapter: {}/{} ({:.2}%)",
                    self.name,
                    data.read_chapters.len(),
                    data.total_chapters,
                    percent_through,
                )
            }
        }
    }
}
