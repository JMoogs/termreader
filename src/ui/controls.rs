use crate::{
    appstate::{AppState, BookInfo, CurrentScreen, MenuType, SelectBox, TypingOptions},
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
        CurrentScreen::Main(MenuType::Typing(_))
    ) {
        match event {
            KeyCode::Backspace => {
                app_state.text_buffer.pop();
            }
            KeyCode::Char(c) => {
                app_state.text_buffer.push(c);
            }
            KeyCode::Esc => {
                app_state.text_buffer = String::new();
                app_state.current_screen = match app_state.current_screen {
                    CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingLocal)) => {
                        CurrentScreen::Main(MenuType::Select(SelectBox::Local))
                    }
                    CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingGlobal)) => {
                        CurrentScreen::Main(MenuType::Select(SelectBox::Global))
                    }
                    CurrentScreen::Main(MenuType::Typing(TypingOptions::Searching)) => {
                        CurrentScreen::Main(MenuType::Select(SelectBox::Source))
                    }
                    _ => unreachable!(),
                }
            }
            KeyCode::Enter => {
                match app_state.current_screen {
                    CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingLocal))
                    | CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingGlobal)) => {
                        let book = app_state
                            .library_data
                            .get_category_list()
                            .selected()
                            .unwrap()
                            .clone();
                        app_state
                            .library_data
                            .rename_book(book.id, app_state.text_buffer.clone());
                        app_state.current_screen = CurrentScreen::Main(MenuType::Default);
                    }
                    CurrentScreen::Main(MenuType::Typing(TypingOptions::Searching)) => {
                        let source = &app_state.source_data.get_list().selected().unwrap().1;
                        let res = source.search_novels(&app_state.text_buffer)?;
                        app_state.source_data.novel_results = StatefulList::with_items(res);
                        app_state.current_screen = CurrentScreen::Main(MenuType::SearchResults);
                    }
                    _ => unreachable!(),
                }
                app_state.text_buffer = String::new();
            }
            _ => (),
        }
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

fn control_bracket(app_state: &mut AppState, event: event::KeyCode) {
    match app_state.current_screen {
        CurrentScreen::Main(menu) => match menu {
            MenuType::Default => match event {
                KeyCode::Char(']') => {
                    app_state.current_main_tab.next();
                }
                KeyCode::Char('[') => {
                    app_state.current_main_tab.previous();
                }
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
            },
            MenuType::SourceBookView => match event {
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
            },
            _ => (),
        },

        CurrentScreen::Reader => (),
    }
}

fn control_back(app_state: &mut AppState) -> Result<bool> {
    match app_state.current_screen {
        CurrentScreen::Main(menu) => match menu {
            MenuType::Default => return Ok(true),
            MenuType::Select(_) => {
                app_state.reset_selections();
                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
            }
            MenuType::SearchResults => {
                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
            }
            MenuType::SourceBookView => {
                app_state.current_screen = CurrentScreen::Main(MenuType::SearchResults)
            }
            _ => (),
        },
        CurrentScreen::Reader => {
            app_state.update_lib_from_reader()?;
            app_state.current_screen = CurrentScreen::Main(MenuType::Default);
        }
    }
    return Ok(false);
}

