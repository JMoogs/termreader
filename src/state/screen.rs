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
