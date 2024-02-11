use crate::global::sources::Chapter;
use crate::reader::ReaderData;
use crate::state::{
    book_info::{BookInfo, BookSource},
    buffer::AppBuffer,
    channels::ChannelData,
    history::{HistoryData, HistoryEntry},
    library::LibraryData,
    menu::MenuOptions,
    screen::{CurrentScreen, HistoryOptions, LibraryOptions, SourceOptions},
};
use crate::{
    global::sources::source_data::SourceData,
    helpers::{CategoryTabs, StatefulList},
    startup,
};
use anyhow::Result;
use std::time::SystemTime;

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
