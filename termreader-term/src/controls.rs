use crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use std::thread;
use termreader_core::book::{Book, ChapterProgress};
use termreader_core::Context;
use termreader_sources::sources::{Scrape, SortOrder};

use crate::enter::{
    self, continue_reading_global_select, create_category, delete_category, enter_category_options,
    enter_category_select, enter_global_book_select, enter_history_book_view, enter_lib_book_view,
    enter_typing, move_book_category, rename_category,
};
use crate::helpers::StatefulList;
use crate::state::channels::BookInfoDetails;
use crate::{
    state::{
        channels::RequestData, sources::SourceNovelPreviewSelection, AppState, HistoryScreen,
        LibScreen, Screen, SettingsScreen, SourceScreen, UpdateScreen,
    },
    trace_dbg,
};

pub fn handle_controls(ctx: &mut Context, app_state: &mut AppState, mut key: KeyCode) {
    // Handle typing seperately to other controls
    if app_state.typing {
        handle_typing(ctx, app_state, key);
        return;
    }

    // Simple aliasing
    key = match key {
        // Vim bindings
        KeyCode::Char('h') => KeyCode::Left,
        KeyCode::Char('j') => KeyCode::Down,
        KeyCode::Char('k') => KeyCode::Up,
        KeyCode::Char('l') => KeyCode::Right,
        x => x,
    };

    if matches!(key, KeyCode::Esc) || matches!(key, KeyCode::Char('q')) {
        control_back(app_state, ctx)
    }

    match app_state.screen {
        Screen::Reader => control_reader(ctx, app_state, key),
        Screen::Lib(s) => match s {
            LibScreen::Main => {
                control_main_menu(app_state, key);
                control_library_menu(ctx, app_state, key);
            }
            LibScreen::GlobalBookSelect => {
                control_library_global_book_select(ctx, app_state, key);
            }
            LibScreen::BookView => control_book_view_no_opts(ctx, app_state, key),
            LibScreen::CategorySelect => control_library_category_select(ctx, app_state, key),
            LibScreen::CategoryOptions => control_library_category_options(ctx, app_state, key),
        },
        Screen::Updates(s) => match s {
            UpdateScreen::Main => control_main_menu(app_state, key),
        },
        Screen::Sources(s) => match s {
            SourceScreen::Main => {
                control_main_menu(app_state, key);
                control_source_menu(ctx, app_state, key);
            }
            SourceScreen::SearchRes => {
                control_search_res(ctx, app_state, key);
            }
            SourceScreen::Select => control_source_select(ctx, app_state, key),
            SourceScreen::BookView => control_source_book_view(ctx, app_state, key),
        },
        Screen::History(s) => match s {
            HistoryScreen::Main => {
                control_main_menu(app_state, key);
                control_history_menu(ctx, app_state, key);
            }
            HistoryScreen::LocalBookOptions => todo!(),
            HistoryScreen::GlobalBookOptions => control_history_global_book(ctx, app_state, key),
            HistoryScreen::BookView => control_book_view_no_opts(ctx, app_state, key),
        },
        Screen::Settings(s) => match s {
            SettingsScreen::Main => control_main_menu(app_state, key),
        },
    }
}

