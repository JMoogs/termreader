use chrono::{TimeZone, Utc};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

/// A structure containing both the vector of items, `items`, as well as the state, `state`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct StatefulList<T> {
    /// The state.
    #[serde(skip)]
    state: ListState,
    /// The items of the list.
    pub items: Vec<T>,
}

impl<T> From<StatefulList<T>> for Vec<T> {
    fn from(value: StatefulList<T>) -> Self {
        value.items
    }
}

impl<T> From<Vec<T>> for StatefulList<T> {
    /// Selects the first element from the list if the list has any elements.
    fn from(value: Vec<T>) -> Self {
        if value.len() > 0 {
            StatefulList {
                state: ListState::default().with_selected(Some(0)),
                items: value,
            }
        } else {
            StatefulList {
                state: ListState::default(),
                items: value,
            }
        }
    }
}

impl<T> StatefulList<T> {
    /// Creates an empty StatefulList
    pub fn new() -> StatefulList<T> {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    /// Gives a mutable reference to the state of the list.
    /// This function should only be required when calling `render_stateful_widget`
    #[inline]
    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    /// Selects the first element of the list.
    /// This function does not check whether the list has any elements.
    #[inline]
    pub fn select_first(&mut self) {
        self.state.select(Some(0));
    }

    /// Unselects any selected list element.
    #[inline]
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    /// Returns a reference to the selected element, or none if no element is selected.
    #[inline]
    pub fn selected(&self) -> Option<&T> {
        let idx = self.state.selected();
        match idx {
            Some(i) => Some(&self.items[i]),
            None => None,
        }
    }

    /// Returns the index of the selected element.
    #[inline]
    pub fn selected_idx(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Creates a StatefulList with a single element, which is selected.
    pub fn with_item(item: T) -> StatefulList<T> {
        Self {
            state: ListState::default().with_selected(Some(0)),
            items: vec![item],
        }
    }

    /// Inserts an element into the list. If the list was previously empty, the first element is selected.
    pub fn insert(&mut self, item: T) {
        self.items.push(item);
        // Now that there's something in the list, it's safe to select the first item.
        if self.state == ListState::default() {
            self.state = ListState::default().with_selected(Some(0));
        }
    }

    /// Selects the next element in the list.
    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            let i = match self.state.selected() {
                Some(idx) => (idx + 1) % self.items.len(),
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    /// Selects the previous element in the list.
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
        } else {
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
    }

    /// Converts the list back to a vector, losing state.
    pub fn to_vec(self) -> Vec<T> {
        self.items
    }
}

/// A struct to store the list of categories.
// Ideally, this struct could be removed entirely.
#[derive(Clone, Debug)]
pub struct CategoryTabs {
    list: StatefulList<String>,
}

// In all cases here, the call to unwrap() is safe as a main tab is always selected.
impl CategoryTabs {
    /// Builds the struct with the correct category names.
    pub fn build() -> Self {
        Self {
            list: StatefulList::from(vec![
                String::from("Library"),
                String::from("Updates"),
                String::from("Sources"),
                String::from("History"),
                String::from("Settings"),
            ]),
        }
    }

    /// Cycles to the next selection
    pub fn next(&mut self) {
        self.list.next();
    }

    /// Cycles to the previous selection
    pub fn previous(&mut self) {
        self.list.previous();
    }

    /// Returns the list of tab names
    pub fn tabs(&self) -> &Vec<String> {
        &self.list.items
    }

    /// Returns the index of the selected tab
    pub fn selected_idx(&self) -> usize {
        self.list.selected_idx().unwrap()
    }

    /// Returns true when in the library tab
    pub fn in_library(&self) -> bool {
        self.list.selected_idx().unwrap() == 0
    }

    /// Returns true when in the updates tab
    pub fn in_updates(&self) -> bool {
        self.list.selected_idx().unwrap() == 1
    }

    /// Returns true when in the sources tab
    pub fn in_sources(&self) -> bool {
        self.list.selected_idx().unwrap() == 2
    }

    /// Returns true when in the history tab
    pub fn in_history(&self) -> bool {
        self.list.selected_idx().unwrap() == 3
    }

    /// Returns true when in the settings tab
    pub fn in_settings(&self) -> bool {
        self.list.selected_idx().unwrap() == 4
    }
}

/// Converts a UNIX timestamp into a formatting date/time string.
pub fn to_datetime(timestamp_secs: u64) -> String {
    let dt = Utc.timestamp_opt(timestamp_secs as i64, 0).unwrap();

    dt.format("%d/%m/%y, %H:%M").to_string()
}
