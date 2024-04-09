use std::thread;

use crate::{
    helpers::StatefulList,
    state::{
        channels::RequestData, sources::SourceNovelPreviewSelection, AppState, LibScreen, Screen,
        SourceScreen,
    },
};
use ratatui::widgets::ListState;
use termreader_core::Context;
use termreader_sources::{novel::Novel, sources::Scrape};
use thiserror::Error;

/// This module provides functions that perform the required
/// validation/setup before moving to a new screen

/// An error relating to moving to a new screen
#[derive(Error, Debug)]
pub enum EntryError {
    /// A value needed to be set before entering a screen, but it was not
    #[error("a required value was unset")]
    UnsetValue,
}

/// Enters book selection for a globally sourced book
///
/// Errors if:
/// - A book has not been selected in the library
pub fn enter_global_book_select(app_state: &mut AppState) -> Result<(), EntryError> {
    if app_state.lib_data.get_selected_book_state_mut() == &ListState::default() {
        Err(EntryError::UnsetValue)
    } else {
        app_state.update_screen(Screen::Lib(LibScreen::GlobalBookSelect));
        Ok(())
    }
}

/// Continue to read a globally sourced book
///
/// Errors if:
/// - A book has not been selected in the library
pub fn continue_reading_global_select(
    app_state: &mut AppState,
    ctx: &mut Context,
) -> Result<(), EntryError> {
    let Some(book) = app_state.lib_data.get_selected_book_mut(ctx) else {
        return Err(EntryError::UnsetValue);
    };

    let id = book.get_id();
    let novel = book.global_get_novel().clone();
    let ch = book.global_get_next_ordered_chap();

    let source_id = book.global_get_source_id();
    let source = ctx
        .source_get_by_id(source_id)
        .expect("source was taken from book but it doesn't exist")
        .clone();
    let tx = app_state.channel.get_sender();

    app_state.channel.loading = true;
    thread::spawn(move || {
        let text = source.parse_chapter(
            novel.get_url().to_string(),
            novel.get_chapter_url(ch).unwrap(),
        );
        let _ = tx.send(RequestData::Chapter((id, text, ch)));
    });

    Ok(())
}

// /// Enter the book view page for a globally sourced book
// ///
// /// Errors if:
// /// - A book has not been selected in the library
// pub fn book_view_global_select(
//     app_state: &mut AppState,
//     ctx: &mut Context,
// ) -> Result<(), EntryError> {
//     let Some(book) = app_state.lib_data.get_selected_book_mut(ctx) else {
//         return Err(EntryError::UnsetValue);
//     };

//     // Rendering uses the buffer so enter the data:
//     app_state.buffer.clear();
//     app_state.buffer.novel = Some(book.global_get_novel().clone());
//     app_state.buffer.chapter_previews =
//         StatefulList::from(book.global_get_novel().get_chapters().clone());
//     // Ensure we have either chapters or summary selected:
//     app_state.source_data.novel_preview_selected_field = SourceNovelPreviewSelection::Chapters;

//     app_state.update_screen(Screen::Lib(LibScreen::BookView));

//     Ok(())
// }

/// Start typing
pub fn enter_typing(app_state: &mut AppState) {
    app_state.buffer.text.clear();
    app_state.typing = true;
}

/// Enter the source book view
pub fn enter_source_book_view(app_state: &mut AppState, novel: Novel) {
    app_state.buffer.clear();
    app_state.source_data.novel_preview_selected_field = SourceNovelPreviewSelection::Options;
    app_state.buffer.chapter_previews = StatefulList::from(novel.get_chapters().clone());
    app_state.buffer.novel = Some(novel);
    app_state.source_data.novel_options.select_first();
    app_state.update_screen(Screen::Sources(SourceScreen::BookView))
}

/// Enter the library book view
pub fn enter_lib_book_view(app_state: &mut AppState, novel: Novel) {
    app_state.buffer.clear();
    app_state.source_data.novel_preview_selected_field = SourceNovelPreviewSelection::Chapters;
    app_state.buffer.chapter_previews = StatefulList::from(novel.get_chapters().clone());
    app_state.buffer.novel = Some(novel);
    app_state.update_screen(Screen::Lib(LibScreen::BookView))
}
