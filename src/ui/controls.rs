use std::thread;

use crate::{
    appstate::{
        AppState, BookInfo, BookSource, CurrentScreen, HistoryOptions, LibBookInfo, LibraryOptions,
        MiscOptions, RequestData, SettingsOptions, SourceOptions, UpdateOptions,
    },
    commands::{parse_command, run_command},
    global::sources::{source_data::SourceBookBox, Scrape, SortOrder},
    helpers::StatefulList,
    reader::buffer::BookProgress,
};
use anyhow::Result;
use crossterm::event::{self, KeyCode};
use ratatui::widgets::ListState;

pub fn handle_controls(app_state: &mut AppState, mut event: event::KeyCode) -> Result<bool> {
    // Command bar logic
    if app_state.command_bar {
        return handle_commands(event, app_state);
    }
    if matches!(event, KeyCode::Char(':')) {
        app_state.command_bar = true;
    }
    // On the rename screen, we just want to append to the string
    if matches!(app_state.current_screen, CurrentScreen::Typing) {
        handle_typing(event, app_state)?;
        return Ok(false);
    }
    // Aliased keys can be set here:
    event = match event {
        KeyCode::Char('h') => KeyCode::Left,
        KeyCode::Char('j') => KeyCode::Down,
        KeyCode::Char('k') => KeyCode::Up,
        KeyCode::Char('l') => KeyCode::Right,
        x => x,
    };
    // Go back, quitting if required.
    if (matches!(event, KeyCode::Esc) || matches!(event, KeyCode::Char('q')))
        && control_back(app_state)?
    {
        return Ok(true);
    }
    // Other times when we're not inputting:
    match app_state.current_screen {
        CurrentScreen::Misc(option) => match option {
            MiscOptions::ChapterView => {
                control_chapter_view(app_state, event)?;
            }
        },
        CurrentScreen::Library(option) => match option {
            LibraryOptions::Default => {
                control_main_menu(app_state, event);
                control_library_menu(app_state, event)?;
            }
            LibraryOptions::LocalBookSelect => control_library_local_book_select(app_state, event)?,
            LibraryOptions::GlobalBookSelect => {
                control_library_global_book_select(app_state, event)?
            }
            LibraryOptions::CategorySelect => control_library_category_select(app_state, event),
            LibraryOptions::CategoryOptions => control_library_category_options(app_state, event),
        },
        CurrentScreen::Updates(option) => match option {
            UpdateOptions::Default => control_main_menu(app_state, event),
        },
        CurrentScreen::Sources(option) => match option {
            SourceOptions::Default => {
                control_main_menu(app_state, event);
                control_sources_menu(app_state, event);
            }
            SourceOptions::SourceSelect => control_source_select(app_state, event)?,
            SourceOptions::SearchResults => control_source_search_result(app_state, event)?,
            SourceOptions::BookView => control_source_book_view(app_state, event)?,
        },
        CurrentScreen::History(option) => match option {
            HistoryOptions::Default => {
                control_main_menu(app_state, event);
                control_history_menu(app_state, event)?;
            }
            HistoryOptions::HistoryLocalBookOptions => {
                control_history_local_book_select(app_state, event)?
            }
            HistoryOptions::HistoryGlobalBookOptions => {
                control_history_global_book_select(app_state, event)?
            }
        },
        CurrentScreen::Settings(option) => match option {
            SettingsOptions::Default => control_main_menu(app_state, event),
        },
        CurrentScreen::Reader => control_reader(app_state, event)?,
        CurrentScreen::Typing => unreachable!(),
    }

    Ok(false)
}

fn control_back(app_state: &mut AppState) -> Result<bool> {
    if app_state.current_screen.in_reader() {
        app_state.update_from_reader()?;
    }

    if app_state.prev_screens.is_empty() {
        return Ok(true);
    }
    let mut prev = app_state.prev_screens.pop().unwrap();
    if prev == CurrentScreen::Typing {
        if app_state.prev_screens.is_empty() {
            return Ok(true);
        } else {
            prev = app_state.prev_screens.pop().unwrap();
        }
    }

    app_state.current_screen = prev;
    Ok(false)
}