fn control_library_category_select(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.buffer.temporary_list.previous(),
        KeyCode::Down => app_state.buffer.temporary_list.next(),
        KeyCode::Enter => {
            if app_state
                .prev_screens
                .last()
                .expect("it should be impossible to start directly into this screen")
                == &Screen::Lib(LibScreen::GlobalBookSelect)
            {
                // We're moving a book to a different category
                move_book_category(app_state, ctx)
                    .expect("a book or category was not selected when it should have been")
            } else {
                // We're managing categories
                match app_state
                    .lib_data
                    .category_options
                    .selected_idx()
                    .expect("an option should always be seleceted")
                {
                    0 | 1 => unreachable!(),
                    // Rename
                    2 => enter_typing(app_state),
                    // Delete
                    3 => {
                        let deleted = delete_category(
                            app_state,
                            ctx,
                            app_state
                                .buffer
                                .temporary_list
                                .selected()
                                .expect("a category should always be selected")
                                .to_string(),
                        );
                        // Do nothing if the category wasn't deleted, but if it was, go back
                        if deleted.is_ok() {
                            app_state.update_screen(Screen::Lib(LibScreen::Main))
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        _ => (),
    }
}

fn control_library_category_options(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.lib_data.category_options.previous(),
        KeyCode::Down => app_state.lib_data.category_options.next(),
        KeyCode::Enter => match app_state
            .lib_data
            .category_options
            .selected_idx()
            .expect("an option should always be selected")
        {
            // Create category
            0 => enter_typing(app_state),
            // Re-order categories
            1 => (),
            // Rename categories / Delete categories
            2 | 3 => enter_category_select(app_state, ctx),
            _ => unreachable!(),
        },
        KeyCode::Char('c') => app_state.update_screen(Screen::Lib(LibScreen::Main)),
        _ => (),
    }
}

fn control_main_menu(app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char(']') | KeyCode::Tab => app_state.menu_tabs.next(),
        KeyCode::Char('[') | KeyCode::BackTab => app_state.menu_tabs.previous(),
        _ => (),
    };
    match app_state.menu_tabs.selected().unwrap().as_str() {
        "Library" => app_state.screen = Screen::Lib(LibScreen::Main),
        "Updates" => app_state.screen = Screen::Updates(UpdateScreen::Main),
        "Sources" => app_state.screen = Screen::Sources(SourceScreen::Main),
        "History" => app_state.screen = Screen::History(HistoryScreen::Main),
        "Settings" => app_state.screen = Screen::Settings(SettingsScreen::Main),
        _ => unreachable!(),
    };
}

fn control_library_menu(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('}') | KeyCode::Right => app_state.lib_data.select_next_category(ctx),
        KeyCode::Char('{') | KeyCode::Left => app_state.lib_data.select_previous_category(ctx),
        KeyCode::Up => app_state.lib_data.select_prev_book(ctx),
        KeyCode::Down => app_state.lib_data.select_next_book(ctx),
        KeyCode::Enter => {
            let _ = enter_global_book_select(app_state);
        }
        KeyCode::Char('c') => enter_category_options(app_state),
        _ => (),
    }
}

fn control_library_global_book_select(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.lib_data.global_selected_book_opts.previous(),
        KeyCode::Down => app_state.lib_data.global_selected_book_opts.next(),
        KeyCode::Enter => {
            match app_state
                .lib_data
                .global_selected_book_opts
                .selected_idx()
                .unwrap()
            {
                // 0 => Continue reading
                // 1 => View ch list
                // 2 => Move to category
                // 3 => Rename
                // 4 => Restart
                // 5 => Remove from lib
                0 => {
                    continue_reading_global_select(app_state, ctx).expect("a book has not been selected, even though this menu is only accessible on a selected book");
                }
                1 => {
                    let book = app_state.lib_data
                        .get_selected_book_mut(ctx)
                        .expect("a book has not been selected, even though this menu is only accessible on a selected book");
                    enter_lib_book_view(app_state, book.global_get_novel().clone())
                }
                2 => enter_category_select(app_state, ctx),
                3 => enter_typing(app_state),
                4 => {
                    let book = app_state.lib_data
                        .get_selected_book_mut(ctx)
                        .expect("a book has not been selected, even though this menu is only accessible on a selected book");
                    book.reset_progress();
                }
                5 => {
                    let book_id = app_state.lib_data
                        .get_selected_book_mut(ctx)
                        .expect("a book has not been selected, even though this menu is only accessible on a selected book")
                        .get_id();
                    ctx.lib_remove_book(book_id);
                    app_state.lib_data.reset_selection(ctx);
                    app_state.update_screen(Screen::Lib(LibScreen::Main))
                }
                _ => unreachable!(),
            }
        }
        _ => (),
    }
}

fn control_source_menu(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.source_data.select_prev(ctx),
        KeyCode::Down => app_state.source_data.select_next(ctx),
        KeyCode::Enter => {
            app_state.update_screen(Screen::Sources(SourceScreen::Select));
        }
        _ => (),
    }
}

