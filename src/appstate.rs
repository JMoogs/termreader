pub struct AppState {
    pub(crate) current_screen: CurrentScreen,
    pub(crate) current_tab: MainScreenTabs,
}

impl AppState {
    pub fn build() -> Self {
        Self {
            current_screen: CurrentScreen::Main,
            current_tab: MainScreenTabs::build(),
        }
    }
}

pub(crate) struct MainScreenTabs {
    pub(crate) tabs: Vec<String>,
    pub(crate) index: usize,
}

impl MainScreenTabs {
    fn build() -> Self {
        let v = vec![
            String::from("Library"),
            String::from("Updates"),
            String::from("Sources"),
            String::from("History"),
            String::from("Settings"),
        ];

        Self { tabs: v, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.tabs.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum CurrentScreen {
    Main,
    Reader,
    ExitingReader,
}

// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
// pub enum MainTabs {
//     Library,
//     Updates,
//     Sources,
//     History,
//     Settings,
// }

// impl MainTabs {
//     /// Gets the next tab in the sequence. Wraps round
//     pub fn get_next_tab(&mut self) -> Self {
//         match self {
//             Self::Library => Self::Updates,
//             Self::Updates => Self::Sources,
//             Self::Sources => Self::History,
//             Self::History => Self::Settings,
//             Self::Settings => Self::Library,
//         }
//     }

//     /// Gets the previous tab in the sequence. Wraps round
//     pub fn get_prev_tab(&mut self) -> Self {
//         match self {
//             Self::Library => Self::Settings,
//             Self::Updates => Self::Library,
//             Self::Sources => Self::Updates,
//             Self::History => Self::Sources,
//             Self::Settings => Self::History,
//         }
//     }
// }
