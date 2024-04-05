// pub struct LibStates {
//     current_category_idx: usize,
//     selected_book: ListState,
// }

// impl LibStates {
//     fn build() -> Self {
//         Self {
//             current_category_idx: 0,
//             selected_book: ListState::default(),
//         }
//     }

//     /// Get the current selected category. This function ensures that the index is always within bounds
//     pub fn get_selected_category(&mut self, ctx: &Context) -> usize {
//         let max_idx = ctx.lib_get_categories().len() - 1;
//         if self.current_category_idx > max_idx {
//             self.current_category_idx = max_idx;
//         }
//         self.current_category_idx = max_idx;

//         self.current_category_idx
//     }

//     /// Selects the next category, wrapping around as required
//     pub fn select_next_category(&mut self, ctx: &Context) {
//         let list_len = ctx.lib_get_categories().len();

//         self.current_category_idx = (self.current_category_idx + 1) % list_len;
//     }

//     /// Selects the previous category, wrapping around as required
//     pub fn select_previous_category(&mut self, ctx: &Context) {
//         let max_idx = ctx.lib_get_categories().len() - 1;

//         if self.current_category_idx == 0 {
//             self.current_category_idx = max_idx;
//         } else {
//             self.current_category_idx -= 1;
//         }
//     }

//     //
//     pub fn get_selected_book_mut(&mut self, books_len: usize) -> &mut ListState {
//         match self.selected_book.selected() {
//             Some(s) => {
//                 if books_len == 0 {
//                     self.selected_book.select(None)
//                 } else if s >= books_len {
//                     self.selected_book.select(Some(0))
//                 };

//                 &mut self.selected_book
//             }
//             None => {
//                 if books_len > 0 {
//                     self.selected_book.select(Some(0));
//                 }
//                 &mut self.selected_book
//             }
//         }
//     }

//     pub fn select_next_book(&mut self, ctx: &Context) {
//         let size = self.get_current_category_size(ctx);
//         match self.selected_book.selected() {
//             Some(s) => {
//                 if size == 0 {
//                     self.selected_book.select(None)
//                 } else {
//                     self.selected_book.select(Some((s + 1) % size))
//                 }
//             }
//             None => {
//                 if size > 0 {
//                     self.selected_book.select(Some(0))
//                 }
//             }
//         }
//     }

//     pub fn select_prev_book(&mut self, ctx: &Context) {
//         let size = self.get_current_category_size(ctx);
//         match self.selected_book.selected() {
//             Some(s) => {
//                 if size == 0 {
//                     self.selected_book.select(None)
//                 } else if s == 0 {
//                     self.selected_book.select(Some(size - 1));
//                 } else {
//                     self.selected_book.select(Some(s - 1));
//                 }
//             }
//             None => {
//                 if size > 0 {
//                     self.selected_book.select(Some(size - 1));
//                 }
//             }
//         }
//     }

//     pub fn reset_selected_book(&mut self) {
//         self.selected_book.select(None);
//     }

//     pub fn get_current_category_size(&mut self, ctx: &Context) -> usize {
//         let cat_name = &ctx.lib_get_categories()[self.get_selected_category(ctx)];
//         ctx.lib_get_books().get(cat_name).unwrap().len()
//     }
// }

// pub struct SourceStates {
//     selected_source: ListState,
// }

// impl SourceStates {
//     fn build() -> Self {
//         Self {
//             selected_source: ListState::default().with_selected(Some(0)),
//         }
//     }

//     pub fn get_selected_source_mut(&mut self) -> &mut ListState {
//         // There will always be at least one source so it should be safe to simply give out the reference
//         &mut self.selected_source
//     }

//     pub fn select_next(&mut self, ctx: &Context) {
//         let size = ctx.source_get_info().len();
//         let sel = self.selected_source.selected().unwrap();
//         self.selected_source.select(Some((sel + 1) % size))
//     }

//     pub fn select_prev(&mut self, ctx: &Context) {
//         let size = ctx.source_get_info().len();
//         let sel = self.selected_source.selected().unwrap();
//         if sel == 0 {
//             self.selected_source.select(Some(size - 1));
//         } else {
//             self.selected_source.select(Some(sel - 1));
//         }
//     }

//     pub fn get_selected_source_id(&self, ctx: &Context) -> SourceID {
//         let idx = self.selected_source.selected().unwrap();
//         ctx.source_get_info()[idx].0
//     }
// }

// pub struct HistoryStates {
//     selected_entry: ListState,
// }

