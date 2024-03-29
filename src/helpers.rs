use ratatui::widgets::ListState;

#[derive(Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_item(item: T) -> StatefulList<T> {
        Self {
            state: ListState::default().with_selected(Some(0)),
            items: vec![item],
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        if items.is_empty() {
            Self {
                state: ListState::default(),
                items,
            }
        } else {
            Self {
                state: ListState::default().with_selected(Some(0)),
                items,
            }
        }
    }

    pub fn insert(&mut self, item: T) {
        self.items.push(item);
        // Now that there's something in the list, it's safe to select the first item.
        self.state = ListState::default().with_selected(Some(0));
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        };
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        };
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct CategoryTabs {
    pub tabs: Vec<String>,
    pub index: usize,
}

impl CategoryTabs {
    pub fn with_tabs(tabs: Vec<String>) -> Self {
        Self { tabs, index: 0 }
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
