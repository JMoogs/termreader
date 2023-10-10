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
