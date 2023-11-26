use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;

use crate::global::sources::{ChapterPreview, NovelPreview};
use crate::reader::buffer::{BookProgress, BookProgressData};
use crate::reader::ReaderData;
use crate::{
    global::sources::{source_data::SourceData, Novel},
    helpers::{CategoryTabs, StatefulList},
    local::LocalBookData,
    startup,
};

pub struct AppState {
    pub current_screen: CurrentScreen,
    /// Stores a list of all the previously accessed screens - the implementation of the back button.
    pub prev_screens: Vec<CurrentScreen>,
    pub current_main_tab: CategoryTabs,
    pub library_data: LibraryData,
    pub reader_data: Option<ReaderData>,
    pub menu_options: MenuOptions,
    pub source_data: SourceData,
    /// A buffer to store any text that may be typed by the user.
    pub text_buffer: String,
    pub channels: ChannelData,
}

pub struct ChannelData {
    pub sender: Sender<RequestData>,
    pub reciever: Receiver<RequestData>,
    pub loading: bool,
}

impl ChannelData {
    fn new() -> Self {
        let (sender, reciever) = std::sync::mpsc::channel();
        Self {
            sender,
            reciever,
            loading: false,
        }
    }
}

pub enum RequestData {
    SearchResults(Result<Vec<NovelPreview>>),
}

pub struct MenuOptions {
    pub local_options: StatefulList<String>,
    pub global_options: StatefulList<String>,
    pub category_moves: StatefulList<String>,
    pub source_options: StatefulList<String>,
    pub source_book_options: StatefulList<String>,
}

impl MenuOptions {
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

        let source_options =
            StatefulList::with_items(vec![String::from("Search"), String::from("View popular")]);

        let source_book_options = StatefulList::with_items(vec![
            String::from("Start from beginning"),
            String::from("Add book to library"),
            // String::from("View chapters"),
        ]);

        Self {
            local_options,
            global_options,
            category_moves,
            source_options,
            source_book_options,
        }
    }
}

impl AppState {
    pub fn get_last_screen(&self) -> CurrentScreen {
        return self.prev_screens.last().unwrap().clone();
    }

    pub fn update_screen(&mut self, new: CurrentScreen) {
        if self.current_screen.in_reader() && new == CurrentScreen::Reader {
            return;
        }
        if self.current_screen.on_main_menu() {
            self.prev_screens = Vec::new();
        }
        if new.on_main_menu() {
            self.prev_screens = Vec::new();
        } else {
            self.prev_screens.push(self.current_screen);
        }
        self.current_screen = new;
    }

    pub fn to_lib_screen(&mut self) {
        self.prev_screens = Vec::new();
        self.current_screen = CurrentScreen::Library(LibraryOptions::Default);
    }

    pub fn to_source_screen(&mut self) {
        self.prev_screens = Vec::new();
        self.current_screen = CurrentScreen::Sources(SourceOptions::Default);
    }

    pub fn build() -> Result<Self, anyhow::Error> {
        let lib_info = startup::load_books()?;
        let library_data = LibraryData::from(lib_info);

        let cats = library_data.categories.tabs.clone();

        Ok(Self {
            current_screen: CurrentScreen::Library(LibraryOptions::Default),
            prev_screens: Vec::new(),
            current_main_tab: CategoryTabs::with_tabs(vec![
                String::from("Library"),
                String::from("Updates"),
                String::from("Sources"),
                String::from("History"),
                String::from("Settings"),
            ]),
            library_data,
            reader_data: None,
            menu_options: MenuOptions::new(cats),
            source_data: SourceData::build(),
            text_buffer: String::new(),
            channels: ChannelData::new(),
        })
    }

    pub fn move_to_reader(
        &mut self,
        mut book: BookInfo,
        chapter: Option<usize>,
    ) -> Result<(), anyhow::Error> {
        self.update_screen(CurrentScreen::Reader);
        match book.get_source_data_mut() {
            BookSource::Local(_) => {
                self.reader_data = Some(ReaderData::create(book, chapter, None)?)
            }
            BookSource::Global(_) => {
                let source = &self.source_data.sources.selected().unwrap();
                self.reader_data = Some(ReaderData::create(book, chapter, Some(source))?)
            }
        }

        Ok(())
    }

