use crossterm::event::KeyCode;
use termreader_core::Context;

use crate::setup::{
    add_book_to_lib, continue_book_history, continue_reading_global_select, create_category,
    delete_category, enter_book_opts_categories, enter_book_view, enter_category_options,
    enter_category_select, enter_typing, exit_typing, goto_next_ch, goto_prev_ch,
    move_book_category, move_category_down, move_category_up, remove_history_entry, rename_book,
    rename_category, search_book_details, search_source, start_book_from_beginning,
    start_book_from_ch, BookViewType,
};
use crate::state::config::ConfigData;
use crate::state::{
    channels::BookInfoDetails, sources::SourceNovelPreviewSelection, AppState, HistoryScreen,
    LibScreen, Screen, SettingsScreen, SourceScreen, UpdateScreen,
};
use crate::ui::sources::BookViewOption;

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
            LibScreen::BookView | LibScreen::BookViewCategory => {
                control_book_view_opts(ctx, app_state, key)
            }
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
            SourceScreen::BookView => control_book_view_opts(ctx, app_state, key),
        },
        Screen::History(s) => match s {
            HistoryScreen::Main => {
                control_main_menu(app_state, key);
                control_history_menu(ctx, app_state, key);
            }
            HistoryScreen::LocalBookOptions => todo!(),
            HistoryScreen::GlobalBookOptions => control_history_global_book(ctx, app_state, key),
            HistoryScreen::BookView => {
                if app_state.history_data.view_book_with_opts {
                    control_book_view_opts(ctx, app_state, key)
                } else {
                    // control_book_view_no_opts(ctx, app_state, key)
                    todo!()
                }
            }
        },
        Screen::Settings(s) => match s {
            SettingsScreen::Main => control_main_menu(app_state, key),
        },
    }
}

