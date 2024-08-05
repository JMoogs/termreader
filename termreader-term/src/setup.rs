use std::thread;

use crate::{
    helpers::StatefulList,
    state::{
        channels::{BookInfo, BookInfoDetails, RequestData},
        sources::SourceNovelPreviewSelection,
        AppState, HistoryScreen, LibScreen, Screen, SourceScreen,
    },
    ui::sources::BookViewOption,
};
use termreader_core::{book::BookRef, history::HistoryEntry, id::ID, Context};
use termreader_sources::{
    novel::NovelPreview,
    sources::{Scrape, SortOrder, SourceID},
};
use thiserror::Error;

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
    #[error("the selected book has no chapters")]
    NoChapters,
    #[error("a category should have been selected but was not")]
    UnselectedCategory,
}

/// An error relating to a book
#[derive(Error, Debug)]
pub enum BookError {
    /// The book does not exist
    #[error("the book does not exist")]
    NonExistent,
    /// The book has no available chapters
    #[error("the book has no available chapters")]
    NoChapters,
    /// The chapter does not exist
    #[error("the requested chapter is unavailable")]
    UnavailableChapter,
}

/// An error relating to a source
#[derive(Error, Debug)]
pub enum SourceError {
    /// The source does not exist
    #[error("the source does not exist")]
    NonExistent,
}

/// Continue to read a globally sourced book
///
/// Errors if:
/// - A book has not been selected in the library
/// - A book has no chapters
pub fn continue_reading_global_select(
    app_state: &mut AppState,
    ctx: &mut Context,
) -> Result<(), EntryError> {
    let Some(mut book) = app_state.lib_data.get_selected_book(ctx) else {
        return Err(EntryError::UnselectedLibBook);
    };

    let id = book.get_id();
    let ch = match book.global_get_next_ordered_chap() {
        Some(c) => c,
        None => return Err(EntryError::NoChapters),
    };

    let source = ctx.get_book_source(id).unwrap().clone();
    let novel_path = book.get_url().unwrap();
    let chapter_path = book.get_chapter_url(ch).unwrap();
    let tx = app_state.channel.get_sender();

    app_state.channel.loading = true;
    thread::spawn(move || {
        let text = source.parse_chapter(novel_path, chapter_path);
        let _ = tx.send(RequestData::Chapter((BookInfo::ID(id), text, ch)));
    });

    Ok(())
}

/// Set up for and enter the typing screen
pub fn enter_typing(app_state: &mut AppState) {
    app_state.buffer.text.clear();
    app_state.typing = true;
}

/// Exit the typing screen
pub fn exit_typing(app_state: &mut AppState) {
    app_state.typing = false;
}

pub enum BookViewType {
    Source,
    Lib,
    History,
}

/// Enter a book view, given a book
pub fn enter_book_view(app_state: &mut AppState, ctx: &Context, book: BookRef, view: BookViewType) {
    app_state.buffer.chapter_previews = StatefulList::from(book.get_chapters().unwrap());
    app_state.buffer.novel = Some(book);
    match view {
        BookViewType::Source => {
            app_state.buffer.book_view_option = BookViewOption::SourceOptions;
            app_state.source_data.novel_preview_selected_field =
                SourceNovelPreviewSelection::Options;
            app_state.source_data.novel_options.select_first();

            app_state.source_data.reset_novel_options();

            // Swap to removing
            if ctx
                .get_book_url(
                    app_state
                        .buffer
                        .novel
                        .as_ref()
                        .expect("we just added the book to the buffer")
                        .get_full_url()
                        .unwrap()
                        .to_string(),
                )
                .is_some_and(|x| x.in_library())
            {
                app_state.source_data.swap_library_options()
            }

            app_state.update_screen(Screen::Sources(SourceScreen::BookView))
        }
        BookViewType::Lib => {
            app_state.buffer.book_view_option = BookViewOption::LibOptions;
            app_state.source_data.novel_preview_selected_field =
                SourceNovelPreviewSelection::Options;
            app_state.lib_data.global_selected_book_opts.select_first();
            app_state.update_screen(Screen::Lib(LibScreen::BookView))
        }
        BookViewType::History => {
            app_state.buffer.book_view_option = BookViewOption::HistoryOptions;
            app_state.source_data.novel_preview_selected_field =
                SourceNovelPreviewSelection::Options;
            // Get rid of any dynamically added options
            app_state.history_data.reset_options();

            // If the book is in the library, have an option to remove it
            // otherwise have an option to add it
            if ctx
                .get_book(
                    app_state
                        .buffer
                        .novel
                        .as_ref()
                        .expect("we just added the book to the buffer")
                        .get_id(),
                )
                .is_some_and(|b| b.in_library())
            {
                app_state
                    .history_data
                    .global_book_options
                    .push(String::from("Remove from library"));
            } else {
                app_state
                    .history_data
                    .global_book_options
                    .push(String::from("Add to library"));
            }
            app_state.history_data.global_book_options.select_first();
            app_state.update_screen(Screen::History(HistoryScreen::BookView))
        }
    }
}