fn handle_commands(event: KeyCode, app_state: &mut AppState) -> Result<bool> {
    match event {
        KeyCode::Enter => {
            let cmd = parse_command(&app_state.buffer.text);
            app_state.buffer.text = String::new();
            app_state.command_bar = false;
            return run_command(cmd);
        }
        KeyCode::Backspace => {
            app_state.buffer.text.pop();
        }
        KeyCode::Char(c) => {
            app_state.buffer.text.push(c);
        }
        KeyCode::Esc => {
            app_state.buffer.text = String::new();
            app_state.command_bar = false;
        }
        _ => (),
    }

    return Ok(false);
}

fn handle_typing(event: KeyCode, app_state: &mut AppState) -> Result<(), anyhow::Error> {
    match event {
        KeyCode::Backspace => {
            app_state.buffer.text.pop();
        }
        KeyCode::Char(c) => {
            app_state.buffer.text.push(c);
        }
        KeyCode::Esc => {
            app_state.buffer.text = String::new();
            app_state.current_screen = app_state.prev_screens.pop().unwrap();
        }
        KeyCode::Enter => {
            match app_state.get_last_screen() {
                CurrentScreen::Library(LibraryOptions::LocalBookSelect)
                | CurrentScreen::Library(LibraryOptions::GlobalBookSelect) => {
                    let book = app_state
                        .library_data
                        .get_category_list()
                        .selected()
                        .unwrap()
                        .clone();
                    app_state
                        .library_data
                        .rename_book(book.id, app_state.buffer.text.clone());
                    app_state.to_lib_screen();
                }
                CurrentScreen::Library(LibraryOptions::CategoryOptions) => {
                    app_state
                        .library_data
                        .create_category(app_state.buffer.text.clone());
                    app_state.update_category_list();
                    app_state.to_lib_screen();
                }
                CurrentScreen::Library(LibraryOptions::CategorySelect) => {
                    app_state.library_data.rename_category(
                        app_state
                            .menu_options
                            .category_list
                            .selected()
                            .unwrap()
                            .clone(),
                        app_state.buffer.text.clone(),
                    );
                    app_state.update_category_list();
                    app_state.to_lib_screen();
                    app_state
                        .update_screen(CurrentScreen::Library(LibraryOptions::CategoryOptions));
                }
                CurrentScreen::Sources(SourceOptions::SourceSelect) => {
                    app_state.channels.loading = true;
                    let source = app_state.source_data.get_list().selected().unwrap().clone();
                    let text = app_state.buffer.text.clone();
                    let tx = app_state.channels.sender.clone();
                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let res = source.search_novels(&text);
                        let _ = tx.send(RequestData::SearchResults(res));
                    });
                }
                _ => unreachable!(),
            }
            app_state.buffer.text = String::new();
        }
        _ => (),
    };
    Ok(())
}

fn control_main_menu(app_state: &mut AppState, event: event::KeyCode) {
    match event {
        KeyCode::Char(']') | KeyCode::Tab => {
            app_state.current_main_tab.next();
            match app_state.current_main_tab.index {
                0 => app_state.current_screen = CurrentScreen::Library(LibraryOptions::Default),
                1 => app_state.current_screen = CurrentScreen::Updates(UpdateOptions::Default),
                2 => app_state.current_screen = CurrentScreen::Sources(SourceOptions::Default),
                3 => app_state.current_screen = CurrentScreen::History(HistoryOptions::Default),
                4 => app_state.current_screen = CurrentScreen::Settings(SettingsOptions::Default),
                _ => unreachable!(),
            };
        }
        KeyCode::Char('[') | KeyCode::BackTab => {
            app_state.current_main_tab.previous();
            match app_state.current_main_tab.index {
                0 => app_state.current_screen = CurrentScreen::Library(LibraryOptions::Default),
                1 => app_state.current_screen = CurrentScreen::Updates(UpdateOptions::Default),
                2 => app_state.current_screen = CurrentScreen::Sources(SourceOptions::Default),
                3 => app_state.current_screen = CurrentScreen::History(HistoryOptions::Default),
                4 => app_state.current_screen = CurrentScreen::Settings(SettingsOptions::Default),
                _ => unreachable!(),
            };
        }
        _ => (),
    }
}

