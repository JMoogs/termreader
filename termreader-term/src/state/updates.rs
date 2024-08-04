use ratatui::widgets::ListState;
use termreader_core::Context;

pub struct UpdatesData {
    /// The currently selected updates entry
    selected_entry: ListState,
}

impl UpdatesData {
    pub fn build(ctx: &Context) -> Self {
        let selected_entry = if ctx.updates_get_len() == 0 {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };

        Self { selected_entry }
    }

    /// Returns a mutable reference to the state representing the selected history entry. This will always succeed. This function should **not** be used directly
    pub fn get_selected_entry_mut(&mut self) -> &mut ListState {
        &mut self.selected_entry
    }

    pub fn select_next_entry(&mut self, ctx: &Context) {
        if ctx.updates_get_len() == 0 {
            self.selected_entry.select(None);
            return;
        }
        match self.selected_entry.selected() {
            Some(s) => {
                self.selected_entry
                    .select(Some((s + 1) % ctx.updates_get_len()));
            }
            None => {
                if ctx.updates_get_len() != 0 {
                    self.selected_entry.select(Some(0));
                }
            }
        }
    }

    pub fn select_prev_entry(&mut self, ctx: &Context) {
        match self.selected_entry.selected() {
            Some(s) => {
                if s == 0 {
                    if ctx.updates_get_len() != 0 {
                        self.selected_entry.select(Some(ctx.updates_get_len() - 1));
                    }
                } else {
                    self.selected_entry.select(Some(s - 1));
                }
            }
            None => {
                let len = ctx.updates_get_len();
                if len != 0 {
                    self.selected_entry.select(Some(len - 1));
                }
            }
        }
    }
}
