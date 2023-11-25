use crate::{
    appstate::{
        AppState, BookInfo, BookSource, CurrentScreen, HistoryOptions, LibBookInfo, LibraryOptions,
        SettingsOptions, SourceOptions, UpdateOptions,
    },
    global::sources::{source_data::SourceBookBox, SortOrder},
    helpers::StatefulList,
    reader::buffer::BookProgress,
};
use anyhow::Result;
use crossterm::event::{self, KeyCode};

pub fn handle_controls(app_state: &mut AppState, event: event::KeyCode) -> Result<bool> {
    // On the rename screen, we just want to append to the string
    if matches!(app_state.current_screen, CurrentScreen::Typing) {
        handle_typing(event, app_state)?;
        return Ok(false);
    }
    // Other times when we're not inputting:
    match event {
        event::KeyCode::Esc | event::KeyCode::Char('q') => return control_back(app_state),
        KeyCode::Left | KeyCode::Right | KeyCode::Down | KeyCode::Up => {
            control_arrows(app_state, event)?;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            control_enter(app_state)?;
        }
        KeyCode::Char('[') | KeyCode::Char(']') | KeyCode::Char('{') | KeyCode::Char('}') => {
            control_bracket(app_state, event)
        }
        _ => (),
    }

    Ok(false)
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
                    let source = &app_state.source_data.get_list().selected().unwrap().1;
                    let res = source.search_novels(&app_state.text_buffer)?;
                    app_state.source_data.novel_results = StatefulList::with_items(res);
                    app_state.update_screen(CurrentScreen::Sources(SourceOptions::SearchResults));
                }
                _ => unreachable!(),
            }
            app_state.text_buffer = String::new();
        }
        _ => (),
    })
}