fn control_reader(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Up => {
            app_state.reader_data.as_mut().unwrap().scroll_up(1);
        }
        KeyCode::Down => {
            app_state.reader_data.as_mut().unwrap().scroll_down(1);
        }
        KeyCode::Right => {
            let ch = app_state
                .reader_data
                .as_ref()
                .unwrap()
                .book_info
                .get_source_data()
                .get_next_chapter();

            if let Some(c) = ch {
                let mut book = app_state.reader_data.as_ref().unwrap().book_info.clone();

                let in_library = matches!(book, BookInfo::Library(_));

                book.get_source_data_mut().set_chapter(c);
                let novel = book.get_novel().unwrap().clone();
                let source = app_state
                    .source_data
                    .get_source_by_id(book.get_source_id().unwrap())
                    .clone();
                let tx = app_state.channels.sender.clone();

                app_state.channels.loading = true;
                thread::spawn(move || {
                    let text = source
                        .parse_chapter(novel.novel_url.clone(), novel.get_chapter_url(c).unwrap());
                    if in_library {
                        let _ = tx.send(RequestData::Chapter((text, c)));
                    } else {
                        let _ = tx.send(RequestData::ChapterTemp((text, c)));
                    }
                });
            }
        }
        KeyCode::Left => {
            let ch = app_state
                .reader_data
                .as_ref()
                .unwrap()
                .book_info
                .get_source_data()
                .get_prev_chapter();

            if let Some(c) = ch {
                let book = app_state.reader_data.as_ref().unwrap().book_info.clone();

                let in_library = matches!(book, BookInfo::Library(_));

                let novel = book.get_novel().unwrap().clone();
                let source = app_state
                    .source_data
                    .get_source_by_id(book.get_source_id().unwrap())
                    .clone();
                let tx = app_state.channels.sender.clone();

                app_state.channels.loading = true;
                thread::spawn(move || {
                    let text = source
                        .parse_chapter(novel.novel_url.clone(), novel.get_chapter_url(c).unwrap());
                    if in_library {
                        let _ = tx.send(RequestData::Chapter((text, c)));
                    } else {
                        let _ = tx.send(RequestData::ChapterTemp((text, c)));
                    }
                });
            }
        }
        _ => (),
    }
    Ok(())
}

fn control_history_menu(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    if app_state.history_data.history.is_empty() {
        return Ok(());
    }
    let hist_len = app_state.history_data.history.len();
    match event {
        KeyCode::Up => {
            if app_state.history_data.selected == ListState::default()
                || app_state.history_data.selected.selected() == Some(0)
            {
                app_state.history_data.selected.select(Some(hist_len - 1))
            } else {
                app_state
                    .history_data
                    .selected
                    .select(app_state.history_data.selected.selected().map(|i| i - 1))
            }
        }
        KeyCode::Down => {
            if app_state.history_data.selected == ListState::default() {
                app_state.history_data.selected.select(Some(0))
            } else {
                app_state.history_data.selected.select(
                    app_state
                        .history_data
                        .selected
                        .selected()
                        .map(|i| (i + 1) % hist_len),
                )
            }
        }
        KeyCode::Enter => {
            let book_idx = app_state.history_data.selected.selected();
            if book_idx.is_none() {
                return Ok(());
            }
            if app_state.history_data.history[book_idx.unwrap()]
                .book
                .is_local()
            {
                app_state.update_screen(CurrentScreen::History(
                    HistoryOptions::HistoryLocalBookOptions,
                ))
            } else {
                app_state.update_screen(CurrentScreen::History(
                    HistoryOptions::HistoryGlobalBookOptions,
                ))
            }
        }
        _ => (),
    }
    Ok(())
}
fn control_library_menu(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Char('}') | KeyCode::Right => {
            // Possibly redundant, needs testing
            if app_state.current_main_tab.in_library() {
                app_state.library_data.categories.next();
            }
        }
        KeyCode::Char('{') | KeyCode::Left => {
            // Possibly redundant, needs testing
            if app_state.current_main_tab.in_library() {
                app_state.library_data.categories.previous();
            }
        }
        KeyCode::Up => {
            app_state.library_data.get_category_list_mut().previous();
        }
        KeyCode::Down => {
            app_state.library_data.get_category_list_mut().next();
        }
        KeyCode::Enter => {
            let b = app_state.library_data.get_category_list().selected();
            if b.is_none() {
                return Ok(());
            } else {
                let book = b.unwrap();
                if book.is_local() {
                    app_state
                        .update_screen(CurrentScreen::Library(LibraryOptions::LocalBookSelect));
                } else {
                    app_state
                        .update_screen(CurrentScreen::Library(LibraryOptions::GlobalBookSelect));
                }
            }
        }
        KeyCode::Char('c') => {
            // Opens a menu in which you can create / delete / reorder categories
            // Select the first option upon entering
            app_state
                .menu_options
                .category_options
                .state
                .select(Some(0));
            app_state.update_screen(CurrentScreen::Library(LibraryOptions::CategoryOptions))
        }
        _ => (),
    }
    Ok(())
}

