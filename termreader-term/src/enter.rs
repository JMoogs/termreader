use std::thread;

use crate::{
    helpers::StatefulList,
    state::{
        channels::RequestData, sources::SourceNovelPreviewSelection, AppState, HistoryScreen,
        LibScreen, Screen, SourceScreen,
    },
};
use ratatui::widgets::ListState;
use termreader_core::{id::ID, Context};
use termreader_sources::{novel::Novel, sources::Scrape};
use thiserror::Error;
use tracing::Instrument;

/// This module provides functions that perform the required
/// validation/setup before moving to a new screen

/// An error relating to moving to a new screen
#[derive(Error, Debug)]
pub enum EntryError {
    /// A value needed to be set before entering a screen, but it was not
    #[error("a required value was unset")]
    UnsetValue,
    #[error("a book should have been selected in the library but was not")]
    UnselectedLibBook,
    #[error("a category should have been selected but was not")]
    UnselectedCategory,
}

/// Enters book selection for a globally sourced book
///
/// Errors if:
/// - A book has not been selected in the library
pub fn enter_global_book_select(app_state: &mut AppState) -> Result<(), EntryError> {
    if app_state.lib_data.get_selected_book_state_mut() == &ListState::default() {
        Err(EntryError::UnselectedLibBook)
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
        return Err(EntryError::UnselectedLibBook);
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

/// Start typing
pub fn enter_typing(app_state: &mut AppState) {
    app_state.buffer.text.clear();
    app_state.typing = true;
}

/// Enter the source book view
pub fn enter_source_book_view(app_state: &mut AppState, novel: Novel) {
    app_state.source_data.novel_preview_selected_field = SourceNovelPreviewSelection::Options;
    app_state.buffer.chapter_previews = StatefulList::from(novel.get_chapters().clone());
    app_state.buffer.novel = Some(novel);
    app_state.source_data.novel_options.select_first();
    app_state.update_screen(Screen::Sources(SourceScreen::BookView))
}

/// Enter the library book view
pub fn enter_lib_book_view(app_state: &mut AppState, novel: Novel) {
    app_state.source_data.novel_preview_selected_field = SourceNovelPreviewSelection::Chapters;
    app_state.buffer.chapter_previews = StatefulList::from(novel.get_chapters().clone());
    app_state.buffer.novel = Some(novel);
    app_state.update_screen(Screen::Lib(LibScreen::BookView))
}

/// Enter the history book view
pub fn enter_history_book_view(app_state: &mut AppState, novel: Novel) {
    app_state.source_data.novel_preview_selected_field = SourceNovelPreviewSelection::Chapters;
    app_state.buffer.chapter_previews = StatefulList::from(novel.get_chapters().clone());
    app_state.buffer.novel = Some(novel);
    app_state.update_screen(Screen::History(HistoryScreen::BookView))
}

pub fn enter_category_select(app_state: &mut AppState, ctx: &Context) {
    let cats = ctx.lib_get_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
    app_state.update_screen(Screen::Lib(LibScreen::CategorySelect));
}

pub fn enter_category_options(app_state: &mut AppState) {
    app_state.lib_data.category_options.select_first();
    app_state.update_screen(Screen::Lib(LibScreen::CategoryOptions));
}

pub fn move_book_category(app_state: &mut AppState, ctx: &mut Context) -> Result<(), EntryError> {
    let Some(b) = app_state.lib_data.get_selected_book(ctx) else {
        return Err(EntryError::UnselectedLibBook);
    };
    let b = b.get_id();

    let cat_idx = app_state.buffer.temporary_list.selected_idx();
    let Some(cat_idx) = cat_idx else {
        return Err(EntryError::UnselectedCategory);
    };
    // .expect("there should always be a category selected");

    ctx.lib_move_book_category(b, Some(&ctx.lib_get_categories()[cat_idx].clone()));

    app_state.lib_data.reset_selection(ctx);
    app_state.lib_data.global_selected_book_opts.select_first();
    app_state.update_screen(Screen::Lib(LibScreen::Main));
    Ok(())
}

pub fn delete_category(
    app_state: &mut AppState,
    ctx: &mut Context,
    category_name: String,
) -> Result<(), ()> {
    if ctx.lib_delete_category(category_name).is_ok() {
        let cats = ctx.lib_get_categories().clone();
        app_state.buffer.temporary_list = StatefulList::from(cats);
        Ok(())
    } else {
        Err(())
    }
}

pub fn create_category(
    app_state: &mut AppState,
    ctx: &mut Context,
    category_name: String,
) -> Result<(), ()> {
    if ctx.lib_create_category(category_name).is_ok() {
        let cats = ctx.lib_get_categories().clone();
        app_state.buffer.temporary_list = StatefulList::from(cats);
        Ok(())
    } else {
        Err(())
    }
}

pub fn rename_category(app_state: &mut AppState, ctx: &mut Context, new_name: String) {
    let old_name = app_state
        .buffer
        .temporary_list
        .selected()
        .expect("a category should always be selected");
    ctx.lib_rename_category(old_name.to_string(), new_name);

    let cats = ctx.lib_get_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
}
