use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use crate::global::sources::{Chapter, SourceID};
use crate::reader::buffer::{BookProgress, BookProgressData};
use crate::reader::ReaderData;
use crate::state::{
    buffer::AppBuffer,
    channels::ChannelData,
    history::{HistoryData, HistoryEntry},
    library::{LibBookInfo, LibraryData},
};
use crate::{
    global::sources::{source_data::SourceData, Novel},
    helpers::{CategoryTabs, StatefulList},
    local::LocalBookData,
    startup,
};

pub struct AppState {
    /// Stores the current screen that is being rendered.
    pub current_screen: CurrentScreen,
    /// Stores a list of all the previously accessed screens - the implementation of the back button.
    pub prev_screens: Vec<CurrentScreen>,
    /// Manages the state of the main tabs
    pub current_main_tab: CategoryTabs,
    /// Stores all data related to the book library
    pub library_data: LibraryData,
    /// Stores all data related to the reader itself, only intialized if the user has entered a book at least once in their session
    pub reader_data: Option<ReaderData>,
    /// Stores reading history
    pub history_data: HistoryData,
    /// Contains all the options for all possible lists, and their states
    pub menu_options: MenuOptions,
    /// Contains data about the currently accessed source
    pub source_data: SourceData,
    /// Buffers to store any temporary data
    pub buffer: AppBuffer,
    /// Contains senders/recievers for the channel, used for any synchronous operations
    pub channels: ChannelData,
    /// Represents whether the user is in the command bar or not
    pub command_bar: bool,
}

pub struct MenuOptions {
    pub local_options: StatefulList<String>,
    pub global_options: StatefulList<String>,
    pub local_history_options: StatefulList<String>,
    pub global_history_options: StatefulList<String>,
    pub category_list: StatefulList<String>,
    pub source_options: StatefulList<String>,
    pub source_book_options: StatefulList<String>,
    pub category_options: StatefulList<String>,
}

impl MenuOptions {
    fn new(categories: Vec<String>) -> Self {
        let local_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("Move to category..."),
            String::from("Rename"),
            String::from("Start from beginning"),
            String::from("Remove book from library"),
        ]);

        let global_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("View chapter list"),
            String::from("Move to category..."),
            String::from("Rename"),
            String::from("Start from beginning"),
            String::from("Remove book from library"),
        ]);

        let local_history_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("Remove from history"),
        ]);

        let global_history_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("View book"),
            String::from("Remove from history"),
        ]);

        let category_list = StatefulList::from(categories);

        let source_options =
            StatefulList::from(vec![String::from("Search"), String::from("View popular")]);

        let source_book_options = StatefulList::from(vec![
            String::from("Start from beginning"),
            String::from("Add book to library"),
            // String::from("View chapters"),
        ]);

        let category_options = StatefulList::from(vec![
            String::from("Create categories"),
            String::from("Re-order categories (Not yet implemented)"),
            String::from("Rename categories"),
            String::from("Delete categories"),
        ]);

        Self {
            local_options,
            global_options,
            local_history_options,
            global_history_options,
            category_list,
            source_options,
            source_book_options,
            category_options,
        }
    }
}