fn control_library_local_book_select(
    app_state: &mut AppState,
    event: event::KeyCode,
) -> Result<()> {
    match event {
        KeyCode::Up => app_state.menu_options.local_options.previous(),
        KeyCode::Down => app_state.menu_options.local_options.next(),
        KeyCode::Enter => {
            let option = app_state
                .menu_options
                .local_options
                .state
                .selected()
                .unwrap();

            let mut book = app_state
                .library_data
                .get_category_list()
                .selected()
                .unwrap()
                .clone();
            match option {
                0 => {
                    // Resume location
                    app_state.move_to_reader(BookInfo::Library(book), None, None)?;
                }
                1 => {
                    // Move to category
                    app_state.update_screen(CurrentScreen::Library(LibraryOptions::CategorySelect));
                }
                2 => {
                    // Rename book
                    app_state.update_screen(CurrentScreen::Typing);
                }
                3 => {
                    // Start from beginning
                    book.update_progress(BookProgress::NONE);
                    app_state.move_to_reader(BookInfo::Library(book), None, None)?;
                }
                4 => {
                    // Remove from library
                    app_state.library_data.remove_book(book.id);
                    app_state.to_lib_screen();
                }
                _ => unreachable!(),
            }
            app_state.menu_options.local_options.state.select(Some(0));
        }
        _ => (),
    }
    Ok(())
}

