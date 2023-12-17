use std::thread;

use crate::{
    appstate::{
        AppState, BookInfo, BookSource, CurrentScreen, HistoryOptions, LibBookInfo, LibraryOptions,
        RequestData, SettingsOptions, SourceOptions, UpdateOptions,
    },
    global::sources::{source_data::SourceBookBox, Scrape, SortOrder},
    helpers::StatefulList,
    reader::buffer::BookProgress,
};
use anyhow::Result;
use crossterm::event::{self, KeyCode};
use ratatui::widgets::ListState;

pub fn handle_controls(app_state: &mut AppState, event: event::KeyCode) -> Result<bool> {
    // On the rename screen, we just want to append to the string
    if matches!(app_state.current_screen, CurrentScreen::Typing) {
        handle_typing(event, app_state)?;
        return Ok(false);
    }
    // Go back, quitting if required.
    if matches!(event, KeyCode::Esc) || matches!(event, KeyCode::Char('q')) {
        if control_back(app_state)? {
            return Ok(true);
        }
    }
    // Other times when we're not inputting:
    match app_state.current_screen {
        CurrentScreen::Library(option) => match option {
            LibraryOptions::Default => {
                control_main_menu(app_state, event);
                control_library_menu(app_state, event)?;
            }
            LibraryOptions::LocalBookSelect => control_library_local_book_select(app_state, event)?,
            LibraryOptions::GlobalBookSelect => {
                control_library_global_book_select(app_state, event)?
            }
            LibraryOptions::MoveCategorySelect => control_library_move_category(app_state, event),
            LibraryOptions::ChapterView => control_library_chapter_view(app_state, event)?,
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
            HistoryOptions::HistoryBookOptions => {
                unimplemented!()
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
    return Ok(false);
}

fn handle_typing(event: KeyCode, app_state: &mut AppState) -> Result<(), anyhow::Error> {
    Ok(match event {
        KeyCode::Backspace => {
            app_state.text_buffer.pop();
        }
        KeyCode::Char(c) => {
            app_state.text_buffer.push(c);
        }
        KeyCode::Esc => {
            app_state.text_buffer = String::new();
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
                        .rename_book(book.id, app_state.text_buffer.clone());
                    app_state.to_lib_screen();
                }
                CurrentScreen::Sources(SourceOptions::SourceSelect) => {
                    app_state.channels.loading = true;
                    let source = app_state.source_data.get_list().selected().unwrap().clone();
                    let text = app_state.text_buffer.clone();
                    let tx = app_state.channels.sender.clone();
                    app_state.channels.loading = true;
                    thread::spawn(move || {
                        let res = source.search_novels(&text);
                        let _ = tx.send(RequestData::SearchResults(res));
                    });
                }
                _ => unreachable!(),
            }
            app_state.text_buffer = String::new();
        }
        _ => (),
    })
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
            app_state.update_screen(CurrentScreen::History(HistoryOptions::HistoryBookOptions))
        }
        _ => (),
    }
    Ok(())
}
fn control_library_menu(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Char('}') | KeyCode::Right => {
            if app_state.current_main_tab.in_library() {
                app_state.library_data.categories.next();
            }
        }
        KeyCode::Char('{') | KeyCode::Left => {
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
            // 0 = resume, 1 = move to category, 2 = rename, 3 = restart
            match option {
                0 => {
                    // Resume location
                    app_state.move_to_reader(BookInfo::Library(book), None, None)?;
                }
                1 => {
                    // Move to category
                    app_state
                        .update_screen(CurrentScreen::Library(LibraryOptions::MoveCategorySelect));
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
                            // app_state.move_to_reader(book, Some(c))?;
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
                    let chs = if let BookSource::Global(d) = book.source_data {
                        d.novel.chapters.clone()
                    } else {
                        unreachable!()
                    };

                    app_state.library_data.menu_data.ch_list = Some(StatefulList::with_items(chs));

                    app_state.update_screen(CurrentScreen::Library(LibraryOptions::ChapterView));
                }
                // Move Category
                2 => app_state
                    .update_screen(CurrentScreen::Library(LibraryOptions::MoveCategorySelect)),
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

fn control_library_move_category(app_state: &mut AppState, event: event::KeyCode) {
    match event {
        KeyCode::Up => app_state.menu_options.category_moves.previous(),
        KeyCode::Down => app_state.menu_options.category_moves.next(),
        KeyCode::Enter => {
            let book = app_state
                .library_data
                .get_category_list()
                .selected()
                .unwrap()
                .id;

            let category = app_state
                .menu_options
                .category_moves
                .selected()
                .unwrap()
                .clone();

            app_state.library_data.move_category(book, Some(category));

            app_state.to_lib_screen();
        }
        _ => (),
    }
}

fn control_library_chapter_view(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    match event {
        KeyCode::Up => {
            if app_state.library_data.menu_data.ch_selected {
                app_state
                    .library_data
                    .menu_data
                    .ch_list
                    .as_mut()
                    .unwrap()
                    .previous();
            } else {
                if app_state.library_data.menu_data.ch_scroll != 0 {
                    app_state.library_data.menu_data.ch_scroll -= 1;
                }
            }
        }
        KeyCode::Down => {
            if app_state.library_data.menu_data.ch_selected {
                app_state
                    .library_data
                    .menu_data
                    .ch_list
                    .as_mut()
                    .unwrap()
                    .next();
            } else {
                app_state.library_data.menu_data.ch_scroll += 1;
            }
        }
        KeyCode::Enter => {
            if app_state.library_data.menu_data.ch_selected {
                // Chapters
                let chap = app_state
                    .library_data
                    .menu_data
                    .ch_list
                    .as_ref()
                    .unwrap()
                    .state
                    .selected()
                    .unwrap();
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
            app_state.library_data.menu_data.ch_selected =
                !app_state.library_data.menu_data.ch_selected;
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
        KeyCode::Char(']') | KeyCode::Tab => match app_state.source_data.current_book_ui_option {
            SourceBookBox::Options => {
                app_state.source_data.current_book_ui_option = SourceBookBox::Chapters
            }
            SourceBookBox::Chapters => {
                app_state.source_data.current_book_ui_option = SourceBookBox::Summary
            }
            SourceBookBox::Summary => {
                app_state.source_data.current_book_ui_option = SourceBookBox::Options
            }
        },
        KeyCode::Char('[') | KeyCode::BackTab => match app_state.source_data.current_book_ui_option
        {
            SourceBookBox::Options => {
                app_state.source_data.current_book_ui_option = SourceBookBox::Summary
            }
            SourceBookBox::Chapters => {
                app_state.source_data.current_book_ui_option = SourceBookBox::Options
            }
            SourceBookBox::Summary => {
                app_state.source_data.current_book_ui_option = SourceBookBox::Chapters
            }
        },

        KeyCode::Up => match app_state.source_data.current_book_ui_option {
            SourceBookBox::Options => app_state.menu_options.source_book_options.previous(),
            SourceBookBox::Chapters => app_state.source_data.current_novel_chaps.previous(),
            SourceBookBox::Summary => {
                if app_state.source_data.current_novel_scroll != 0 {
                    app_state.source_data.current_novel_scroll -= 1;
                }
            }
        },
        KeyCode::Down => match app_state.source_data.current_book_ui_option {
            SourceBookBox::Options => app_state.menu_options.source_book_options.next(),
            SourceBookBox::Chapters => app_state.source_data.current_novel_chaps.next(),
            SourceBookBox::Summary => {
                app_state.source_data.current_novel_scroll += 1;
            }
        },
        KeyCode::Enter => {
            match app_state.source_data.current_book_ui_option {
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
                            let novel = app_state.source_data.current_novel.clone().unwrap();
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
                            let novel = app_state.source_data.current_novel.clone().unwrap();
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
                        .source_data
                        .current_novel_chaps
                        .selected()
                        .unwrap()
                        .clone();
                    let novel = app_state.source_data.current_novel.clone().unwrap();
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
        KeyCode::Up => app_state.source_data.novel_results.previous(),
        KeyCode::Down => app_state.source_data.novel_results.next(),
        KeyCode::Enter => {
            let url = app_state
                .source_data
                .novel_results
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