fn control_bracket(app_state: &mut AppState, event: event::KeyCode) {
    if app_state.current_screen.on_main_menu() {
        match event {
            KeyCode::Char(']') => {
                app_state.current_main_tab.next();
                match app_state.current_main_tab.index {
                    0 => app_state.current_screen = CurrentScreen::Library(LibraryOptions::Default),
                    1 => app_state.current_screen = CurrentScreen::Updates(UpdateOptions::Default),
                    2 => app_state.current_screen = CurrentScreen::Sources(SourceOptions::Default),
                    3 => app_state.current_screen = CurrentScreen::History(HistoryOptions::Default),
                    4 => {
                        app_state.current_screen = CurrentScreen::Settings(SettingsOptions::Default)
                    }
                    _ => unreachable!(),
                };
            }
            KeyCode::Char('[') => {
                app_state.current_main_tab.previous();
                match app_state.current_main_tab.index {
                    0 => app_state.current_screen = CurrentScreen::Library(LibraryOptions::Default),
                    1 => app_state.current_screen = CurrentScreen::Updates(UpdateOptions::Default),
                    2 => app_state.current_screen = CurrentScreen::Sources(SourceOptions::Default),
                    3 => app_state.current_screen = CurrentScreen::History(HistoryOptions::Default),
                    4 => {
                        app_state.current_screen = CurrentScreen::Settings(SettingsOptions::Default)
                    }
                    _ => unreachable!(),
                };
            }
            _ => (),
        }
    }
    if app_state.current_screen.on_library_menu() {
        match event {
            KeyCode::Char('}') => {
                if app_state.current_main_tab.in_library() {
                    app_state.library_data.categories.next();
                }
            }
            KeyCode::Char('{') => {
                if app_state.current_main_tab.in_library() {
                    app_state.library_data.categories.previous();
                }
            }
            _ => (),
        }
    }

    if matches!(
        app_state.current_screen,
        CurrentScreen::Sources(SourceOptions::BookView)
    ) {
        match event {
            KeyCode::Char(']') => match app_state.source_data.current_book_ui_option {
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
            KeyCode::Char('[') => match app_state.source_data.current_book_ui_option {
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
            _ => (),
        }
    }

    if matches!(
        app_state.current_screen,
        CurrentScreen::Library(LibraryOptions::ChapterView)
    ) {
        match event {
            KeyCode::Char('[') | KeyCode::Char(']') => {
                app_state.library_data.menu_data.ch_selected =
                    !app_state.library_data.menu_data.ch_selected;
            }
            _ => (),
        }
    }
}

fn control_back(app_state: &mut AppState) -> Result<bool> {
    if app_state.current_screen.in_reader() {
        app_state.update_lib_from_reader()?;
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

fn control_arrows(app_state: &mut AppState, event: event::KeyCode) -> Result<()> {
    if app_state.current_screen.on_main_menu() {
        match event {
            KeyCode::Right => {
                app_state.current_main_tab.next();
                match app_state.current_main_tab.index {
                    0 => app_state.current_screen = CurrentScreen::Library(LibraryOptions::Default),
                    1 => app_state.current_screen = CurrentScreen::Updates(UpdateOptions::Default),
                    2 => app_state.current_screen = CurrentScreen::Sources(SourceOptions::Default),
                    3 => app_state.current_screen = CurrentScreen::History(HistoryOptions::Default),
                    4 => {
                        app_state.current_screen = CurrentScreen::Settings(SettingsOptions::Default)
                    }

                    _ => unreachable!(),
                };
            }
            KeyCode::Left => {
                app_state.current_main_tab.previous();
                match app_state.current_main_tab.index {
                    0 => app_state.current_screen = CurrentScreen::Library(LibraryOptions::Default),
                    1 => app_state.current_screen = CurrentScreen::Updates(UpdateOptions::Default),
                    2 => app_state.current_screen = CurrentScreen::Sources(SourceOptions::Default),
                    3 => app_state.current_screen = CurrentScreen::History(HistoryOptions::Default),
                    4 => {
                        app_state.current_screen = CurrentScreen::Settings(SettingsOptions::Default)
                    }
                    _ => unreachable!(),
                };
            }
            _ => (),
        }
    }

    match app_state.current_screen {
        CurrentScreen::Library(LibraryOptions::Default) => match event {
            KeyCode::Up => {
                app_state.library_data.get_category_list_mut().previous();
            }
            KeyCode::Down => {
                app_state.library_data.get_category_list_mut().next();
            }
            _ => (),
        },
        CurrentScreen::Sources(SourceOptions::Default) => match event {
            KeyCode::Up => {
                app_state.source_data.get_list_mut().previous();
            }
            KeyCode::Down => {
                app_state.source_data.get_list_mut().next();
            }
            _ => (),
        },
        CurrentScreen::Library(LibraryOptions::LocalBookSelect) => match event {
            KeyCode::Up => app_state.menu_options.local_options.previous(),
            KeyCode::Down => app_state.menu_options.local_options.next(),
            _ => (),
        },
        CurrentScreen::Library(LibraryOptions::GlobalBookSelect) => match event {
            KeyCode::Up => app_state.menu_options.global_options.previous(),
            KeyCode::Down => app_state.menu_options.global_options.next(),
            _ => (),
        },
        CurrentScreen::Library(LibraryOptions::MoveCategorySelect) => match event {
            KeyCode::Up => app_state.menu_options.category_moves.previous(),
            KeyCode::Down => app_state.menu_options.category_moves.next(),
            _ => (),
        },
        CurrentScreen::Sources(SourceOptions::SourceSelect) => match event {
            KeyCode::Up => app_state.menu_options.source_options.previous(),
            KeyCode::Down => app_state.menu_options.source_options.next(),
            _ => (),
        },
        CurrentScreen::Library(LibraryOptions::ChapterView) => match event {
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
            _ => (),
        },
        CurrentScreen::Sources(SourceOptions::SearchResults) => match event {
            KeyCode::Up => app_state.source_data.novel_results.previous(),
            KeyCode::Down => app_state.source_data.novel_results.next(),
            _ => (),
        },
        CurrentScreen::Sources(SourceOptions::BookView) => match event {
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
            _ => (),
        },
        CurrentScreen::Reader => match event {
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

                    book.get_source_data_mut().set_chapter(c);
                    app_state.move_to_reader(book, Some(c))?;
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
                    let mut book = app_state.reader_data.as_ref().unwrap().book_info.clone();

                    book.get_source_data_mut().set_chapter(c);
                    app_state.move_to_reader(book, Some(c))?;
                }
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

fn control_enter(app_state: &mut AppState) -> Result<()> {
    match app_state.current_screen {
        CurrentScreen::Library(lib_options) => match lib_options {
            LibraryOptions::Default => {
                let b = app_state.library_data.get_category_list().selected();
                if b.is_none() {
                    return Ok(());
                } else {
                    let book = b.unwrap();
                    if book.is_local() {
                        app_state
                            .update_screen(CurrentScreen::Library(LibraryOptions::LocalBookSelect));
                    } else {
                        app_state.update_screen(CurrentScreen::Library(
                            LibraryOptions::GlobalBookSelect,
                        ));
                    }
                }
            }
            LibraryOptions::LocalBookSelect => {
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
                        app_state.move_to_reader(BookInfo::Library(book), None)?;
                    }
                    1 => {
                        // Move to category
                        app_state.update_screen(CurrentScreen::Library(
                            LibraryOptions::MoveCategorySelect,
                        ));
                    }
                    2 => {
                        // Rename book
                        app_state.update_screen(CurrentScreen::Typing);
                    }
                    3 => {
                        // Start from beginning
                        book.update_progress(BookProgress::NONE);
                        app_state.move_to_reader(BookInfo::Library(book), None)?;
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
            LibraryOptions::GlobalBookSelect => {
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
                        app_state.move_to_reader(BookInfo::Library(book), Some(ch))?;
                    }
                    // Chapter List
                    1 => {
                        let chs = if let BookSource::Global(d) = book.source_data {
                            d.novel.chapters.clone()
                        } else {
                            unreachable!()
                        };

                        app_state.library_data.menu_data.ch_list =
                            Some(StatefulList::with_items(chs));

                        app_state
                            .update_screen(CurrentScreen::Library(LibraryOptions::ChapterView));
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
                        if let BookSource::Global(ref mut d) = book.source_data {
                            if let Some(data) = d.chapter_progress.get_mut(&1) {
                                data.progress = BookProgress::NONE
                            }
                        }
                        app_state.move_to_reader(BookInfo::Library(book), Some(1))?;
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
            LibraryOptions::MoveCategorySelect => {
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
            LibraryOptions::ChapterView => {
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
                    let novel = app_state
                        .library_data
                        .get_category_list()
                        .selected()
                        .unwrap();

                    let mut novel = novel.clone();
                    novel.source_data.set_chapter(chap + 1);
                    app_state.move_to_reader(BookInfo::Library(novel), Some(chap + 1))?;
                }
            }
        },
        CurrentScreen::Sources(source_options) => match source_options {
            SourceOptions::Default => {
                let source = app_state.source_data.get_list().selected();
                if source.is_some() {
                    app_state.update_screen(CurrentScreen::Sources(SourceOptions::SourceSelect));
                }
            }
            SourceOptions::SourceSelect => {
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
                        let source = &app_state.source_data.get_list().selected().unwrap().1;
                        let res = source.get_popular(SortOrder::Rating, 1)?;
                        app_state.source_data.novel_results = StatefulList::with_items(res);
                        app_state
                            .update_screen(CurrentScreen::Sources(SourceOptions::SearchResults));
                    }
                    _ => (),
                }
            }
            SourceOptions::SearchResults => {
                let url = &app_state.source_data.novel_results.selected().unwrap().url;
                let source = &app_state.source_data.sources.selected().unwrap().1;

                let full_book = source.parse_novel_and_chapters(url.clone())?;

                app_state.source_data.current_novel_chaps =
                    StatefulList::with_items(full_book.chapters.clone());
                app_state.source_data.current_novel = Some(full_book);

                app_state.update_screen(CurrentScreen::Sources(SourceOptions::BookView));
            }
            SourceOptions::BookView => {
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
                                app_state
                                    .move_to_reader(BookInfo::from_novel_temp(novel)?, Some(1))?;
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
                            .unwrap();
                        let novel = app_state.source_data.current_novel.clone().unwrap();
                        let mut info = BookInfo::from_novel_temp(novel)?;
                        info.get_source_data_mut().set_chapter(chap.chapter_no);
                        app_state.move_to_reader(info, Some(chap.chapter_no))?;
                    }
                    SourceBookBox::Summary => (),
                }
            }
        },
        CurrentScreen::Typing => unreachable!(),
        _ => (),
    }
    Ok(())
}