    pub fn update_lib_from_reader(&mut self) -> Result<()> {
        let reader_data = if self.reader_data.is_none() {
            return Ok(());
        } else {
            self.reader_data.as_mut().unwrap()
        };

        reader_data.set_progress()?;

        let reader_data = match &reader_data.book_info {
            BookInfo::Library(d) => d,
            BookInfo::Reader(_) => return Ok(()),
        };

        let copy = reader_data.clone();
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

    pub fn reset_selections(&mut self) {
        self.menu_options.global_options.state.select(Some(0));
        self.menu_options.local_options.state.select(Some(0));
        self.menu_options.source_options.state.select(Some(0));
    }
}

#[derive(Clone)]
pub struct LibraryData {
    // This string contains the category name, and the stateful list contains all books under the aforementioned category.
    pub books: HashMap<String, StatefulList<LibBookInfo>>,
    pub default_category_name: String,
    pub categories: CategoryTabs,
    pub menu_data: LibMenuData,
}

#[derive(Clone)]
pub struct LibMenuData {
    pub ch_scroll: usize,
    /// When true, the chapter tab is selected. When false, the synopsis is selected.
    pub ch_selected: bool,
    pub ch_list: Option<StatefulList<ChapterPreview>>,
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

    pub fn get_category_list(&self) -> &StatefulList<LibBookInfo> {
        let idx = self.categories.index;

        let name = &self.categories.tabs[idx];

        match self.books.get(name) {
            Some(books) => return books,
            None => panic!("This should never happen"),
        }
    }