fn control_source_select(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.source_data.source_options.previous(),
        KeyCode::Down => app_state.source_data.source_options.next(),
        KeyCode::Enter => {
            // 0: Search
            // 1: View popular
            match app_state.source_data.source_options.selected_idx().unwrap() {
                0 => {
                    app_state.buffer.text.clear();
                    app_state.typing = true;
                }
                1 => {
                    app_state.channel.loading = true;
                    let id = app_state.source_data.get_selected_source_id(ctx);
                    let source = ctx.source_get_by_id(id).cloned().unwrap();
                    let tx = app_state.channel.get_sender();
                    thread::spawn(move || {
                        let res = source.get_popular(SortOrder::Rating, 1);
                        let _ = tx.send(RequestData::SearchResults(res));
                    });
                }
                _ => unreachable!(),
            }
        }
        _ => (),
    }
}

fn control_search_res(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.buffer.novel_search_res.previous(),
        KeyCode::Down => app_state.buffer.novel_search_res.next(),
        KeyCode::Enter => {
            if let Some(preview) = app_state.buffer.novel_search_res.selected() {
                let url = preview.get_url().to_string();
                let source_id = app_state.source_data.get_selected_source_id(ctx);
                let source = ctx.source_get_by_id(source_id).cloned().unwrap();
                let tx = app_state.channel.get_sender();

                app_state.channel.loading = true;
                thread::spawn(move || {
                    let book = source.parse_novel_and_chapters(url);
                    tx.send(RequestData::BookInfo((
                        book,
                        BookInfoDetails::SourceWithOptions,
                    )))
                    .unwrap();
                });
            }
        }
        _ => (),
    }
}

fn handle_typing(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Backspace => {
            app_state.buffer.text.pop();
        }
        KeyCode::Char(c) => {
            app_state.buffer.text.push(c);
        }
        KeyCode::Esc => {
            app_state.buffer.text = String::new();
            app_state.typing = false;
        }
        KeyCode::Enter => {
            match app_state.screen {
                Screen::Sources(SourceScreen::Select) => {
                    // Search a source
                    app_state.channel.loading = true;
                    let id = app_state.source_data.get_selected_source_id(ctx);
                    let source = ctx.source_get_by_id(id).cloned().unwrap();
                    let text = app_state.buffer.text.clone();
                    let tx = app_state.channel.get_sender();

                    thread::spawn(move || {
                        let res = source.search_novels(&text);
                        tx.send(RequestData::SearchResults(res)).unwrap()
                    });
                }
                Screen::Lib(LibScreen::GlobalBookSelect) => {
                    // Rename a book
                    let book = app_state.lib_data
                        .get_selected_book_mut(ctx)
                        .expect("a book has not been selected, even though this menu is only accessible on a selected book");
                    if !app_state.buffer.text.is_empty() {
                        book.rename(app_state.buffer.text.to_string());
                    } else {
                        // Set back to the name from the source
                        book.rename(book.global_get_novel().get_name().to_string());
                    }
                    app_state.buffer.text.clear()
                }
                // Creating a category
                Screen::Lib(LibScreen::CategoryOptions) => {
                    let created =
                        create_category(app_state, ctx, app_state.buffer.text.to_string());
                    // Do nothing if we can't create the category
                    if created.is_ok() {
                        app_state.update_screen(Screen::Lib(LibScreen::Main))
                    }
                }
                // Renaming a category
                Screen::Lib(LibScreen::CategorySelect) => {
                    rename_category(app_state, ctx, app_state.buffer.text.to_string());
                    app_state.update_screen(Screen::Lib(LibScreen::Main))
                }
                _ => unreachable!(),
            }
            app_state.typing = false;
        }
        _ => (),
    }
}