fn control_library_category_select(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up => {
            if app_state.buffer.reorder_lock {
                move_category_up(app_state, ctx);
                return;
            }
            app_state.buffer.temporary_list.previous();
        }
        KeyCode::Down => {
            if app_state.buffer.reorder_lock {
                move_category_down(app_state, ctx);
                return;
            }
            app_state.buffer.temporary_list.next();
        }
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
                    0 => unreachable!(),
                    1 => {
                        // TODO: Use custom styles here
                        app_state.buffer.reorder_lock = !app_state.buffer.reorder_lock;
                        if app_state.config.selected_style == ConfigData::DEFAULT_SELECTED_STYLE {
                            app_state.config.selected_style = ConfigData::DEFAULT_SELECTED_STYLE_2;
                        } else {
                            app_state.config.selected_style = ConfigData::DEFAULT_SELECTED_STYLE;
                        }
                    }
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
        KeyCode::Up => {
            app_state.lib_data.category_options.previous();
        }
        KeyCode::Down => {
            app_state.lib_data.category_options.next();
        }
        KeyCode::Enter => match app_state
            .lib_data
            .category_options
            .selected_idx()
            .expect("an option should always be selected")
        {
            // Create category
            0 => enter_typing(app_state),
            // Part 2 of re-ordering
            // 1 if app_state.buffer.reorder_lock => {
            //     // TODO: Use custom styles here
            //     app_state.config.selected_style = ConfigData::DEFAULT_SELECTED_STYLE;
            //     app_state.buffer.reorder_lock = false;
            //     app_state.update_screen(Screen::Lib(LibScreen::CategorySelect))
            // }
            // Re-order categories / Rename categories / Delete categories
            1 | 2 | 3 => enter_category_select(app_state, ctx),
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
            // let _ = enter_global_book_select(app_state);
            let b = app_state.lib_data.get_selected_book(ctx);
            // If there's no book selected do nothing
            let Some(book) = b else {
                return;
            };

            if book.is_global() {
                let novel = book.global_get_novel();
                enter_book_view(app_state, novel.clone(), BookViewType::Lib);
            } else {
                unimplemented!()
            }
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
                    enter_book_view(
                        app_state,
                        book.global_get_novel().clone(),
                        BookViewType::Lib,
                    )
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
                    app_state.lib_data.global_selected_book_opts.select_first();
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
                0 => enter_typing(app_state),
                1 => {
                    search_source(
                        app_state,
                        ctx,
                        app_state.source_data.get_selected_source_id(ctx),
                        None,
                    )
                    .expect("this source should exist given it was selected from a menu");
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
            if let Some(preview) = app_state.buffer.novel_search_res.selected().cloned() {
                search_book_details(
                    app_state,
                    ctx,
                    app_state.source_data.get_selected_source_id(ctx),
                    &preview,
                    BookInfoDetails::SourceWithOptions,
                )
                .expect("source should exist");
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
        KeyCode::Esc => exit_typing(app_state),
        KeyCode::Enter => {
            match app_state.screen {
                Screen::Sources(SourceScreen::Select) => {
                    let id = app_state.source_data.get_selected_source_id(ctx);
                    search_source(app_state, ctx, id, Some(app_state.buffer.text.clone()))
                        .expect("source should exist");
                }
                Screen::Lib(LibScreen::BookView) => {
                    // Rename a book
                    let book = app_state.lib_data
                        .get_selected_book_mut(ctx)
                        .expect("a book has not been selected, even though this menu is only accessible on a selected book");
                    let new_name = if app_state.buffer.text.is_empty() {
                        None
                    } else {
                        Some(app_state.buffer.text.clone())
                    };
                    let new_name = rename_book(book, new_name);
                    app_state
                        .buffer
                        .novel
                        .as_mut()
                        .expect("we're in book view so there must be a novel here")
                        .set_alias(new_name);
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

fn control_book_view_opts(ctx: &mut Context, app_state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char(']') | KeyCode::Tab => {
            if matches!(app_state.screen, Screen::Lib(LibScreen::BookViewCategory)) {
                return;
            }
            app_state
                .source_data
                .novel_preview_selected_field
                .next_opts();
        }
        KeyCode::Char('[') | KeyCode::BackTab => {
            if matches!(app_state.screen, Screen::Lib(LibScreen::BookViewCategory)) {
                return;
            }
            app_state
                .source_data
                .novel_preview_selected_field
                .prev_opts();
        }
        KeyCode::Up => match app_state.source_data.novel_preview_selected_field {
            SourceNovelPreviewSelection::Options => match app_state.buffer.book_view_option {
                BookViewOption::None => unreachable!(),
                BookViewOption::LibOptions => {
                    if matches!(app_state.screen, Screen::Lib(LibScreen::BookViewCategory)) {
                        app_state.buffer.temporary_list.previous()
                    } else {
                        app_state.lib_data.global_selected_book_opts.previous()
                    }
                }
                BookViewOption::SourceOptions => app_state.source_data.novel_options.previous(),
            },
            SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.previous(),
            SourceNovelPreviewSelection::Summary => {
                if app_state.buffer.novel_preview_scroll != 0 {
                    app_state.buffer.novel_preview_scroll -= 1;
                }
            }
        },
        KeyCode::Down => match app_state.source_data.novel_preview_selected_field {
            // SourceNovelPreviewSelection::Options => app_state.source_data.novel_options.next(),
            SourceNovelPreviewSelection::Options => match app_state.buffer.book_view_option {
                BookViewOption::None => unreachable!(),
                BookViewOption::LibOptions => {
                    if matches!(app_state.screen, Screen::Lib(LibScreen::BookViewCategory)) {
                        app_state.buffer.temporary_list.next()
                    } else {
                        app_state.lib_data.global_selected_book_opts.next()
                    }
                }
                BookViewOption::SourceOptions => app_state.source_data.novel_options.next(),
            },
            SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.next(),
            SourceNovelPreviewSelection::Summary => {
                app_state.buffer.novel_preview_scroll += 1;
            }
        },
        KeyCode::Enter => {
            match app_state.source_data.novel_preview_selected_field {
                SourceNovelPreviewSelection::Summary => (),
                SourceNovelPreviewSelection::Options => {
                    if matches!(app_state.screen, Screen::Lib(LibScreen::BookViewCategory)) {
                        move_book_category(app_state, ctx)
                            .expect("a book and category should always be selected here");
                        return;
                    }

                    match app_state.buffer.book_view_option {
                        // Never none if we're in this screen
                        BookViewOption::None => unreachable!(),
                        BookViewOption::LibOptions => {
                            match app_state
                                .lib_data
                                .global_selected_book_opts
                                .selected_idx()
                                .expect("an option should be selected")
                            {
                                // 0 => Continue reading
                                // 1 => Move to category
                                // 2 => Rename
                                // 3 => Restart
                                // 4 => Remove from lib
                                0 => {
                                    match continue_reading_global_select(app_state, ctx) {
                                        Ok(()) => (),
                                        // Failure is due to a book having no chapters.
                                        // As this is the case, we can fail silently
                                        Err(_) => (),
                                    }
                                }
                                1 => enter_book_opts_categories(app_state, ctx),
                                2 => enter_typing(app_state),
                                3 => {
                                    let book = app_state.lib_data
                                        .get_selected_book_mut(ctx)
                                        .expect("a book has not been selected, even though this menu is only accessible on a selected book");
                                    book.reset_progress();
                                }
                                4 => {
                                    let book_id = app_state.lib_data
                                        .get_selected_book_mut(ctx)
                                        .expect("a book has not been selected, even though this menu is only accessible on a selected book")
                                        .get_id();
                                    ctx.lib_remove_book(book_id);
                                    app_state.lib_data.reset_selection(ctx);
                                    app_state.lib_data.global_selected_book_opts.select_first();
                                    app_state.update_screen(Screen::Lib(LibScreen::Main))
                                }
                                _ => unreachable!(),
                            };
                        }

                        BookViewOption::SourceOptions => {
                            match app_state.source_data.novel_options.selected_idx().unwrap() {
                                // Start from beginning
                                0 => start_book_from_beginning(
                                    app_state,
                                    ctx,
                                    app_state.buffer.novel.clone().unwrap(),
                                )
                                .expect("the book in the buffer should be valid"),
                                // Add to lib
                                1 => {
                                    let novel = app_state.buffer.novel.clone().unwrap();
                                    add_book_to_lib(app_state, ctx, novel);
                                    app_state.screen = app_state.prev_screens.pop().unwrap();
                                }
                                _ => unreachable!(),
                            }
                        }
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

                    start_book_from_ch(
                        app_state,
                        ctx,
                        app_state.buffer.novel.clone().unwrap(),
                        ch_no,
                    )
                    .expect("there should always be enough chapters");
                }
            }
        }
        _ => (),
    }
}

// fn control_book_view_no_opts(ctx: &Context, app_state: &mut AppState, key: KeyCode) {
//     match key {
//         KeyCode::Char(']') | KeyCode::Tab => {
//             match app_state.source_data.novel_preview_selected_field {
//                 SourceNovelPreviewSelection::Chapters => {
//                     app_state.source_data.novel_preview_selected_field =
//                         SourceNovelPreviewSelection::Summary
//                 }
//                 SourceNovelPreviewSelection::Summary => {
//                     app_state.source_data.novel_preview_selected_field =
//                         SourceNovelPreviewSelection::Chapters
//                 }
//                 _ => unreachable!(),
//             }
//         }
//         KeyCode::Char('[') | KeyCode::BackTab => {
//             match app_state.source_data.novel_preview_selected_field {
//                 SourceNovelPreviewSelection::Chapters => {
//                     app_state.source_data.novel_preview_selected_field =
//                         SourceNovelPreviewSelection::Summary
//                 }
//                 SourceNovelPreviewSelection::Summary => {
//                     app_state.source_data.novel_preview_selected_field =
//                         SourceNovelPreviewSelection::Chapters
//                 }
//                 _ => unreachable!(),
//             }
//         }
//         KeyCode::Up => match app_state.source_data.novel_preview_selected_field {
//             SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.previous(),
//             SourceNovelPreviewSelection::Summary => {
//                 if app_state.buffer.novel_preview_scroll != 0 {
//                     app_state.buffer.novel_preview_scroll -= 1;
//                 }
//             }
//             _ => unreachable!(),
//         },
//         KeyCode::Down => match app_state.source_data.novel_preview_selected_field {
//             SourceNovelPreviewSelection::Chapters => app_state.buffer.chapter_previews.next(),
//             SourceNovelPreviewSelection::Summary => {
//                 app_state.buffer.novel_preview_scroll += 1;
//             }
//             _ => unreachable!(),
//         },
//         KeyCode::Enter => match app_state.source_data.novel_preview_selected_field {
//             SourceNovelPreviewSelection::Chapters => {
//                 let ch = app_state
//                     .buffer
//                     .chapter_previews
//                     .selected()
//                     .unwrap()
//                     .clone();
//                 let ch_no = ch.get_chapter_no();
//                 start_book_from_ch(
//                     app_state,
//                     ctx,
//                     app_state.buffer.novel.clone().unwrap(),
//                     ch_no,
//                 )
//                 .expect("chapter should exist");
//             }
//             _ => (),
//         },
//         _ => (),
//     }
// }

fn control_back(app_state: &mut AppState, ctx: &mut Context) {
    app_state.buffer.clear(); // See if this is valid

    // Reset styles
    // TODO: Make this work with custom styles when implemented
    app_state.config.selected_style = ConfigData::DEFAULT_SELECTED_STYLE;
    app_state.config.unselected_style = ConfigData::DEFAULT_UNSELECTED_STYLE;

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
            // We do nothing if the next chapter doesn't exist or we can't get to the next book for some reason
            let _ = goto_next_ch(app_state, ctx);
        }
        KeyCode::Left => {
            // Again, do nothing if it fails
            let _ = goto_prev_ch(app_state, ctx);
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
                    continue_book_history(app_state, ctx, b);
                }
                1 => {
                    // View book
                    // Prefer the library copy in case it's more up to date
                    // TODO: We should really avoid storing book data twice - look to change it later
                    let lib_copy = ctx.lib_find_book(book.get_id());
                    match lib_copy {
                        Some(lib_book) => {
                            enter_book_view(
                                app_state,
                                lib_book.global_get_novel().clone(),
                                BookViewType::History,
                            );
                        }
                        None => enter_book_view(
                            app_state,
                            book.global_get_novel().clone(),
                            BookViewType::History,
                        ),
                    }
                }
                2 => {
                    remove_history_entry(app_state, ctx, book.get_id());
                    app_state.update_screen(Screen::History(HistoryScreen::Main));
                }
                _ => unreachable!(),
            }
        }
        _ => (),
    }
}
