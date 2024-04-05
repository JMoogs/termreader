#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Screen {
    // Main(MenuType),
    Misc(MiscOptions),
    Library(LibraryScreen),
    Updates(UpdateOptions),
    Sources(SourceOptions),
    History(HistoryOptions),
    Settings(SettingsOptions),
    Reader,
    Typing,
}

impl Screen {
    pub fn in_reader(&self) -> bool {
        matches!(self, Screen::Reader)
    }
    pub fn on_main_menu(&self) -> bool {
        match self {
            Screen::Library(LibraryScreen::Default)
            | Screen::Sources(SourceOptions::Default)
            | Screen::Updates(UpdateOptions::Default)
            | Screen::History(HistoryOptions::Default)
            | Screen::Settings(SettingsOptions::Default) => true,

            _ => false,
        }
    }

    pub fn on_library_menu(&self) -> bool {
        matches!(self, Screen::Library(LibraryScreen::Default))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum MiscOptions {
    ChapterView,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum LibraryScreen {
    Default,
    BookSelect,
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
    HistoryBookOptions,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum SettingsOptions {
    Default,
}
