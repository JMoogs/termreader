use crate::{
    appstate::{
        AppState, BookInfo, CurrentScreen, HistoryOptions, LibraryOptions, SettingsOptions,
        SourceOptions, UpdateOptions,
    },
    global::sources::{source_data::SourceBookBox, SortOrder},
    helpers::StatefulList,
    reader::buffer::BookProgress,
};
use anyhow::Result;
use crossterm::event::{self, KeyCode};

pub fn handle_controls(app_state: &mut AppState, event: event::KeyCode) -> Result<bool> {
    // On the rename screen, we just want to append to the string
    if matches!(
        app_state.current_screen,
        // CurrentScreen::Main(MenuType::Typing(_))
        CurrentScreen::Typing
    ) {
        handle_typing(event, app_state)?;
        return Ok(false);
    }
    // Other times when we're not inputting:
    match event {
        event::KeyCode::Esc | event::KeyCode::Char('q') => return control_back(app_state),
        KeyCode::Left | KeyCode::Right | KeyCode::Down | KeyCode::Up => {
            control_arrows(app_state, event)
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
            // app_state.text_buffer = String::new();
            // app_state.current_screen = match app_state.current_screen {
            //     CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingLocal)) => {
            //         CurrentScreen::Main(MenuType::Select(SelectBox::Local))
            //     }
            //     CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingGlobal)) => {
            //         CurrentScreen::Main(MenuType::Select(SelectBox::Global))
            //     }
            //     CurrentScreen::Main(MenuType::Typing(TypingOptions::Searching)) => {
            //         CurrentScreen::Main(MenuType::Select(SelectBox::Source))
            //     }
            //     _ => unreachable!(),
            // }
            todo!()
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
            // match app_state.current_screen {
            //     CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingLocal))
            //     | CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingGlobal)) => {
            //         let book = app_state
            //             .library_data
            //             .get_category_list()
            //             .selected()
            //             .unwrap()
            //             .clone();
            //         app_state
            //             .library_data
            //             .rename_book(book.id, app_state.text_buffer.clone());
            //         app_state.current_screen = CurrentScreen::Main(MenuType::Default);
            //     }
            //     CurrentScreen::Main(MenuType::Typing(TypingOptions::Searching)) => {
            //         let source = &app_state.source_data.get_list().selected().unwrap().1;
            //         let res = source.search_novels(&app_state.text_buffer)?;
            //         app_state.source_data.novel_results = StatefulList::with_items(res);
            //         app_state.current_screen = CurrentScreen::Main(MenuType::SearchResults);
            //     }
            //     _ => unreachable!(),
            // }
            // app_state.text_buffer = String::new();
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
            KeyCode::Char(']') | KeyCode::Char('[') => {
                match app_state.source_data.current_book_ui_option {
                    SourceBookBox::Options => {
                        app_state.source_data.current_book_ui_option = SourceBookBox::Chapters
                    }
                    SourceBookBox::Chapters => {
                        app_state.source_data.current_book_ui_option = SourceBookBox::Options
                    }
                }
            }
            _ => (),
        }
    }
}

fn control_back(app_state: &mut AppState) -> Result<bool> {
    if app_state.prev_screens.is_empty() {
        return Ok(true);
    }
    let prev = app_state.prev_screens.pop().unwrap();
    if prev == CurrentScreen::Typing {
        if app_state.prev_screens.is_empty() {
            return Ok(true);
        }
    }

    app_state.current_screen = app_state.prev_screens.pop().unwrap();
    return Ok(false);
}

fn control_arrows(app_state: &mut AppState, event: event::KeyCode) {
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
        CurrentScreen::Sources(SourceOptions::SearchResults) => match event {
            KeyCode::Up => app_state.source_data.novel_results.previous(),
            KeyCode::Down => app_state.source_data.novel_results.next(),
            _ => (),
        },
        CurrentScreen::Sources(SourceOptions::BookView) => match event {
            KeyCode::Up => match app_state.source_data.current_book_ui_option {
                SourceBookBox::Options => app_state.menu_options.source_book_options.previous(),
                SourceBookBox::Chapters => app_state.source_data.current_novel_chaps.previous(),
            },
            KeyCode::Down => match app_state.source_data.current_book_ui_option {
                SourceBookBox::Options => app_state.menu_options.source_book_options.next(),
                SourceBookBox::Chapters => app_state.source_data.current_novel_chaps.next(),
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
            _ => (),
        },
        _ => (),
    }
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

                let idx = app_state
                    .library_data
                    .get_category_list()
                    .state
                    .selected()
                    .unwrap();
                let mut book = app_state.library_data.get_category_list().items[idx].clone();
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
                todo!()
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
                    SourceBookBox::Options => todo!(),
                    SourceBookBox::Chapters => {
                        let chap = app_state
                            .source_data
                            .current_novel_chaps
                            .selected()
                            .unwrap();
                        let novel = app_state.source_data.current_novel.clone().unwrap();
                        // app_state.update_reader(BookInfo::from_novel(novel, None)?)?;
                        // app_state.current_screen = CurrentScreen::Reader;
                        app_state.move_to_reader(
                            BookInfo::from_novel_temp(novel)?,
                            Some(chap.chapter_no),
                        )?;
                    }
                }
            }
        },
        CurrentScreen::Typing => unreachable!(),
        _ => (),
    }
    Ok(())
}