fn control_library_global_book_select(
    app_state: &mut AppState,
    event: event::KeyCode,
) -> Result<()> {
    match event {
        KeyCode::Up => app_state.menu_options.global_options.previous(),
        KeyCode::Down => app_state.menu_options.global_options.next(),
        KeyCode::Enter => {
            let option = app_state
                .menu_options
                .global_options
                .state
                .selected()
                .unwrap();

            let mut book = app_state
                .library_data
                .get_category_list()
                .selected()
                .unwrap()
                .clone();

            match option {
                // Resume
                0 => {
                    let ch = book.source_data.get_chapter();
                    let progress = book.get_progress();
                    let mut book = BookInfo::Library(book);
                    let novel = book.get_novel().unwrap().clone();
                    let source = app_state
                        .source_data
                        .get_source_by_id(book.get_source_id().unwrap())
                        .clone();
                    let tx = app_state.channels.sender.clone();

                    if matches!(progress, BookProgress::Finished) {
                        let next = book.get_source_data().get_next_chapter();
                        if let Some(c) = next {
                            book.get_source_data_mut().set_chapter(c);

                            app_state.channels.loading = true;
                            thread::spawn(move || {
                                let text = source.parse_chapter(
                                    novel.novel_url.clone(),
                                    novel.get_chapter_url(c).unwrap(),
                                );
                                let _ = tx.send(RequestData::Chapter((text, ch)));
                            });
                            return Ok(());
                        }
                    }

                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let text = source.parse_chapter(
                            novel.novel_url.clone(),
                            novel.get_chapter_url(ch).unwrap(),
                        );
                        let _ = tx.send(RequestData::Chapter((text, ch)));
                    });

                    // app_state.move_to_reader(book, Some(ch))?;
                }
                // Chapter List
                1 => {
                    let novel = if let BookSource::Global(d) = book.source_data {
                        d.novel.clone()
                    } else {
                        unreachable!()
                    };

                    app_state.buffer.ch_list_setup();
                    app_state.buffer.chapter_previews =
                        StatefulList::with_items(novel.chapters.clone());
                    app_state.buffer.novel = Some(novel);

                    app_state.update_screen(CurrentScreen::Misc(MiscOptions::ChapterView));
                }
                // Move Category
                2 => {
                    app_state.update_screen(CurrentScreen::Library(LibraryOptions::CategorySelect))
                }
                // Rename
                3 => {
                    app_state.update_screen(CurrentScreen::Typing);
                }
                // Restart
                4 => {
                    book.source_data.set_chapter(1);
                    book.source_data.clear_chapter_data();
                    book.source_data.set_chapter(1);

                    let book = BookInfo::Library(book);

                    let novel = book.get_novel().unwrap().clone();
                    let source = app_state
                        .source_data
                        .get_source_by_id(book.get_source_id().unwrap())
                        .clone();
                    let tx = app_state.channels.sender.clone();

                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let text = source.parse_chapter(
                            novel.novel_url.clone(),
                            novel.get_chapter_url(1).unwrap(),
                        );
                        let _ = tx.send(RequestData::Chapter((text, 1)));
                    });
                }
                // Remove
                5 => {
                    app_state.library_data.remove_book(book.id);
                    app_state.to_lib_screen();
                }
                _ => unreachable!(),
            }

            app_state.menu_options.global_options.state.select(Some(0));
        }
        _ => (),
    }
    Ok(())
}

fn control_library_category_select(app_state: &mut AppState, event: event::KeyCode) {
    match event {
        KeyCode::Up => app_state.menu_options.category_list.previous(),
        KeyCode::Down => app_state.menu_options.category_list.next(),
        KeyCode::Enter => {
            if app_state.get_last_screen()
                == CurrentScreen::Library(LibraryOptions::CategoryOptions)
            {
                let opt = app_state
                    .menu_options
                    .category_options
                    .state
                    .selected()
                    .unwrap();

                if opt == 2 {
                    // Rename
                    app_state.update_screen(CurrentScreen::Typing)
                } else if opt == 3 {
                    // Delete
                    app_state.library_data.delete_category(
                        app_state
                            .menu_options
                            .category_list
                            .selected()
                            .unwrap()
                            .clone(),
                    );
                    app_state.menu_options.category_list.state.select(Some(0));
                    app_state.update_category_list();
                    app_state.to_lib_screen();
                    app_state
                        .update_screen(CurrentScreen::Library(LibraryOptions::CategoryOptions));
                } else {
                    unreachable!();
                }
            } else {
                let book = app_state
                    .library_data
                    .get_category_list()
                    .selected()
                    .unwrap()
                    .id;

                let category = app_state
                    .menu_options
                    .category_list
                    .selected()
                    .unwrap()
                    .clone();

                app_state.library_data.move_category(book, Some(category));

                app_state.to_lib_screen();
            }
        }
        _ => (),
    }
}

fn control_library_category_options(app_state: &mut AppState, event: event::KeyCode) {
    match event {
        KeyCode::Up => app_state.menu_options.category_options.previous(),
        KeyCode::Down => app_state.menu_options.category_options.next(),
        KeyCode::Enter => {
            let choice = app_state
                .menu_options
                .category_options
                .state
                .selected()
                .unwrap();
            match choice {
                // Create a category
                0 => app_state.update_screen(CurrentScreen::Typing),
                // Reorder categories
                1 => (),
                // Rename & delete
                2 | 3 => {
                    app_state.update_screen(CurrentScreen::Library(LibraryOptions::CategorySelect))
                }
                _ => unreachable!(),
            }
        }

        _ => (),
    }
}