fn control_source_book_view(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char(']') | KeyCode::Tab => {
            match app_state.source_data.novel_preview_selected_field {
                SourceNovelPreviewSelection::Options => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Chapters
                }
                SourceNovelPreviewSelection::Chapters => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Summary
                }
                SourceNovelPreviewSelection::Summary => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Options
                }
            }
        }
        KeyCode::Char('[') | KeyCode::BackTab => {
            match app_state.source_data.novel_preview_selected_field {
                SourceNovelPreviewSelection::Options => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Summary
                }
                SourceNovelPreviewSelection::Chapters => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Options
                }
                SourceNovelPreviewSelection::Summary => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Chapters
                }
            }
        }
        KeyCode::Up => match app_state.source_data.novel_preview_selected_field {
            SourceNovelPreviewSelection::Options => app_state.source_data.novel_options.previous(),
            SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.previous(),
            SourceNovelPreviewSelection::Summary => {
                if app_state.buffer.novel_preview_scroll != 0 {
                    app_state.buffer.novel_preview_scroll -= 1;
                }
            }
        },
        KeyCode::Down => match app_state.source_data.novel_preview_selected_field {
            SourceNovelPreviewSelection::Options => app_state.source_data.novel_options.next(),
            SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.next(),
            SourceNovelPreviewSelection::Summary => {
                app_state.buffer.novel_preview_scroll += 1;
            }
        },
        KeyCode::Enter => {
            match app_state.source_data.novel_preview_selected_field {
                SourceNovelPreviewSelection::Summary => (),
                SourceNovelPreviewSelection::Options => {
                    match app_state.source_data.novel_options.selected_idx().unwrap() {
                        // Start from beginning
                        0 => {
                            let novel = app_state.buffer.novel.clone().unwrap();
                            let nov2 = novel.clone();

                            let book = if let Some(book) =
                                ctx.find_book_by_url(novel.get_full_url().to_string())
                            {
                                book.clone()
                            } else {
                                Book::from_novel(novel)
                            };

                            let id = book.get_id();
                            // TODO: Add some display symbolizing there are no chapters maybe
                            if book.global_get_total_chs() == 0 {
                                return;
                            }

                            let source_id = book.global_get_source_id();
                            let source = ctx.source_get_by_id(source_id).unwrap().clone();
                            app_state.buffer.book = Some(book);

                            let tx = app_state.channel.get_sender();
                            app_state.channel.loading = true;
                            thread::spawn(move || {
                                let text = source.parse_chapter(
                                    nov2.get_url().to_string(),
                                    nov2.get_chapter_url(1).unwrap(),
                                );
                                let _ = tx.send(RequestData::Chapter((id, text, 1)));
                            });
                        }
                        // Add to lib
                        1 => {
                            let novel = app_state.buffer.novel.clone().unwrap();

                            let book = if let Some(book) =
                                ctx.find_book_by_url(novel.get_full_url().to_string())
                            {
                                book.clone()
                            } else {
                                Book::from_novel(novel)
                            };

                            ctx.lib_add_book(book, None);
                            // If there were previously no books selected then select this one
                            app_state.lib_data.fix_book_selection_state(ctx);
                            app_state.screen = app_state.prev_screens.pop().unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                SourceNovelPreviewSelection::Chapters => {
                    let ch = app_state
                        .buffer
                        .chapter_previews
                        .selected()
                        .unwrap()
                        .clone();
                    let ch_no = ch.get_chapter_no();
                    let novel = app_state.buffer.novel.clone().unwrap();
                    let nov2 = novel.clone();

                    let book = if let Some(book) =
                        ctx.find_book_by_url(novel.get_full_url().to_string())
                    {
                        book.clone()
                    } else {
                        Book::from_novel(novel)
                    };

                    let id = book.get_id();

                    let source_id = book.global_get_source_id();
                    let source = ctx.source_get_by_id(source_id).unwrap().clone();
                    app_state.buffer.book = Some(book);

                    let tx = app_state.channel.get_sender();
                    app_state.channel.loading = true;
                    thread::spawn(move || {
                        let text = source.parse_chapter(
                            nov2.get_url().to_string(),
                            nov2.get_chapter_url(ch_no).unwrap(),
                        );
                        let _ = tx.send(RequestData::Chapter((id, text, ch_no)));
                    });
                }
            }
        }
        _ => (),
    }
}

fn control_book_view_no_opts(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char(']') | KeyCode::Tab => {
            match app_state.source_data.novel_preview_selected_field {
                SourceNovelPreviewSelection::Chapters => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Summary
                }
                SourceNovelPreviewSelection::Summary => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Chapters
                }
                _ => unreachable!(),
            }
        }
        KeyCode::Char('[') | KeyCode::BackTab => {
            match app_state.source_data.novel_preview_selected_field {
                SourceNovelPreviewSelection::Chapters => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Summary
                }
                SourceNovelPreviewSelection::Summary => {
                    app_state.source_data.novel_preview_selected_field =
                        SourceNovelPreviewSelection::Chapters
                }
                _ => unreachable!(),
            }
        }
        KeyCode::Up => match app_state.source_data.novel_preview_selected_field {
            SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.previous(),
            SourceNovelPreviewSelection::Summary => {
                if app_state.buffer.novel_preview_scroll != 0 {
                    app_state.buffer.novel_preview_scroll -= 1;
                }
            }
            _ => unreachable!(),
        },
        KeyCode::Down => match app_state.source_data.novel_preview_selected_field {
            SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.next(),
            SourceNovelPreviewSelection::Summary => {
                app_state.buffer.novel_preview_scroll += 1;
            }
            _ => unreachable!(),
        },
        KeyCode::Enter => match app_state.source_data.novel_preview_selected_field {
            SourceNovelPreviewSelection::Chapters => {
                let ch = app_state
                    .buffer
                    .chapter_previews
                    .selected()
                    .unwrap()
                    .clone();
                let ch_no = ch.get_chapter_no();

                let novel = app_state.buffer.novel.clone().unwrap();
                let novel_thread = novel.clone();

                let book =
                    if let Some(book) = ctx.find_book_by_url(novel.get_full_url().to_string()) {
                        book.clone()
                    } else {
                        Book::from_novel(novel)
                    };

                let id = book.get_id();

                let source_id = book.global_get_source_id();
                let source = ctx.source_get_by_id(source_id).unwrap().clone();
                app_state.buffer.book = Some(book);

                let tx = app_state.channel.get_sender();
                app_state.channel.loading = true;
                thread::spawn(move || {
                    let text = source.parse_chapter(
                        novel_thread.get_url().to_string(),
                        novel_thread.get_chapter_url(ch_no).unwrap(),
                    );
                    let _ = tx.send(RequestData::Chapter((id, text, ch_no)));
                });
            }
            _ => (),
        },
        _ => (),
    }
}