pub fn search_book_details(
    app_state: &mut AppState,
    ctx: &Context,
    source_id: SourceID,
    novel: &NovelPreview,
    view_type: BookInfoDetails,
) -> Result<(), SourceError> {
    let url = novel.get_url().to_string();
    let Some(source) = ctx.get_source_by_id(source_id) else {
        return Err(SourceError::NonExistent);
    };
    let source = source.clone();
    let tx = app_state.channel.get_sender();
    app_state.channel.loading = true;

    thread::spawn(move || {
        let book = source.parse_novel_and_chapters(url);
        let _ = tx.send(RequestData::BookInfo((book, view_type)));
    });

    Ok(())
}

pub fn enter_category_select(app_state: &mut AppState, ctx: &Context) {
    let cats = ctx.get_library_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
    app_state.update_screen(Screen::Lib(LibScreen::CategorySelect));
}

pub fn enter_book_opts_categories(app_state: &mut AppState, ctx: &Context) {
    let cats = ctx.get_library_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
    app_state.update_screen(Screen::Lib(LibScreen::BookViewCategory));
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

    ctx.move_book_category(b, Some(&ctx.get_library_categories()[cat_idx].clone()));

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
    if ctx.delete_library_category(category_name).is_ok() {
        // If we deleted the last category, and it's currently selected,
        // We subtract one
        if ctx.get_library_categories().len() == app_state.lib_data.get_selected_category() {
            app_state.lib_data.current_category_idx =
                app_state.lib_data.current_category_idx.saturating_sub(1);
        }

        let cats = ctx.get_library_categories().clone();
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
    if ctx.create_library_category(category_name).is_ok() {
        let cats = ctx.get_library_categories().clone();
        app_state.buffer.temporary_list = StatefulList::from(cats);
        Ok(())
    } else {
        Err(())
    }
}

pub fn move_category_up(app_state: &mut AppState, ctx: &mut Context) {
    let cat = app_state
        .buffer
        .temporary_list
        .selected_idx()
        .expect("a category should always be selected");

    let Some(new_pos) = ctx.reorder_category_forwards(cat) else {
        return;
    };

    // Fix the list to reflect the new state
    let cats = ctx.get_library_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
    app_state
        .buffer
        .temporary_list
        .state_mut()
        .select(Some(new_pos))
}

pub fn move_category_down(app_state: &mut AppState, ctx: &mut Context) {
    let cat = app_state
        .buffer
        .temporary_list
        .selected_idx()
        .expect("a category should always be selected");

    let Some(new_pos) = ctx.reorder_category_backwards(cat) else {
        return;
    };

    // Fix the list to reflect the new state
    let cats = ctx.get_library_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
    app_state
        .buffer
        .temporary_list
        .state_mut()
        .select(Some(new_pos))
}

pub fn rename_category(app_state: &mut AppState, ctx: &mut Context, new_name: String) {
    let old_name = app_state
        .buffer
        .temporary_list
        .selected()
        .expect("a category should always be selected");
    ctx.rename_library_category(old_name.to_string(), new_name);

    let cats = ctx.get_library_categories().clone();
    app_state.buffer.temporary_list = StatefulList::from(cats);
}

/// Starts a book from the beginning
pub fn start_book_from_beginning(
    app_state: &mut AppState,
    ctx: &Context,
    book: BookRef,
) -> Result<(), BookError> {
    if book.is_local() {
        unimplemented!()
    }

    let (b_info, source) = {
        if book.get_total_ch_count().expect("invariants were checked") == 0 {
            return Err(BookError::NoChapters);
        }
        let source = ctx
            .get_book_source(book.get_id())
            .expect("invariants were checked")
            .clone();
        (BookInfo::ID(book.get_id()), source)
    };

    let novel_path = book.get_url().unwrap().to_string();
    let chapter_path = book.get_chapter_url(1).unwrap().to_string();

    let tx = app_state.channel.get_sender();

    app_state.channel.loading = true;
    thread::spawn(move || {
        // We checked there's at least 1 chapter
        #[allow(clippy::unwrap_used)]
        let text = source.parse_chapter(novel_path, chapter_path);
        let _ = tx.send(RequestData::Chapter((b_info, text, 1)));
    });

    Ok(())
}

/// Starts a book from a given chapter
pub fn start_book_from_ch(
    app_state: &mut AppState,
    ctx: &Context,
    book: BookRef,
    chapter: usize,
) -> Result<(), BookError> {
    if book.is_local() {
        unimplemented!()
    }

    let (b_info, source) = {
        if book.get_total_ch_count().expect("invariants were checked") < chapter {
            return Err(BookError::NoChapters);
        }
        let source = ctx
            .get_book_source(book.get_id())
            .expect("invariants were checked")
            .clone();

        let info = if ctx.book_exists(book.get_id()) {
            BookInfo::ID(book.get_id())
        } else {
            unreachable!()
            // BookInfo::NewBook(book.clone())
        };
        (info, source)
    };

    let tx = app_state.channel.get_sender();

    let novel_path = book.get_url().unwrap().to_string();
    let chapter_path = book.get_chapter_url(chapter).unwrap().to_string();

    app_state.channel.loading = true;
    thread::spawn(move || {
        let text = source.parse_chapter(novel_path, chapter_path);
        let _ = tx.send(RequestData::Chapter((b_info, text, chapter)));
    });

    Ok(())
}

pub fn continue_book_history(app_state: &mut AppState, ctx: &Context, entry: &HistoryEntry) {
    let ch = entry.get_chapter();
    let book = entry.get_book_ref();
    let id = entry.get_book_id();

    let source = ctx.get_book_source(id).unwrap().clone();
    let tx = app_state.channel.get_sender();

    let novel_path = book.get_url().unwrap();
    let chapter_path = book.get_chapter_url(ch).unwrap();

    app_state.channel.loading = true;
    thread::spawn(move || {
        let text = source.parse_chapter(novel_path, chapter_path);
        let _ = tx.send(RequestData::Chapter((BookInfo::ID(id), text, ch)));
    });
}

pub fn goto_next_ch(app_state: &mut AppState, ctx: &mut Context) -> Result<(), BookError> {
    app_state.update_from_reader(ctx);

    let Some(book) = app_state.reader_data.get_book() else {
        return Err(BookError::NonExistent);
    };

    let id = book.get_id();

    let ch = if book.get_current_ch().unwrap() + 1 <= book.get_total_ch_count().unwrap() {
        book.get_current_ch().unwrap() + 1
    } else {
        return Err(BookError::UnavailableChapter);
    };

    let source = ctx.get_book_source(id).unwrap().clone();

    let tx = app_state.channel.get_sender();

    app_state.channel.loading = true;
    let novel_path = book.get_url().unwrap().to_string();
    let chapter_path = book.get_chapter_url(ch).unwrap().to_string();

    thread::spawn(move || {
        let text = source.parse_chapter(novel_path, chapter_path);
        let _ = tx.send(RequestData::Chapter((BookInfo::ID(id), text, ch)));
    });

    Ok(())
}

pub fn goto_prev_ch(app_state: &mut AppState, ctx: &mut Context) -> Result<(), BookError> {
    app_state.update_from_reader(ctx);

    let Some(book) = app_state.reader_data.get_book() else {
        return Err(BookError::NonExistent);
    };

    let id = book.get_id();

    let ch = if book.get_current_ch().unwrap() != 1 {
        book.get_current_ch().unwrap() - 1
    } else {
        return Err(BookError::UnavailableChapter);
    };

    let source = ctx.get_book_source(id).unwrap().clone();

    let novel_path = book.get_url().unwrap();
    let chapter_path = book.get_chapter_url(ch).unwrap();

    let tx = app_state.channel.get_sender();
    app_state.channel.loading = true;

    thread::spawn(move || {
        let text = source.parse_chapter(novel_path, chapter_path);
        let _ = tx.send(RequestData::Chapter((BookInfo::ID(id), text, ch)));
    });

    Ok(())
}

pub fn remove_history_entry(app_state: &mut AppState, ctx: &mut Context, book: ID) {
    // Remove book
    ctx.remove_history_entry(book);

    // Select the previous entry if one exists, otherwise select none
    let sel = app_state.history_data.get_selected_entry_mut().selected();
    match sel {
        Some(n) => {
            if n == 0 {
                if ctx.get_history_entry_count() == 0 {
                    app_state.history_data.get_selected_entry_mut().select(None);
                } else {
                    app_state
                        .history_data
                        .get_selected_entry_mut()
                        .select(Some(0));
                }
            } else {
                app_state
                    .history_data
                    .get_selected_entry_mut()
                    .select(Some(n - 1));
            }
        }
        // If nothing's selected then we leave it
        None => return,
    }
}

/// Search a source for a term. If no term is given, search for popular books
pub fn search_source(
    app_state: &mut AppState,
    ctx: &Context,
    source_id: SourceID,
    search_term: Option<String>,
) -> Result<(), SourceError> {
    let Some(source) = ctx.get_source_by_id(source_id) else {
        return Err(SourceError::NonExistent);
    };
    let source = source.clone();
    let tx = app_state.channel.get_sender();
    app_state.channel.loading = true;

    if let Some(term) = search_term {
        thread::spawn(move || {
            let res = source.search_novels(&term);
            let _ = tx.send(RequestData::SearchResults(res));
        });
    } else {
        thread::spawn(move || {
            let res = source.get_popular(SortOrder::Rating, 1);
            let _ = tx.send(RequestData::SearchResults(res));
        });
    }

    Ok(())
}

/// Rename a book, setting it's name back to the default if a new name isn't given. Returns the resulting name.
pub fn rename_book(book: &mut BookRef, new_name: Option<String>) -> String {
    if let Some(n) = new_name {
        book.rename(n.clone());
        n
    } else {
        if book.is_local() {
            todo!()
        } else {
            let n = book
                .global_get_original_name()
                .expect("invariants were checked");
            book.rename(n.clone());
            n
        }
    }
}

pub fn add_book_to_lib(app_state: &mut AppState, ctx: &mut Context, book: BookRef) {
    ctx.add_to_lib(book.get_id(), None);
    // Select this book if there were previously no books selected
    app_state.lib_data.fix_book_selection_state(ctx);
}