// impl HistoryStates {
//     fn build() -> Self {
//         Self {
//             selected_entry: ListState::default().with_selected(Some(0)),
//         }
//     }

//     pub fn get_selected_entry_mut(&mut self, entries_len: usize) -> &mut ListState {
//         if entries_len == 0 {
//             self.selected_entry.select(None);
//         }
//         // If nothing is selected and there's something to select, select it.
//         // If something is selected, but there is nothing to select, unselect.
//         // If something is selected, but it is out of bounds, and there is something to select, select the first thing.
//         match self.selected_entry.selected() {
//             Some(s) => {
//                 if entries_len == 0 {
//                     self.selected_entry.select(None)
//                 } else if s >= entries_len {
//                     self.selected_entry.select(Some(0))
//                 }
//             }
//             None => {
//                 if entries_len > 0 {
//                     self.selected_entry.select(Some(0))
//                 }
//             }
//         }
//         &mut self.selected_entry
//     }
// }

// #[derive(PartialEq, Eq, Clone, Copy)]
// pub enum NovelPreviewSelection {
//     Summary,
//     Chapters,
//     Options,
// }

// pub struct Buffer {
//     pub text: String,
//     pub selection_options: Vec<String>,
//     pub selection_title: String,
//     pub temp_state: ListState,
//     pub novel_previews: StatefulList<NovelPreview>,
//     pub novel: Option<Novel>,
//     pub novel_preview_selection: NovelPreviewSelection,
//     pub novel_preview_scroll: usize,
//     pub chapter_previews: StatefulList<ChapterPreview>,
// }

// impl Buffer {
//     fn build() -> Self {
//         Self {
//             text: String::default(),
//             selection_options: Vec::new(),
//             selection_title: String::new(),
//             temp_state: ListState::default(),
//             novel_previews: StatefulList::new(),
//             novel: None,
//             novel_preview_selection: NovelPreviewSelection::Chapters,
//             novel_preview_scroll: 0,
//             chapter_previews: StatefulList::new(),
//         }
//     }

//     pub fn clear(&mut self) {
//         let _ = std::mem::replace(self, Buffer::build());
//     }

//     pub fn set_selection(&mut self, title: String, options: Vec<String>) {
//         self.selection_title = title;
//         self.selection_options = options;
//         if self.selection_options.len() > 0 {
//             self.temp_state = ListState::default().with_selected(Some(0))
//         } else {
//             self.temp_state = ListState::default()
//         }
//     }

//     pub fn select_next(&mut self) {
//         let size = self.selection_options.len();
//         match self.temp_state.selected() {
//             Some(s) => {
//                 if size == 0 {
//                     self.temp_state.select(None)
//                 } else {
//                     self.temp_state.select(Some((s + 1) % size))
//                 }
//             }
//             None => {
//                 if size > 0 {
//                     self.temp_state.select(Some(0))
//                 }
//             }
//         };
//     }

//     pub fn select_prev(&mut self) {
//         let size = self.selection_options.len();
//         match self.temp_state.selected() {
//             Some(s) => {
//                 if size == 0 {
//                     self.temp_state.select(None)
//                 } else if s == 0 {
//                     self.temp_state.select(Some(size - 1))
//                 } else {
//                     self.temp_state.select(Some(s - 1))
//                 }
//             }
//             None => {
//                 if size > 0 {
//                     self.temp_state.select(Some(size - 1))
//                 }
//             }
//         }
//     }
// }

// /// Contains all the possible requests that can be made through channels.
// /// These are requests that need to happen synchronously
// pub enum RequestData {
//     /// The results of a search
//     SearchResults(Result<Vec<NovelPreview>>),
//     /// Info about a novel. The bool signifies whether or not to show an options menu.
//     BookInfo((Result<Novel>, bool)),
//     // /// A chapter
//     // Chapter((ID, Result<Chapter>, usize)),
//     // /// Update info for a book
//     // Updated(Book),
// }

// pub struct ChannelData {
//     sender: Sender<RequestData>,
//     pub reciever: Receiver<RequestData>,
//     pub loading: bool,
// }

// impl ChannelData {
//     /// Creates a new channel
//     pub fn build() -> Self {
//         let (sender, reciever) = std::sync::mpsc::channel();
//         Self {
//             sender,
//             reciever,
//             loading: false,
//         }
//     }

//     /// Get a sender for the channel
//     pub fn get_sender(&self) -> Sender<RequestData> {
//         return self.sender.clone();
//     }
// }
