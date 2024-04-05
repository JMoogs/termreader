use crate::helpers::StatefulList;
use crate::reader::ReaderData;
use crate::state::{
    channels::ChannelData,
    menu::MenuOptions,
    screen::{HistoryOptions, LibraryScreen, Screen, SourceOptions},
};
use anyhow::Result;
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::time::SystemTime;
use termreader_core::id::ID;
use termreader_core::Context;
use termreader_sources::chapter::ChapterPreview;
use termreader_sources::novel::{Novel, NovelPreview};
use termreader_sources::{
    chapter::Chapter,
    sources::{Source, Sources},
};

pub struct AppState {
    pub context: Context,
    /// Stores the current screen that is being rendered.
    pub current_screen: Screen,
    /// Stores a list of all the previously accessed screens - the implementation of the back button.
    pub prev_screens: Vec<Screen>,
    /// Buffers for temporary data
    pub buffer: AppBuffer,
    /// Channels to send data
    pub channels: ChannelData,
    /// The main tabs
    pub tab: StatefulList<String>,
    /// Whether or not the command bar is open
    pub command_bar: bool,
    pub states: States,
}

impl AppState {
    pub fn build() -> Result<AppState> {
        let context = Context::build()?;
        Ok(AppState {
            context,
            current_screen: Screen::Library(LibraryScreen::Default),
            prev_screens: Vec::new(),
            buffer: AppBuffer::new(),
            channels: ChannelData::build(),
            tab: StatefulList::from(vec![
                String::from("Library"),
                String::from("Updates"),
                String::from("Sources"),
                String::from("History"),
                String::from("Settings"),
            ]),
            command_bar: false,
            states: States::build(&context),
        })
    }

    pub fn save(self) -> Result<()> {
        self.context.save()?;
        Ok(())
    }

    pub fn get_last_screen(&self) -> Screen {
        return *self.prev_screens.last().unwrap();
    }