fn control_back(app_state: &mut AppState, ctx: &mut Context) {
    if app_state.screen == Screen::Reader {
        app_state.update_from_reader(ctx);
        app_state.screen = Screen::Lib(LibScreen::Main);
        return;
    }

    if app_state.prev_screens.is_empty() {
        app_state.quit = true;
        return;
    }

    let prev = app_state.prev_screens.pop().unwrap();
    app_state.screen = prev;
}

fn control_reader(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.reader_data.scroll_up(),
        KeyCode::Down => app_state.reader_data.scroll_down(),
        KeyCode::Right => {
            // Go to the next chapter
            app_state.update_from_reader(ctx);

            let book = app_state.reader_data.get_book().as_ref().unwrap();
            let id = book.get_id();
            let novel = book.global_get_novel().clone();

            let ch = if book.global_get_current_ch() + 1 <= book.global_get_total_chs() {
                book.global_get_current_ch() + 1
            } else {
                return;
            };

            let source = ctx.source_get_by_id(novel.get_source()).unwrap().clone();
            let tx = app_state.channel.get_sender();

            app_state.channel.loading = true;
            thread::spawn(move || {
                let text = source.parse_chapter(
                    novel.get_url().to_string(),
                    novel.get_chapter_url(ch).unwrap(),
                );
                let _ = tx.send(RequestData::Chapter((id, text, ch)));
            });
        }
        KeyCode::Left => {
            // Go to the previous chapter
            app_state.update_from_reader(ctx);

            let book = app_state.reader_data.get_book().as_ref().unwrap();
            let id = book.get_id();
            let novel = book.global_get_novel().clone();
            // The previous chapter will always exist unless we're reading the first one
            let ch = if book.global_get_current_ch() != 1 {
                book.global_get_current_ch() - 1
            } else {
                return;
            };

            let source = ctx.source_get_by_id(novel.get_source()).unwrap().clone();
            let tx = app_state.channel.get_sender();

            app_state.channel.loading = true;
            thread::spawn(move || {
                let text = source.parse_chapter(
                    novel.get_url().to_string(),
                    novel.get_chapter_url(ch).unwrap(),
                );
                let _ = tx.send(RequestData::Chapter((id, text, ch)));
            });
        }
        _ => (),
    }
}