fn control_chapter_view(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Up => {
            if app_state.buffer.novel_preview_selection == SourceBookBox::Chapters {
                app_state.buffer.chapter_previews.previous();
            } else if app_state.buffer.novel_preview_scroll != 0 {
                app_state.buffer.novel_preview_scroll -= 1;
            }
        }
        KeyCode::Down => {
            if app_state.buffer.novel_preview_selection == SourceBookBox::Chapters {
                app_state.buffer.chapter_previews.next();
            } else {
                app_state.buffer.novel_preview_scroll += 1;
            }
        }
        KeyCode::Enter => {
            if app_state.buffer.novel_preview_selection == SourceBookBox::Chapters {
                // Chapters
                let chap = app_state.buffer.chapter_previews.state.selected().unwrap();
                let book = app_state
                    .library_data
                    .get_category_list()
                    .selected()
                    .unwrap();

                let mut book = book.clone();
                book.source_data.set_chapter(chap + 1);

                let book = BookInfo::Library(book);
                let novel = book.get_novel().unwrap().clone();
                let source = app_state
                    .source_data
                    .get_source_by_id(book.get_source_id().unwrap())
                    .clone();
                let tx = app_state.channels.sender.clone();
                app_state.channels.loading = true;

                thread::spawn(move || {
                    let text = source.parse_chapter(
                        novel.novel_url.clone(),
                        novel.get_chapter_url(chap + 1).unwrap(),
                    );
                    let _ = tx.send(RequestData::Chapter((text, chap + 1)));
                });
            }
        }
        KeyCode::Char('[') | KeyCode::BackTab | KeyCode::Char(']') | KeyCode::Tab => {
            if app_state.buffer.novel_preview_selection == SourceBookBox::Summary {
                app_state.buffer.novel_preview_selection = SourceBookBox::Chapters
            } else {
                app_state.buffer.novel_preview_selection = SourceBookBox::Summary
            }
        }
        _ => (),
    }
    Ok(())
}

fn control_sources_menu(app_state: &mut AppState, event: event::KeyCode) {
    match event {
        KeyCode::Up => {
            app_state.source_data.get_list_mut().previous();
        }
        KeyCode::Down => {
            app_state.source_data.get_list_mut().next();
        }
        KeyCode::Enter => {
            let source = app_state.source_data.get_list().selected();
            if source.is_some() {
                app_state.update_screen(CurrentScreen::Sources(SourceOptions::SourceSelect));
            }
        }
        _ => (),
    }
}

fn control_source_select(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Up => app_state.menu_options.source_options.previous(),
        KeyCode::Down => app_state.menu_options.source_options.next(),
        KeyCode::Enter => {
            let option = app_state
                .menu_options
                .source_options
                .state
                .selected()
                .unwrap();
            // 0 = search
            // 1 = view popular
            match option {
                0 => {
                    app_state.update_screen(CurrentScreen::Typing);
                }
                1 => {
                    app_state.channels.loading = true;
                    let source = app_state.source_data.get_list().selected().unwrap().clone();
                    let tx = app_state.channels.sender.clone();
                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let res = source.get_popular(SortOrder::Rating, 1);
                        let _ = tx.send(RequestData::SearchResults(res));
                    });
                }
                _ => (),
            }
        }
        _ => (),
    }
    Ok(())
}