    pub fn get_category_list_mut(&mut self) -> &mut StatefulList<LibBookInfo> {
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
            menu_data: LibMenuData {
                ch_scroll: 0,
                ch_selected: true,
                ch_list: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum CurrentScreen {
    // Main(MenuType),
    Library(LibraryOptions),
    Updates(UpdateOptions),
    Sources(SourceOptions),
    History(HistoryOptions),
    Settings(SettingsOptions),
    Reader,
    Typing,
}

impl CurrentScreen {
    pub fn in_reader(&self) -> bool {
        matches!(self, CurrentScreen::Reader)
    }
    pub fn on_main_menu(&self) -> bool {
        match self {
            CurrentScreen::Library(LibraryOptions::Default)
            | CurrentScreen::Sources(SourceOptions::Default)
            | CurrentScreen::Updates(UpdateOptions::Default)
            | CurrentScreen::History(HistoryOptions::Default)
            | CurrentScreen::Settings(SettingsOptions::Default) => return true,

            _ => false,
        }
    }

    pub fn on_library_menu(&self) -> bool {
        matches!(self, CurrentScreen::Library(LibraryOptions::Default))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum LibraryOptions {
    Default,
    LocalBookSelect,
    GlobalBookSelect,
    MoveCategorySelect,
    ChapterView,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum UpdateOptions {
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum SourceOptions {
    Default,
    SourceSelect,
    SearchResults,
    BookView,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum HistoryOptions {
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum SettingsOptions {
    Default,
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
    pub entries: Vec<LibBookInfo>,
}

impl LibraryJson {
    pub fn new(categories: Vec<String>, entries: Vec<LibBookInfo>) -> Self {
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

        let entries: Vec<LibBookInfo> = value
            .books
            .into_values()
            .map(|v| {
                let s: Vec<LibBookInfo> = v.into();
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

#[derive(Clone, Debug, PartialEq)]
pub enum BookInfo {
    Library(LibBookInfo),
    Reader(ReaderBookInfo),
}

impl BookInfo {
    /// Creates an instance of `BookInfo` from a `Novel`
    pub fn from_novel_temp(novel: Novel) -> Result<Self, anyhow::Error> {
        let name = novel.name.clone();
        let source = BookSource::Global(GlobalBookData::create(novel));

        Ok(BookInfo::Reader(ReaderBookInfo {
            name,
            source_data: source,
        }))
    }

    pub fn is_local(&self) -> bool {
        match self {
            BookInfo::Library(d) => matches!(d.source_data, BookSource::Local(_)),
            BookInfo::Reader(d) => matches!(d.source_data, BookSource::Local(_)),
        }
    }

    pub fn get_source_data_mut(&mut self) -> &mut BookSource {
        match self {
            BookInfo::Library(d) => &mut d.source_data,
            BookInfo::Reader(d) => &mut d.source_data,
        }
    }

    pub fn get_source_data(&self) -> &BookSource {
        match self {
            BookInfo::Library(d) => &d.source_data,
            BookInfo::Reader(d) => &d.source_data,
        }
    }

    pub fn update_progress(&mut self, progress: BookProgress, chapter: Option<usize>) {
        match self {
            BookInfo::Library(d) => match d.source_data {
                BookSource::Local(ref mut data) => data.progress.progress = progress,
                BookSource::Global(ref mut data) => {
                    let c = chapter.unwrap();
                    let entry = data.chapter_progress.get_mut(&c).unwrap();
                    entry.progress = progress;
                }
            },
            BookInfo::Reader(d) => match d.source_data {
                BookSource::Local(ref mut data) => {
                    data.progress.progress = progress;
                }
                BookSource::Global(ref mut data) => {
                    let c = chapter.unwrap();
                    let entry = data.chapter_progress.get_mut(&c).unwrap();
                    entry.progress = progress;
                }
            },
        }
    }

    pub fn set_progress(&mut self, progress: BookProgressData, chapter: Option<usize>) {
        match self {
            BookInfo::Library(d) => match d.source_data {
                BookSource::Local(ref mut data) => data.progress = progress,
                BookSource::Global(ref mut data) => {
                    data.chapter_progress.insert(chapter.unwrap(), progress);
                }
            },
            BookInfo::Reader(d) => match d.source_data {
                BookSource::Local(ref mut data) => {
                    data.progress = progress;
                }
                BookSource::Global(ref mut data) => {
                    data.chapter_progress.insert(chapter.unwrap(), progress);
                }
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReaderBookInfo {
    pub name: String,
    pub source_data: BookSource,
}

/// Contains info about a book.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibBookInfo {
    /// The name of the book.
    pub name: String,
    /// Contains information that is unique to the type of source.
    pub source_data: BookSource,
    /// The category name of which the book is in. None implies that it is in the default category.
    pub category: Option<String>,
    /// The unique ID of the book.
    pub id: ID,
}

impl LibBookInfo {
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

    pub fn from_global(novel: Novel, category: Option<String>) -> Result<Self, anyhow::Error> {
        let data = GlobalBookData::create(novel);
        let source = BookSource::Global(data);
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
            BookSource::Global(d) => {
                let p = d.chapter_progress.get_mut(&d.current_chapter).unwrap();
                p.progress = progress;
            }
        }
    }

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
    pub fn get_name(&self) -> String {
        match self {
            BookSource::Local(d) => d.name.clone(),
            BookSource::Global(d) => d.name.clone(),
        }
    }

    pub fn get_chapter(&self) -> usize {
        match self {
            BookSource::Local(_) => return 0,
            BookSource::Global(d) => return d.current_chapter,
        }
    }

    pub fn get_next_chapter(&self) -> Option<usize> {
        match self {
            BookSource::Local(_) => return None,
            BookSource::Global(d) => {
                let next = d.current_chapter + 1;
                if next <= d.total_chapters {
                    Some(next)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_prev_chapter(&self) -> Option<usize> {
        match self {
            BookSource::Local(_) => return None,
            BookSource::Global(d) => {
                if d.current_chapter <= 1 {
                    return None;
                } else {
                    return Some(d.current_chapter - 1);
                }
            }
        }
    }

    pub fn set_chapter(&mut self, chapter: usize) {
        match self {
            BookSource::Local(_) => (),
            BookSource::Global(d) => d.current_chapter = chapter,
        }
    }

    /// Clears any progress on any chapters
    pub fn clear_chapter_data(&mut self) {
        match self {
            BookSource::Local(_) => (),
            BookSource::Global(d) => d.chapter_progress = HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalBookData {
    pub name: String,
    pub read_chapters: HashSet<usize>,
    pub current_chapter: usize,
    pub total_chapters: usize,
    pub chapter_progress: HashMap<usize, BookProgressData>,
    pub novel: Novel,
}

impl GlobalBookData {
    pub fn create(novel: Novel) -> Self {
        Self {
            name: novel.name.clone(),
            read_chapters: HashSet::new(),
            current_chapter: 1,
            total_chapters: novel.chapters.len(),
            chapter_progress: HashMap::new(),
            novel,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChapterDownloadData {
    pub location: String,
    pub chapter_progress: HashMap<usize, BookProgressData>,
}
