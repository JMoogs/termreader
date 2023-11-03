use crate::{
    appstate::{AppState, CurrentScreen, MenuType},
    reader::buffer::BookProgress,
};
use anyhow::Result;
use crossterm::event::{self, KeyCode};

pub fn handle_controls(app_state: &mut AppState, event: event::KeyCode) -> Result<bool> {
    // On the rename screen, we just want to append to the string
    if matches!(
        app_state.current_screen,
        CurrentScreen::Main(MenuType::Renaming(_))
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
                if let CurrentScreen::Main(MenuType::Renaming(0)) = app_state.current_screen {
                    app_state.current_screen = CurrentScreen::Main(MenuType::LocalSelected)
                } else {
                    app_state.current_screen = CurrentScreen::Main(MenuType::GlobalSelected)
                };
            }
            KeyCode::Enter => {
                let idx = app_state
                    .library_data
                    .get_category_list()
                    .state
                    .selected()
                    .unwrap();
                let book = app_state.library_data.get_category_list().items[idx].clone();
                app_state
                    .library_data
                    .rename_book(book.id, app_state.text_buffer.clone());
                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
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
                _ => unreachable!(),
            },
            MenuType::LocalSelected => (),
            MenuType::GlobalSelected => (),
            _ => (),
        },

        CurrentScreen::Reader => (),
    }
}

fn control_back(app_state: &mut AppState) -> Result<bool> {
    match app_state.current_screen {
        CurrentScreen::Main(menu) => match menu {
            MenuType::Default => return Ok(true),
            MenuType::LocalSelected => {
                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
                app_state
                    .reader_menu_options
                    .local_options
                    .state
                    .select(Some(0));
            }
            MenuType::GlobalSelected => {
                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
                app_state
                    .reader_menu_options
                    .global_options
                    .state
                    .select(Some(0));
            }
            MenuType::MoveCategories => {
                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
                app_state
                    .reader_menu_options
                    .category_moves
                    .state
                    .select(Some(0));
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
                    }
                }
                KeyCode::Down => {
                    if app_state.current_main_tab.in_library() {
                        app_state.library_data.get_category_list_mut().next();
                    }
                }
                _ => unreachable!(),
            },
            MenuType::LocalSelected | MenuType::GlobalSelected | MenuType::MoveCategories => {
                let list = if matches!(menu, MenuType::LocalSelected) {
                    &mut app_state.reader_menu_options.local_options
                } else if matches!(menu, MenuType::GlobalSelected) {
                    &mut app_state.reader_menu_options.global_options
                } else {
                    &mut app_state.reader_menu_options.category_moves
                };

                match event {
                    KeyCode::Up => list.previous(),
                    KeyCode::Down => list.next(),
                    _ => (),
                }
            }
            _ => (),
        },
        CurrentScreen::Reader => match event {
            KeyCode::Up => {
                app_state.reader_data.as_mut().unwrap().scroll_up(1);
            }
            KeyCode::Down => {
                app_state.reader_data.as_mut().unwrap().scroll_down(1);
            }
            _ => unreachable!(),
        },
    }
}

fn control_enter(app_state: &mut AppState) -> Result<()> {
    match app_state.current_screen {
        CurrentScreen::Main(menu) => match menu {
            MenuType::Default => {
                if app_state.current_main_tab.in_library() {
                    let idx = app_state.library_data.get_category_list().state.selected();
                    if idx.is_none() {
                        return Ok(());
                    } else {
                        let idx = idx.unwrap();
                        let book_local =
                            app_state.library_data.get_category_list().items[idx].is_local();
                        if book_local {
                            app_state.current_screen = CurrentScreen::Main(MenuType::LocalSelected);
                        } else {
                            app_state.current_screen =
                                CurrentScreen::Main(MenuType::GlobalSelected);
                        }
                    }
                }
            }
            MenuType::LocalSelected => {
                let option = app_state
                    .reader_menu_options
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
                        app_state.current_screen = CurrentScreen::Reader;
                        app_state.update_reader(book)?;
                    }
                    1 => {
                        // Move to category
                        app_state.current_screen = CurrentScreen::Main(MenuType::MoveCategories);
                    }
                    2 => {
                        // Rename book
                        app_state.current_screen = CurrentScreen::Main(MenuType::Renaming(0));
                    }
                    3 => {
                        // Start from beginning
                        app_state.current_screen = CurrentScreen::Reader;
                        book.update_progress(BookProgress::NONE);
                        app_state.update_reader(book)?;
                    }
                    4 => {
                        // Remove from library
                        app_state.library_data.remove_book(book.id);
                        app_state.current_screen = CurrentScreen::Main(MenuType::Default);
                    }
                    _ => unreachable!(),
                }
                app_state
                    .reader_menu_options
                    .local_options
                    .state
                    .select(Some(0));
            }
            MenuType::GlobalSelected => {
                app_state
                    .reader_menu_options
                    .global_options
                    .state
                    .select(Some(0));
                todo!()
            }
            MenuType::MoveCategories => {
                let idx = app_state
                    .library_data
                    .get_category_list()
                    .state
                    .selected()
                    .unwrap();
                let book = app_state.library_data.get_category_list().items[idx].id;

                let category_idx = app_state
                    .reader_menu_options
                    .category_moves
                    .state
                    .selected()
                    .unwrap();
                let category =
                    app_state.reader_menu_options.category_moves.items[category_idx].clone();

                app_state.library_data.move_category(book, Some(category));

                app_state.current_screen = CurrentScreen::Main(MenuType::Default);
            }
            _ => (),
        },
        CurrentScreen::Reader => (),
    }
    Ok(())
}