fn control_source_book_view(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Char(']') | KeyCode::Tab => match app_state.buffer.novel_preview_selection {
            SourceBookBox::Options => {
                app_state.buffer.novel_preview_selection = SourceBookBox::Chapters
            }
            SourceBookBox::Chapters => {
                app_state.buffer.novel_preview_selection = SourceBookBox::Summary
            }
            SourceBookBox::Summary => {
                app_state.buffer.novel_preview_selection = SourceBookBox::Options
            }
        },
        KeyCode::Char('[') | KeyCode::BackTab => match app_state.buffer.novel_preview_selection {
            SourceBookBox::Options => {
                app_state.buffer.novel_preview_selection = SourceBookBox::Summary
            }
            SourceBookBox::Chapters => {
                app_state.buffer.novel_preview_selection = SourceBookBox::Options
            }
            SourceBookBox::Summary => {
                app_state.buffer.novel_preview_selection = SourceBookBox::Chapters
            }
        },

        KeyCode::Up => match app_state.buffer.novel_preview_selection {
            SourceBookBox::Options => app_state.menu_options.source_book_options.previous(),
            SourceBookBox::Chapters => app_state.buffer.chapter_previews.previous(),
            SourceBookBox::Summary => {
                if app_state.buffer.novel_preview_scroll != 0 {
                    app_state.buffer.novel_preview_scroll -= 1;
                }
            }
        },
        KeyCode::Down => match app_state.buffer.novel_preview_selection {
            SourceBookBox::Options => app_state.menu_options.source_book_options.next(),
            SourceBookBox::Chapters => app_state.buffer.chapter_previews.next(),
            SourceBookBox::Summary => {
                app_state.buffer.novel_preview_scroll += 1;
            }
        },
        KeyCode::Enter => {
            match app_state.buffer.novel_preview_selection {
                SourceBookBox::Options => {
                    let opt = app_state
                        .menu_options
                        .source_book_options
                        .state
                        .selected()
                        .unwrap();
                    match opt {
                        // Start from beginning
                        0 => {
                            let novel = app_state.buffer.novel.clone().unwrap();
                            let source = app_state.source_data.sources.selected().unwrap().clone();
                            let tx = app_state.channels.sender.clone();
                            app_state.channels.loading = true;

                            thread::spawn(move || {
                                let text = source.parse_chapter(
                                    novel.novel_url.clone(),
                                    novel.get_chapter_url(1).unwrap(),
                                );
                                let _ = tx.send(RequestData::ChapterTemp((text, 1)));
                            });
                        }
                        // Add to lib
                        1 => {
                            let novel = app_state.buffer.novel.clone().unwrap();
                            app_state
                                .library_data
                                .add_book(LibBookInfo::from_global(novel, None)?, None);
                            app_state.current_screen = app_state.prev_screens.pop().unwrap();
                        }
                        _ => (),
                    }
                }
                SourceBookBox::Chapters => {
                    let chap = app_state
                        .buffer
                        .chapter_previews
                        .selected()
                        .unwrap()
                        .clone();
                    let novel = app_state.buffer.novel.clone().unwrap();
                    let mut info = BookInfo::from_novel_temp(novel, chap.chapter_no)?;
                    info.get_source_data_mut().set_chapter(chap.chapter_no);
                    // Yes, this is necessary
                    let novel = info.get_novel().unwrap().clone();

                    let source = app_state
                        .source_data
                        .get_source_by_id(info.get_source_id().unwrap())
                        .clone();
                    let tx = app_state.channels.sender.clone();
                    app_state.channels.loading = true;

                    thread::spawn(move || {
                        let text = source.parse_chapter(
                            novel.novel_url.clone(),
                            novel.get_chapter_url(chap.chapter_no).unwrap(),
                        );
                        let _ = tx.send(RequestData::ChapterTemp((text, chap.chapter_no)));
                    });

                    // app_state.move_to_reader(info, Some(chap.chapter_no))?;
                }
                SourceBookBox::Summary => (),
            }
        }
        _ => (),
    }
    Ok(())
}

fn control_source_search_result(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Up => app_state.buffer.novel_previews.previous(),
        KeyCode::Down => app_state.buffer.novel_previews.next(),
        KeyCode::Enter => {
            let url = app_state
                .buffer
                .novel_previews
                .selected()
                .unwrap()
                .url
                .clone();
            let source = app_state.source_data.sources.selected().unwrap().clone();
            let tx = app_state.channels.sender.clone();

            app_state.channels.loading = true;
            thread::spawn(move || {
                let full_book = source.parse_novel_and_chapters(url);
                let _ = tx.send(RequestData::BookInfo(full_book));
            });
        }
        _ => (),
    }
    Ok(())
}