impl AppState {
    pub fn get_last_screen(&self) -> CurrentScreen {
        return *self.prev_screens.last().unwrap();
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

    pub fn to_history_screen(&mut self) {
        self.prev_screens = Vec::new();
        self.current_screen = CurrentScreen::History(HistoryOptions::Default);
    }

    pub fn to_lib_screen(&mut self) {
        self.prev_screens = Vec::new();
        self.current_screen = CurrentScreen::Library(LibraryOptions::Default);
    }

    pub fn to_source_screen(&mut self) {
        self.prev_screens = Vec::new();
        self.current_screen = CurrentScreen::Sources(SourceOptions::Default);
    }

    pub fn update_category_list(&mut self) {
        self.menu_options.category_list =
            StatefulList::from(self.library_data.categories.items.clone());
    }

    pub fn build() -> Result<Self, anyhow::Error> {
        let lib_info = startup::load_books()?;
        let library_data = LibraryData::from(lib_info);

        let mut history_data = startup::load_history()?;
        if !history_data.history.is_empty() {
            history_data.selected.select(Some(0));
        }

        let cats = library_data.categories.items.clone();

        Ok(Self {
            current_screen: CurrentScreen::Library(LibraryOptions::Default),
            prev_screens: Vec::new(),
            current_main_tab: CategoryTabs::build(),
            library_data,
            history_data,
            reader_data: None,
            menu_options: MenuOptions::new(cats),
            source_data: SourceData::build(),
            buffer: AppBuffer::default(),
            channels: ChannelData::new(),
            command_bar: false,
        })
    }

    pub fn move_to_reader(
        &mut self,
        mut book: BookInfo,
        chapter: Option<usize>,
        text: Option<Chapter>,
    ) -> Result<(), anyhow::Error> {
        self.update_screen(CurrentScreen::Reader);
        match book.get_source_data_mut() {
            BookSource::Local(_) => {
                self.reader_data = Some(ReaderData::create(book, chapter, None)?)
            }
            BookSource::Global(_) => {
                self.reader_data = Some(ReaderData::create(book, chapter, text)?)
            }
        }

        Ok(())
    }

    pub fn update_from_reader(&mut self) -> Result<()> {
        let reader_data = if self.reader_data.is_none() {
            return Ok(());
        } else {
            self.reader_data.as_mut().unwrap()
        };

        reader_data.set_progress()?;

        // For updating the library, we only care if the book is in the library.
        if let BookInfo::Library(d) = &reader_data.book_info {
            let copy = d.clone();
            let id = copy.id;
            let b = self.library_data.find_book_mut(id);
            match b {
                None => panic!("This should be unreachable"),
                Some(book) => {
                    let _ = std::mem::replace(book, copy);
                }
            }
        }

        // Adding to the history:
        let info = reader_data.book_info.clone();
        let ch = info.get_source_data().get_chapter();
        let timestamp = {
            let now = SystemTime::now();
            now.duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time has once again gone VERY backwards.")
        }
        .as_secs();

        // Remove any previous instances
        let mut index = None;
        for (i, item) in self.history_data.history.iter().enumerate() {
            if item.book == info {
                index = Some(i)
            }
        }
        if let Some(idx) = index {
            self.history_data.history.remove(idx);
        }

        self.history_data.history.push_front(HistoryEntry {
            book: info,
            timestamp,
            chapter: ch,
        });

        Ok(())
    }

    pub fn reset_selections(&mut self) {
        self.menu_options.global_options.select_first();
        self.menu_options.local_options.select_first();
        self.menu_options.source_options.select_first();
        self.menu_options.local_history_options.select_first();
        self.menu_options.global_history_options.select_first();
        self.menu_options.category_list.select_first();
        self.menu_options.source_book_options.select_first();
        self.menu_options.category_options.select_first();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum CurrentScreen {
    // Main(MenuType),
    Misc(MiscOptions),
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
            | CurrentScreen::Settings(SettingsOptions::Default) => true,

            _ => false,
        }
    }

    pub fn on_library_menu(&self) -> bool {
        matches!(self, CurrentScreen::Library(LibraryOptions::Default))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum MiscOptions {
    ChapterView,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum LibraryOptions {
    Default,
    LocalBookSelect,
    GlobalBookSelect,
    CategorySelect,
    CategoryOptions,
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
    HistoryLocalBookOptions,
    HistoryGlobalBookOptions,
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
pub enum BookInfo {
    Library(LibBookInfo),
    Reader(ReaderBookInfo),
}

impl BookInfo {
    /// Creates an instance of `BookInfo` from a `Novel`
    pub fn from_novel_temp(novel: Novel, ch: usize) -> Result<Self, anyhow::Error> {
        let name = novel.name.clone();
        let source = BookSource::Global(GlobalBookData::create(novel, ch));

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

    pub fn get_progress(&self) -> BookProgress {
        let source_data = self.get_source_data();
        match source_data {
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

    pub fn get_source_id(&self) -> Option<SourceID> {
        match self.get_source_data() {
            BookSource::Local(_) => None,
            BookSource::Global(d) => Some(d.novel.source),
        }
    }

    pub fn get_novel(&self) -> Option<&Novel> {
        match self.get_source_data() {
            BookSource::Local(_) => None,
            BookSource::Global(d) => Some(&d.novel),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReaderBookInfo {
    pub name: String,
    pub source_data: BookSource,
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
            BookSource::Local(_) => 0,
            BookSource::Global(d) => d.current_chapter,
        }
    }

    pub fn get_next_chapter(&self) -> Option<usize> {
        match self {
            BookSource::Local(_) => None,
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
            BookSource::Local(_) => None,
            BookSource::Global(d) => {
                if d.current_chapter <= 1 {
                    None
                } else {
                    Some(d.current_chapter - 1)
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalBookData {
    pub name: String,
    pub read_chapters: HashSet<usize>,
    pub current_chapter: usize,
    pub total_chapters: usize,
    pub chapter_progress: HashMap<usize, BookProgressData>,
    pub novel: Novel,
}

impl PartialEq for GlobalBookData {
    fn eq(&self, other: &Self) -> bool {
        self.novel == other.novel
    }
}

impl GlobalBookData {
    pub fn create(novel: Novel, ch: usize) -> Self {
        Self {
            name: novel.name.clone(),
            read_chapters: HashSet::new(),
            current_chapter: ch,
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