    pub fn update_screen(&mut self, new: Screen) {
        if self.current_screen.in_reader() && new == Screen::Reader {
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

    pub fn move_to_Reader(&mut self) 
}

pub struct States {
    current_category_idx: usize,
    lib_category_states: HashMap<String, ListState>,
    /// The state for the window opened after selecting a book
    pub selection_box: ListState,
    pub source_selection: ListState,
    pub history_selection: ListState,
}

impl States {
    pub fn build(ctx: &Context) -> Self {
        let current_category_idx = 0;
        let mut map = HashMap::new();

        for (name, v) in ctx.library.get_books() {
            if v.len() == 0 {
                map.insert(name.to_string(), ListState::default());
            } else {
                map.insert(
                    name.to_string(),
                    ListState::default().with_selected(Some(0)),
                );
            }
        }

        Self {
            current_category_idx,
            lib_category_states: map,
            selection_box: ListState::default().with_selected(Some(0)),
            source_selection: ListState::default().with_selected(Some(0)),
            history_selection: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn get_selected_category(&mut self, ctx: &Context) -> usize {
        let max_idx = ctx.library.get_category_count() - 1;
        if self.current_category_idx > max_idx {
            self.current_category_idx = max_idx;
        }

        return self.current_category_idx;
    }

    pub fn get_category_books(&self, ctx: &Context) -> Vec<String> {
        let max_idx = ctx.library.get_category_count() - 1;
        if self.current_category_idx > max_idx {
            self.current_category_idx = max_idx;
        }

        let category = ctx.library.get_categories()[self.current_category_idx];
        let books = ctx.library.get_books().get(&category).unwrap();

        books.iter().map(|b| b.display_info()).collect()
    }

    pub fn get_category_list_state(&self, ctx: &Context) -> &mut ListState {
        let max_idx = ctx.library.get_category_count() - 1;
        if self.current_category_idx > max_idx {
            self.current_category_idx = max_idx;
        }

        let category = ctx.library.get_categories()[self.current_category_idx];

        return self.lib_category_states.get_mut(&category).unwrap();
    }

    pub fn get_selected_book(&self, ctx: &Context) -> Option<ID> {
        let category = ctx
            .library
            .get_categories()
            .get(self.current_category_idx)?;
        let book = ctx.library.get_books().get(category)?
            [self.lib_category_states.get(category)?.selected()?];

        Some(book.get_id())
    }
}

pub struct AppBuffer {
    /// Text for textboxes
    pub text: String,
    pub selection_options: Vec<String>,
    pub temp_state_unselect: ListState,
    pub temp_state_select: ListState,
    pub novel: Option<Novel>,
    pub novel_preview_selection: NovelPreviewSelection,
    pub novel_preview_scroll: usize,
    pub novel_previews: StatefulList<NovelPreview>,
    pub chapter_previews: StatefulList<ChapterPreview>,
}

#[derive(PartialEq, Eq)]
pub enum NovelPreviewSelection {
    Summary,
    Chapters,
    Options,
}

impl AppBuffer {
    pub fn new() -> Self {
        AppBuffer {
            text: String::new(),
            selection_options: Vec::new(),
            temp_state_unselect: ListState::default(),
            temp_state_select: ListState::default().with_selected(Some(0)),
            novel: None,
            novel_preview_selection: NovelPreviewSelection::Chapters,
            novel_preview_scroll: 0,
            novel_previews: StatefulList::new(),
            chapter_previews: StatefulList::new(),
        }
    }

    pub fn clear(&mut self) {
        std::mem::replace(self, AppBuffer::new());
    }
}
// // pub struct AppState {
// //     /// Manages the state of the main tabs
// //     pub current_main_tab: CategoryTabs,
// //     /// Contains all the options for all possible lists, and their states
// //     pub menu_options: MenuOptions,
// //     /// Contains data about the currently accessed source
// //     pub source_data: StatefulList<(SourceID, String)>,
// //     /// Buffers to store any temporary data
// //     pub buffer: AppBuffer,
// //     /// Contains senders/recievers for the channel, used for any synchronous operations
// //     pub channels: ChannelData,
// //     /// Represents whether the user is in the command bar or not
// //     pub command_bar: bool,
// // }

// impl AppState {
// pub fn get_last_screen(&self) -> Screen {
//     return *self.prev_screens.last().unwrap();
// }

// pub fn update_screen(&mut self, new: Screen) {
//     if self.current_screen.in_reader() && new == Screen::Reader {
//         return;
//     }
//     if self.current_screen.on_main_menu() {
//         self.prev_screens = Vec::new();
//     }
//     if new.on_main_menu() {
//         self.prev_screens = Vec::new();
//     } else {
//         self.prev_screens.push(self.current_screen);
//     }
//     self.current_screen = new;
// }

//     pub fn to_history_screen(&mut self) {
//         self.prev_screens = Vec::new();
//         self.current_screen = Screen::History(HistoryOptions::Default);
//     }

//     pub fn to_lib_screen(&mut self) {
//         self.prev_screens = Vec::new();
//         self.current_screen = Screen::Library(LibraryOptions::Default);
//     }

//     pub fn to_source_screen(&mut self) {
//         self.prev_screens = Vec::new();
//         self.current_screen = Screen::Sources(SourceOptions::Default);
//     }

//     pub fn update_category_list(&mut self) {
//         self.menu_options.category_list =
//             StatefulList::from(self.library_data.categories.items.clone());
//     }

//     pub fn build() -> Result<Self, anyhow::Error> {
//         let lib_info = startup::load_books()?;
//         let library_data = LibraryData::from(lib_info);

//         let mut history_data = startup::load_history()?;
//         if !history_data.history.is_empty() {
//             history_data.selected.select(Some(0));
//         }

//         let cats = library_data.categories.items.clone();

//         let sources = Sources::build();

//         Ok(Self {
//             current_screen: Screen::Library(LibraryOptions::Default),
//             prev_screens: Vec::new(),
//             current_main_tab: CategoryTabs::build(),
//             library_data,
//             history_data,
//             reader_data: None,
//             menu_options: MenuOptions::new(cats),
//             sources: sources.clone(),
//             source_data: StatefulList::from(sources.get_source_info()),
//             buffer: AppBuffer::default(),
//             channels: ChannelData::new(),
//             command_bar: false,
//         })
//     }

//     pub fn move_to_reader(
//         &mut self,
//         mut book: BookInfo,
//         chapter: Option<usize>,
//         text: Option<Chapter>,
//     ) -> Result<(), anyhow::Error> {
//         self.update_screen(Screen::Reader);
//         match book.get_source_data_mut() {
//             BookSource::Local(_) => {
//                 self.reader_data = Some(ReaderData::create(book, chapter, None)?)
//             }
//             BookSource::Global(_) => {
//                 self.reader_data = Some(ReaderData::create(book, chapter, text)?)
//             }
//         }

//         Ok(())
//     }

//     pub fn update_from_reader(&mut self) -> Result<()> {
//         let reader_data = if self.reader_data.is_none() {
//             return Ok(());
//         } else {
//             self.reader_data.as_mut().unwrap()
//         };

//         reader_data.set_progress()?;

//         // For updating the library, we only care if the book is in the library.
//         if let BookInfo::Library(d) = &reader_data.book_info {
//             let copy = d.clone();
//             let id = copy.id;
//             let b = self.library_data.find_book_mut(id);
//             match b {
//                 None => panic!("This should be unreachable"),
//                 Some(book) => {
//                     let _ = std::mem::replace(book, copy);
//                 }
//             }
//         }

//         // Adding to the history:
//         let info = reader_data.book_info.clone();
//         let ch = info.get_source_data().get_chapter();
//         let timestamp = {
//             let now = SystemTime::now();
//             now.duration_since(SystemTime::UNIX_EPOCH)
//                 .expect("Time has once again gone VERY backwards.")
//         }
//         .as_secs();

//         // Remove any previous instances
//         let mut index = None;
//         for (i, item) in self.history_data.history.iter().enumerate() {
//             if item.book == info {
//                 index = Some(i)
//             }
//         }
//         if let Some(idx) = index {
//             self.history_data.history.remove(idx);
//         }

//         self.history_data.history.push_front(HistoryEntry {
//             book: info,
//             timestamp,
//             chapter: ch,
//         });

//         Ok(())
//     }

//     pub fn reset_selections(&mut self) {
//         self.menu_options.global_options.select_first();
//         self.menu_options.local_options.select_first();
//         self.menu_options.source_options.select_first();
//         self.menu_options.local_history_options.select_first();
//         self.menu_options.global_history_options.select_first();
//         self.menu_options.category_list.select_first();
//         self.menu_options.source_book_options.select_first();
//         self.menu_options.category_options.select_first();
//     }

//     pub fn get_selected_source(&self) -> Option<&Source> {
//         let selected_id = self.source_data.selected();
//         match selected_id {
//             Some((id, _)) => self.get_source_by_id(id),
//             None => None,
//         }
//     }
// }

// #[derive(Default, Clone, Debug, PartialEq, Eq)]
// pub enum SourceBookBox {
//     #[default]
//     Options,
//     Chapters,
//     Summary,
// }
