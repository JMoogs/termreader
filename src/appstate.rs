pub struct AppState {
    pub(crate) current_screen: CurrentScreen,
    pub(crate) current_tab: Option<MainTabs>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::Main,
            current_tab: Some(MainTabs::Library),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum CurrentScreen {
    Main,
    Reader,
    ExitingReader,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum MainTabs {
    Library,
    Updates,
    Sources,
    History,
    Settings,
}

impl MainTabs {
    /// Gets the next tab in the sequence. Wraps round
    pub fn get_next_tab(&mut self) -> Self {
        match self {
            Self::Library => Self::Updates,
            Self::Updates => Self::Sources,
            Self::Sources => Self::History,
            Self::History => Self::Settings,
            Self::Settings => Self::Library,
        }
    }

    /// Gets the previous tab in the sequence. Wraps round
    pub fn get_prev_tab(&mut self) -> Self {
        match self {
            Self::Library => Self::Settings,
            Self::Updates => Self::Library,
            Self::Sources => Self::Updates,
            Self::History => Self::Sources,
            Self::Settings => Self::History,
        }
    }
}