fn control_history_menu(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.history_data.select_prev_entry(ctx),
        KeyCode::Down => app_state.history_data.select_next_entry(ctx),
        KeyCode::Enter => {
            let b = app_state.history_data.get_selected_book(ctx);
            let Some(entry) = b else {
                return;
            };
            if entry.get_book().is_local() {
                app_state.update_screen(Screen::History(HistoryScreen::LocalBookOptions));
            } else {
                app_state.update_screen(Screen::History(HistoryScreen::GlobalBookOptions));
            }
        }
        _ => (),
    }
}

fn control_history_global_book(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => app_state.history_data.global_book_options.previous(),
        KeyCode::Down => app_state.history_data.global_book_options.next(),
        KeyCode::Enter => {
            let opt = app_state
                .history_data
                .global_book_options
                .selected_idx()
                .unwrap();

            let b = app_state
                .history_data
                .get_selected_book(ctx)
                .expect("A book was selected without being selected?");

            let book = b.get_book();

            match opt {
                0 => {
                    // Continue reading
                    let ch = b.get_chapter();
                    let novel = book.global_get_novel().clone();
                    let id = book.get_id();
                    let source = ctx.source_get_by_id(novel.get_source()).unwrap().clone();
                    let tx = app_state.channel.get_sender();
                    app_state.channel.loading = true;
                    thread::spawn(move || {
                        let text = source.parse_chapter(
                            novel.get_url().to_string(),
                            novel.get_chapter_url(ch).unwrap(),
                        );
                        let _ = tx.send(RequestData::Chapter((id, text, ch)));
                    });
                }
                1 => {
                    // View book
                    let lib_copy = ctx.lib_find_book(book.get_id());
                    match lib_copy {
                        Some(lib_book) => {
                            enter_history_book_view(app_state, lib_book.global_get_novel().clone());
                        }
                        None => {
                            let novel = book.global_get_novel().clone();
                            let source = ctx.source_get_by_id(novel.get_source()).unwrap().clone();
                            let tx = app_state.channel.get_sender();
                            app_state.channel.loading = true;
                            thread::spawn(move || {
                                let info =
                                    source.parse_novel_and_chapters(novel.get_url().to_string());
                                let _ = tx.send(RequestData::BookInfo((
                                    info,
                                    BookInfoDetails::HistoryWithOptions,
                                )));
                            });
                        }
                    }
                    // app_state.channel.loading = true;
                    // thread::spawn(move || {
                    //     let novel_info =
                    //         source.parse_novel_and_chapters(novel.get_url().to_string());
                    //     let _ = tx.send(RequestData::BookInfo((novel_info, false)));
                    // });
                }
                2 => {
                    // Remove from history
                    ctx.hist_remove_entry(book.get_id());

                    // Select the previous entry if one exists
                    let sel = app_state
                        .history_data
                        .get_selected_entry_mut()
                        .selected()
                        .expect("we just removed an entry from history so it must have an index");
                    if sel == 0 {
                        if ctx.hist_get_len() == 0 {
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
                            .select(Some(sel - 1));
                    }
                    app_state.update_screen(Screen::History(HistoryScreen::Main));
                }
                _ => unreachable!(),
            }
        }
        _ => (),
    }
}
