use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

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

        LibraryData {
            books: map,
            categories,
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

// A book must be split into lines with a maximum width equivalent to the width of the terminal
// The amount of lines rendered must be the same as the height of the terminal
// These can change at any time, though it is generally unlikely to happen.
// Note that lines must be split on spaces.

/// Represents a chapter for a non-local source. Behaviour for a local source is undecided as of current.
pub struct BookPortion<'a> {
    /// The entire portion split on whitespace into words.
    // Note: Consider using Rc<str> here?
    words: Vec<String>,
    /// A value representing the current width (i.e. the max character length) of the stored lines.
    current_width: u16,
    /// A vector of ranges [start, end) of words contained in the line.
    lines: Option<Vec<(usize, usize)>>,
    /// A value representing the current height that is rendered i.e. `rendered_lines.len()`
    current_height: u16,
    /// Represents the index of the first line of `lines` that is contained in `rendered_lines`
    renedered_line_idx: Option<usize>,
    /// A vector containing all the lines that should be rendered onto the terminal.
    rendered_lines: Option<VecDeque<&'a str>>,
}

impl<'a> BookPortion<'a> {
    fn new(text: String, width: u16, height: u16) -> Self {
        Self {
            words: to_words(text),
            current_width: width,
            lines: None,
            current_height: height,
            renedered_line_idx: None,
            rendered_lines: None,
        }
    }

    fn set_lines(&mut self) {
        let max_len = self.current_width as usize;

        let mut lines = Vec::new();

        let mut idx = 0;
        let mut line_start = 0;
        let mut line_len = 0;
        let words = &self.words;

        while idx < words.len() {
            // If the word is longer than the terminal size, just skip the word for now.
            // Later we can worry about splitting it into smaller words to be displayed.
            if words[idx].len() > line_len {
                idx += 1;
            }
            // Using '<' instead of '<=' as we are adding a length of 1 for the space between the words.
            if line_len + words[idx].len() < max_len {
                line_len += words[idx].len() + 1;
            } else {
                lines.push((line_start, idx));
                line_start = idx;
                line_len = 0;
            }

            idx += 1;
        }

        self.lines = Some(lines);
    }

    fn set_rendered_lines(&mut self, start_line: usize) {
        if self.lines.is_none() {
            return;
        }

        self.renedered_line_idx = Some(start_line);

        let mut lines = VecDeque::new();

        for i in start_line..(start_line + self.current_height as usize) {
            let (start_word, end_word) = self.lines.as_ref().unwrap()[i];
            let str: &'a str = self.words[start_word..end_word].join(" ");

            lines.push_back(str);
        }

        self.rendered_lines = Some(lines);
    }
}

fn to_words(text: impl AsRef<str>) -> Vec<String> {
    text.as_ref()
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

#[cfg(test)]
mod tests {}
