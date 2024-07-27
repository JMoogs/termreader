use chrono::{TimeZone, Utc};
use ratatui::widgets::ListState;

/// A structure containing both the vector of items, `items`, as well as the state, `state`
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatefulList<T> {
    /// The state.
    state: ListState,
    /// The items of the list.
    pub items: Vec<T>,
}

impl<T> From<StatefulList<T>> for Vec<T> {
    /// Converts the list back into a vector, removing any state
    fn from(value: StatefulList<T>) -> Self {
        value.items
    }
}

impl<T> From<Vec<T>> for StatefulList<T> {
    /// Selects the first element from the list if the list has any elements
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

    /// Selects the first element, selecting none if the list is empty
    #[inline]
    pub fn select_first(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
    }

    /// Appends an item to the end of the list
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }
}

/// Converts a UNIX timestamp into a formatting date/time string.
pub fn to_datetime(timestamp_secs: u64) -> String {
    let dt = Utc.timestamp_opt(timestamp_secs as i64, 0).unwrap();

    dt.format("%d/%m/%y, %H:%M").to_string()
}
