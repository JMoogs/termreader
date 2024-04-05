use crate::id::ID;
use crate::Book;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct LibraryContext {
    books: HashMap<String, Vec<Book>>,
    default_category_name: String,
    category_order: Vec<String>,
}

impl LibraryContext {
    /// Creates an empty library
    pub(super) fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(String::from("Default"), Vec::new());
        Self {
            books: map,
            default_category_name: String::from("Default"),
            category_order: vec![String::from("Default")],
        }
    }

    pub(super) fn find_book(&self, id: ID) -> Option<&Book> {
        let lists = self.books.values();

        for list in lists {
            let search = list.iter().find(|i| i.get_id() == id);
            if search.is_none() {
                continue;
            }
            let res = search.unwrap();
            return Some(res);
        }

        None
    }

    pub(super) fn find_book_mut(&mut self, id: ID) -> Option<&mut Book> {
        let lists = self.books.values_mut();

        for list in lists {
            let search = list.iter_mut().find(|i| i.get_id() == id);
            if search.is_none() {
                continue;
            }
            let res = search.unwrap();
            return Some(res);
        }

        None
    }

    pub(super) fn remove_book(&mut self, id: ID) {
        let lists = self.books.values_mut();

        for list in lists {
            let search = list.iter().position(|i| i.get_id() == id);
            if search.is_none() {
                continue;
            }
            let pos = search.unwrap();
            list.remove(pos);
        }
    }

    pub(super) fn add_book(&mut self, book: Book, category: Option<String>) {
        match category {
            None => {
                let l = self.books.get_mut(&self.default_category_name).unwrap();
                l.push(book);
            }
            Some(c) => match self.books.get_mut(&c) {
                Some(list) => list.push(book),
                None => self.add_book(book, None),
            },
        }
    }

    pub(super) fn move_category(&mut self, id: ID, category_name: Option<String>) {
        if let Some(book) = self.find_book(id).cloned() {
            self.remove_book(id);
            self.add_book(book, category_name);
        }
    }

    pub(super) fn create_category(&mut self, name: String) {
        // Don't allow multiple categories with the same name.
        if self.books.contains_key(&name) {
            return;
        }
        self.books.insert(name.clone(), Vec::new());
        self.category_order.push(name);
    }

    pub(super) fn delete_category(&mut self, name: String) {
        // Don't allow the default category to be deleted.
        if self.default_category_name == name {
            return;
        }

        if let Some(v) = self.books.remove(&name) {
            let new_l = self.books.get_mut(&self.default_category_name).unwrap();
            for book in v {
                new_l.push(book);
            }
            self.category_order.retain(|x| x != &name)
        }
    }

    pub(super) fn rename_category(&mut self, old_name: String, new_name: String) {
        // Don't allow multiple categories with the same name.
        if self.books.contains_key(&new_name) {
            return;
        }

        if let Some(v) = self.books.remove(&old_name) {
            if self.default_category_name == old_name {
                self.default_category_name = new_name.clone();
            }

            for val in self.category_order.iter_mut() {
                if val == &old_name {
                    let _ = std::mem::replace(val, new_name.clone());
                }
            }
            self.books.insert(new_name, v);
        }
    }

    pub(super) fn get_categories(&self) -> &Vec<String> {
        &self.category_order
    }

    pub(super) fn get_books(&self) -> &HashMap<String, Vec<Book>> {
        &self.books
    }

    pub(super) fn get_books_mut(&mut self) -> &mut HashMap<String, Vec<Book>> {
        &mut self.books
    }

    pub(super) fn get_category_count(&self) -> usize {
        self.books.len()
    }
}