fn control_arrows(app_state: &mut AppState, event: event::KeyCode) {
    match app_state.current_screen {
        CurrentScreen::Main(menu) => match menu {
            MenuType::Default => match event {
                KeyCode::Up => {
                    if app_state.current_main_tab.in_library() {
                        app_state.library_data.get_category_list_mut().previous();
                    } else if app_state.current_main_tab.in_sources() {
                        app_state.source_data.get_list_mut().previous();
                    }
                }
                KeyCode::Down => {
                    if app_state.current_main_tab.in_library() {
                        app_state.library_data.get_category_list_mut().next();
                    } else if app_state.current_main_tab.in_sources() {
                        app_state.source_data.get_list_mut().next();
                    }
                }
                KeyCode::Right => {
                    app_state.current_main_tab.next();
                }
                KeyCode::Left => {
                    app_state.current_main_tab.previous();
                }
                _ => (),
            },
            MenuType::Select(sel) => {
                let list = if matches!(sel, SelectBox::Local) {
                    &mut app_state.menu_options.local_options
                } else if matches!(sel, SelectBox::Global) {
                    &mut app_state.menu_options.global_options
                } else if matches!(sel, SelectBox::MoveCategories) {
                    &mut app_state.menu_options.category_moves
                } else if matches!(sel, SelectBox::Source) {
                    &mut app_state.menu_options.source_options
                } else {
                    unimplemented!()
                };

                match event {
                    KeyCode::Up => list.previous(),
                    KeyCode::Down => list.next(),
                    _ => (),
                }
            }
            MenuType::SearchResults => match event {
                KeyCode::Up => app_state.source_data.novel_results.previous(),
                KeyCode::Down => app_state.source_data.novel_results.next(),
                _ => (),
            },
            MenuType::SourceBookView => match event {
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
    }
}

fn control_enter(app_state: &mut AppState) -> Result<()> {
    match app_state.current_screen {
        CurrentScreen::Main(menu) => match menu {
            MenuType::Default => {
                if app_state.current_main_tab.in_library() {
                    let b = app_state.library_data.get_category_list().selected();
                    if b.is_none() {
                        return Ok(());
                    } else {
                        let book = b.unwrap();
                        if book.is_local() {
                            app_state.current_screen =
                                CurrentScreen::Main(MenuType::Select(SelectBox::Local));
                        } else {
                            app_state.current_screen =
                                CurrentScreen::Main(MenuType::Select(SelectBox::Global));
                        }
                    }
                } else if app_state.current_main_tab.in_sources() {
                    let source = app_state.source_data.get_list().selected();
                    if source.is_some() {
                        app_state.current_screen =
                            CurrentScreen::Main(MenuType::Select(SelectBox::Source));
                    }
                }
            }
            MenuType::Select(SelectBox::Local) => {
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
                        app_state.current_screen =
                            CurrentScreen::Main(MenuType::Select(SelectBox::MoveCategories));
                    }
                    2 => {
                        // Rename book
                        app_state.current_screen =
                            CurrentScreen::Main(MenuType::Typing(TypingOptions::RenamingLocal));
                    }
                    3 => {
                        // Start from beginning
                        book.update_progress(BookProgress::NONE);
                        app_state.move_to_reader(BookInfo::Library(book), None)?;
                    }
                    4 => {
                        // Remove from library
                        app_state.library_data.remove_book(book.id);
                        app_state.current_screen = CurrentScreen::Main(MenuType::Default);
                    }
                    _ => unreachable!(),
                }
                app_state.menu_options.local_options.state.select(Some(0));
            }
            MenuType::Select(SelectBox::Global) => {
                app_state.menu_options.global_options.state.select(Some(0));
                todo!()
            }
            MenuType::Select(SelectBox::Source) => {
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
                        app_state.current_screen =
                            CurrentScreen::Main(MenuType::Typing(TypingOptions::Searching));
                    }
                    1 => {
                        let source = &app_state.source_data.get_list().selected().unwrap().1;
                        let res = source.get_popular(SortOrder::Rating, 1)?;
                        app_state.source_data.novel_results = StatefulList::with_items(res);
                        app_state.current_screen = CurrentScreen::Main(MenuType::SearchResults);
                    }
                    _ => (),
                }
            }
            MenuType::Select(SelectBox::MoveCategories) => {
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

                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
            }
            MenuType::SearchResults => {
                let url = &app_state.source_data.novel_results.selected().unwrap().url;
                let source = &app_state.source_data.sources.selected().unwrap().1;

                let full_book = source.parse_novel_and_chapters(url.clone())?;

                app_state.source_data.current_novel_chaps =
                    StatefulList::with_items(full_book.chapters.clone());
                app_state.source_data.current_novel = Some(full_book);

                app_state.current_screen = CurrentScreen::Main(MenuType::SourceBookView);
            }
            MenuType::SourceBookView => match app_state.source_data.current_book_ui_option {
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
                    app_state
                        .move_to_reader(BookInfo::from_novel_temp(novel)?, Some(chap.chapter_no))?;
                }
            },
            _ => (),
        },
        CurrentScreen::Reader => (),
    }
    Ok(())
}
