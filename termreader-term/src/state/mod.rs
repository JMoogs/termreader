use crate::helpers::StatefulList;
use crate::state::reader::ReaderData;
use crate::trace_dbg;
use std::time::SystemTime;
use termreader_core::book::Book;
use termreader_core::Context;
use termreader_sources::chapter::Chapter;

use self::buffer::Buffer;
use self::channels::ChannelData;
use self::history::HistoryData;
use self::library::LibData;
use self::sources::SourceData;

pub mod buffer;
pub mod channels;
pub mod history;
pub mod library;
pub mod reader;
pub mod sources;

/// The state of the TUI.
/// Contains all data required to run the TUI.
/// Data relating to the core function of the program is in `termreader_core::Context`
pub struct AppState {
    /// Whether or not the user has requested to quit the program
    pub quit: bool,
    /// The current screen the user is viewing
    pub screen: Screen,
    /// The previously viewed screens. This is used to implement a back button among other things
    pub prev_screens: Vec<Screen>,
    /// Channel data for multi-threaded requests
    pub channel: ChannelData,
    /// The possible menu tabs
    pub menu_tabs: StatefulList<String>,
    /// Data related to the library tab
    pub lib_data: LibData,
    /// Data related to the sources tab
    pub source_data: SourceData,
    /// Data related to the history tab
    pub history_data: HistoryData,
    /// Data from the reader
    pub reader_data: ReaderData,
    /// A buffer for temporary values
    pub buffer: Buffer,
    /// A boolean representing whether the user is in a command bar or not
    pub command_bar: bool,
    /// A boolean representing whether the user is typing or not
    pub typing: bool,
}

impl AppState {
    /// Creates the state of the TUI. For use on startup
    pub fn build(ctx: &Context) -> Self {
        Self {
            quit: false,
            screen: Screen::Lib(LibScreen::Main),
            prev_screens: Vec::new(),
            channel: ChannelData::build(),
            menu_tabs: StatefulList::from(vec![
                String::from("Library"),
                String::from("Updates"),
                String::from("Sources"),
                String::from("History"),
                String::from("Settings"),
            ]),
            lib_data: LibData::build(ctx),
            source_data: SourceData::build(),
            history_data: HistoryData::build(ctx),
            reader_data: ReaderData::build(),
            buffer: Buffer::build(),
            command_bar: false,
            typing: false,
        }
    }

    /// Moves the user into a new menu/screen
    pub fn update_screen(&mut self, new: Screen) {
        // Return early if the screen doesn't change
        if self.screen == new {
            return;
        }
        // Whenever the user moves into a "main" screen, we can get rid of screen history
        let main = vec![
            Screen::Lib(LibScreen::Main),
            Screen::Updates(UpdateScreen::Main),
            Screen::Sources(SourceScreen::Main),
            Screen::History(HistoryScreen::Main),
            Screen::Settings(SettingsScreen::Main),
        ];
        let selections = vec![Screen::Sources(SourceScreen::Select)];
        if main.contains(&new) {
            self.prev_screens = Vec::new();
        } else if !selections.contains(&self.screen) {
            self.prev_screens.push(self.screen);
        }
        self.screen = new;
    }

    /// Returns whether or not the current screen is a main screen
    pub fn in_main_screen(&self) -> bool {
        vec![
            Screen::Lib(LibScreen::Main),
            Screen::Updates(UpdateScreen::Main),
            Screen::Sources(SourceScreen::Main),
            Screen::History(HistoryScreen::Main),
            Screen::Settings(SettingsScreen::Main),
        ]
        .contains(&self.screen)
    }

    /// Moves the user into the reader
    pub fn move_to_reader(&mut self, book: Book, chapter: Option<Chapter>) {
        self.reader_data.set_data(book, chapter);
        self.prev_screens = Vec::new();
        self.screen = Screen::Reader;
    }

    pub fn update_from_reader(&mut self, ctx: &mut Context) {
        // Do nothing if we haven't got a book
        if self.reader_data.get_book().is_none() {
            return;
        }
        let mut b = self.reader_data.get_book_mut().as_mut().unwrap().clone();

        // Set the chapter progress
        if !b.is_local() {
            let current_ch = b.global_get_current_ch();
            b.global_set_progress(
                self.reader_data
                    .get_ch_progress()
                    .expect("A book should be selected at this point"),
                current_ch,
            );
            trace_dbg!("Set chapter progress");
            trace_dbg!(self.reader_data.get_ch_progress().unwrap());
        } else {
            todo!()
        }

        // If the book is in library then update the copy in the lib
        // FIXME: Check that this isn't just an empty copy
        if let Some(book) = ctx.lib_find_book_mut(b.get_id()) {
            let _ = std::mem::replace(book, b.clone());
            trace_dbg!("Updated book with copy");
        }

        // Add to the history

        // Remove any entries with the same ID
        // FIXME: This should probably be done on another field as temporary
        // books are assigned an ID that will end up being unique.
        // Maybe do it on the full novel link (hash works for local books)
        ctx.hist_remove_entry(b.get_id());

        let timestamp = {
            let now = SystemTime::now();
            now.duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time has gone very backwards...")
        }
        .as_secs();
        ctx.hist_add_entry(b.clone(), timestamp);
        // We added to history so we can select the first entry
        // (it doesn't matter if there's already one selected or not)
        self.history_data.get_selected_entry_mut().select(Some(0));
    }
}

/// An enum representing the current screen the user is viewing
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Reader,
    Lib(LibScreen),
    Updates(UpdateScreen),
    Sources(SourceScreen),
    History(HistoryScreen),
    Settings(SettingsScreen),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LibScreen {
    Main,
    GlobalBookSelect,
    BookView,
    CategorySelect,
    CategoryOptions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UpdateScreen {
    Main,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceScreen {
    Main,
    SearchRes,
    Select,
    BookView,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HistoryScreen {
    Main,
    BookView,
    LocalBookOptions,
    GlobalBookOptions,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsScreen {
    Main,
}