fn control_history_global_book_select(
    app_state: &mut AppState,
    event: event::KeyCode,
) -> Result<()> {
    match event {
        KeyCode::Up => app_state.menu_options.global_history_options.previous(),
        KeyCode::Down => app_state.menu_options.global_history_options.next(),
        KeyCode::Enter => {
            let option = app_state
                .menu_options
                .global_history_options
                .state
                .selected()
                .unwrap();

            let idx = app_state.history_data.selected.selected();
            if idx.is_none() {
                return Ok(());
            }

            let mut book = app_state.history_data.history[idx.unwrap()].book.clone();

            match option {
                0 => {
                    // Continue reading

                    let is_lib_book = matches!(book, BookInfo::Library(_));

                    let ch = book.get_source_data().get_chapter();
                    let progress = book.get_progress();
                    let novel = book.get_novel().unwrap().clone();
                    let source = app_state
                        .source_data
                        .get_source_by_id(book.get_source_id().unwrap())
                        .clone();
                    let tx = app_state.channels.sender.clone();

                    if matches!(progress, BookProgress::Finished) {
                        let next = book.get_source_data().get_next_chapter();
                        if let Some(c) = next {
                            book.get_source_data_mut().set_chapter(c);

                            app_state.channels.loading = true;
                            thread::spawn(move || {
                                let text = source.parse_chapter(
                                    novel.novel_url.clone(),
                                    novel.get_chapter_url(c).unwrap(),
                                );
                                let _ = tx.send(RequestData::Chapter((text, ch)));
                            });
                            return Ok(());
                        }
                    }

                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let text = source.parse_chapter(
                            novel.novel_url.clone(),
                            novel.get_chapter_url(ch).unwrap(),
                        );
                        if is_lib_book {
                            let _ = tx.send(RequestData::Chapter((text, ch)));
                        } else {
                            let _ = tx.send(RequestData::ChapterTemp((text, ch)));
                        }
                    });
                }
                1 => {
                    // View book
                    let source = app_state
                        .source_data
                        .get_source_by_id(book.get_source_id().unwrap())
                        .clone();
                    let tx = app_state.channels.sender.clone();

                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let novel = source
                            .parse_novel_and_chapters(book.get_novel().unwrap().novel_url.clone());
                        let _ = tx.send(RequestData::BookInfoNoOpts(novel));
                    });
                }
                2 => {
                    // Remove from history
                    app_state.history_data.history.remove(idx.unwrap());
                    if !app_state.history_data.history.is_empty() {
                        app_state
                            .history_data
                            .selected
                            .select(Some(idx.unwrap() - 1))
                    } else {
                        app_state.history_data.selected.select(None)
                    }
                    app_state.to_history_screen();
                }
                _ => (),
            }
        }
        _ => (),
    }
    Ok(())
}

fn control_history_local_book_select(
    app_state: &mut AppState,
    event: event::KeyCode,
) -> Result<()> {
    match event {
        KeyCode::Up => app_state.menu_options.local_history_options.previous(),
        KeyCode::Down => app_state.menu_options.local_history_options.next(),
        KeyCode::Enter => {
            let option = app_state
                .menu_options
                .local_history_options
                .state
                .selected()
                .unwrap();

            let idx = app_state.history_data.selected.selected();
            if idx.is_none() {
                return Ok(());
            }

            let book = app_state.history_data.history[idx.unwrap()].book.clone();

            match option {
                0 => {
                    // Continue reading
                    app_state.move_to_reader(book, None, None)?;
                }
                1 => {
                    // Remove from history
                    app_state.history_data.history.remove(idx.unwrap());
                    if idx.unwrap() != 0 {
                        app_state
                            .history_data
                            .selected
                            .select(Some(idx.unwrap() - 1))
                    } else {
                        app_state.history_data.selected.select(None)
                    }
                    app_state.to_history_screen();
                }
                _ => (),
            }
        }
        _ => (),
    }
    Ok(())
}
